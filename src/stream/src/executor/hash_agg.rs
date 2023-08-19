// Copyright 2023 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::Hasher;
use std::marker::PhantomData;
use std::sync::Arc;

use futures::{stream, StreamExt};
use futures_async_stream::try_stream;
use iter_chunks::IterChunks;
use itertools::Itertools;
use risingwave_common::array::{Op, StreamChunk};
use risingwave_common::buffer::{Bitmap, BitmapBuilder};
use risingwave_common::catalog::Schema;
use risingwave_common::hash::{HashKey, PrecomputedBuildHasher};
use risingwave_common::types::ScalarImpl;
use risingwave_common::util::epoch::EpochPair;
use risingwave_common::util::iter_util::ZipEqFast;
use risingwave_expr::agg::AggCall;
use risingwave_storage::StateStore;

use super::agg_common::{AggExecutorArgs, HashAggExecutorExtraArgs};
use super::aggregation::{
    agg_call_filter_res, iter_table_storage, AggStateStorage, ChunkBuilder, DistinctDeduplicater,
    GroupKey, OnlyOutputIfHasInput,
};
use super::sort_buffer::SortBuffer;
use super::{
    expect_first_barrier, ActorContextRef, ExecutorInfo, PkIndicesRef, StreamExecutorResult,
    Watermark, BUCKET_NUMBER, DEFAULT_GHOST_CAP_MUTIPLE, HACK_JOIN_KEY_SIZE, INIT_GHOST_CAP,
    REAL_UPDATE_INTERVAL, SAMPLE_NUM_IN_TEN_K,
};
use crate::cache::{cache_may_stale, new_indexed_with_hasher, ManagedIndexedLruCache};
use crate::common::metrics::MetricsInfo;
use crate::common::table::state_table::StateTable;
use crate::error::StreamResult;
use crate::executor::aggregation::{generate_agg_schema, AggGroup as GenericAggGroup};
use crate::executor::error::StreamExecutorError;
use crate::executor::monitor::StreamingMetrics;
use crate::executor::{BoxedMessageStream, Executor, Message};
use crate::task::AtomicU64Ref;

type AggGroup<S> = GenericAggGroup<S, OnlyOutputIfHasInput>;
type AggGroupCache<K, S> = ManagedIndexedLruCache<K, AggGroup<S>, PrecomputedBuildHasher>;

/// [`HashAggExecutor`] could process large amounts of data using a state backend. It works as
/// follows:
///
/// * The executor pulls data from the upstream, and apply the data chunks to the corresponding
///   aggregation states.
/// * While processing, it will record which keys have been modified in this epoch using
///   `group_change_set`.
/// * Upon a barrier is received, the executor will call `.flush` on the storage backend, so that
///   all modifications will be flushed to the storage backend. Meanwhile, the executor will go
///   through `group_change_set`, and produce a stream chunk based on the state changes.
pub struct HashAggExecutor<K: HashKey, S: StateStore> {
    input: Box<dyn Executor>,
    inner: ExecutorInner<K, S>,
}

struct ExecutorInner<K: HashKey, S: StateStore> {
    _phantom: PhantomData<K>,

    actor_ctx: ActorContextRef,
    info: ExecutorInfo,

    /// Pk indices from input.
    input_pk_indices: Vec<usize>,

    /// Schema from input.
    input_schema: Schema,

    /// Indices of the columns
    /// all of the aggregation functions in this executor should depend on same group of keys
    group_key_indices: Vec<usize>,

    // The projection from group key in table schema to table pk.
    group_key_table_pk_projection: Arc<[usize]>,

    /// A [`HashAggExecutor`] may have multiple [`AggCall`]s.
    agg_calls: Vec<AggCall>,

    /// Index of row count agg call (`count(*)`) in the call list.
    row_count_index: usize,

    /// State storages for each aggregation calls.
    /// `None` means the agg call need not to maintain a state table by itself.
    storages: Vec<AggStateStorage<S>>,

    /// State table for the previous result of all agg calls.
    /// The outputs of all managed agg states are collected and stored in this
    /// table when `flush_data` is called.
    /// Also serves as EOWC sort buffer table.
    result_table: StateTable<S>,

    /// State tables for deduplicating rows on distinct key for distinct agg calls.
    /// One table per distinct column (may be shared by multiple agg calls).
    distinct_dedup_tables: HashMap<usize, StateTable<S>>,

    /// Watermark epoch.
    watermark_epoch: AtomicU64Ref,

    /// State cache size for extreme agg.
    extreme_cache_size: usize,

    /// The maximum size of the chunk produced by executor at a time.
    chunk_size: usize,

    /// Should emit on window close according to watermark?
    emit_on_window_close: bool,

    metrics: Arc<StreamingMetrics>,
}

impl<K: HashKey, S: StateStore> ExecutorInner<K, S> {
    fn all_state_tables_mut(&mut self) -> impl Iterator<Item = &mut StateTable<S>> {
        iter_table_storage(&mut self.storages)
            .chain(self.distinct_dedup_tables.values_mut())
            .chain(std::iter::once(&mut self.result_table))
    }
}

struct ExecutionVars<K: HashKey, S: StateStore> {
    stats: ExecutionStats,

    /// Cache for [`AggGroup`]s. `HashKey` -> `AggGroup`.
    agg_group_cache: AggGroupCache<K, S>,

    /// Changed group keys in the current epoch (before next flush).
    group_change_set: HashSet<K>,

    /// Distinct deduplicater to deduplicate input rows for each distinct agg call.
    distinct_dedup: DistinctDeduplicater<S>,

    /// Buffer watermarks on group keys received since last barrier.
    buffered_watermarks: Vec<Option<Watermark>>,

    /// Latest watermark on window column.
    window_watermark: Option<ScalarImpl>,

    /// Stream chunk builder.
    chunk_builder: ChunkBuilder,

    buffer: SortBuffer<S>,
}

struct ExecutionStats {
    /// How many times have we hit the cache of hash agg executor for the lookup of each key.
    lookup_miss_count: u64,
    lookup_real_miss_count: u64,
    total_lookup_count: u64,

    /// How many times have we hit the cache of hash agg executor for all the lookups generated by
    /// one StreamChunk.
    chunk_lookup_miss_count: u64,
    chunk_total_lookup_count: u64,

    bucket_size: usize,
    ghost_bucket_size: usize,
    ghost_start: usize,

    bucket_ids: Vec<String>,
    bucket_counts: Vec<usize>,
    ghost_bucket_counts: Vec<usize>,
}

impl ExecutionStats {
    fn new() -> Self {
        let mut bucket_ids = vec![];
        for i in 0..=BUCKET_NUMBER {
            bucket_ids.push(i.to_string());
        }
        let bucket_counts = vec![0; BUCKET_NUMBER + 1];
        let ghost_bucket_counts = vec![0; BUCKET_NUMBER + 1];
        Self {
            lookup_miss_count: 0,
            lookup_real_miss_count: 0,
            total_lookup_count: 0,
            chunk_lookup_miss_count: 0,
            chunk_total_lookup_count: 0,
            bucket_size: 1,
            ghost_bucket_size: 1,
            ghost_start: 0,
            bucket_ids,
            bucket_counts,
            ghost_bucket_counts,
        }
    }
}

impl<K: HashKey, S: StateStore> Executor for HashAggExecutor<K, S> {
    fn execute(self: Box<Self>) -> BoxedMessageStream {
        self.execute_inner().boxed()
    }

    fn schema(&self) -> &Schema {
        &self.inner.info.schema
    }

    fn pk_indices(&self) -> PkIndicesRef<'_> {
        &self.inner.info.pk_indices
    }

    fn identity(&self) -> &str {
        &self.inner.info.identity
    }
}

impl<K: HashKey, S: StateStore> HashAggExecutor<K, S> {
    pub fn new(mut args: AggExecutorArgs<S, HashAggExecutorExtraArgs>) -> StreamResult<Self> {
        let input_info = args.input.info();
        let schema = generate_agg_schema(
            args.input.as_ref(),
            &args.agg_calls,
            Some(&args.extra.group_key_indices),
        );
        args.result_table.set_actor_id(args.actor_ctx.id);

        let group_key_len = args.extra.group_key_indices.len();
        // NOTE: we assume the prefix of table pk is exactly the group key
        let group_key_table_pk_projection = &args.result_table.pk_indices()[..group_key_len];
        assert!(group_key_table_pk_projection
            .iter()
            .sorted()
            .copied()
            .eq(0..group_key_len));

        Ok(Self {
            input: args.input,
            inner: ExecutorInner {
                _phantom: PhantomData,
                actor_ctx: args.actor_ctx,
                info: ExecutorInfo {
                    schema,
                    pk_indices: args.pk_indices,
                    identity: format!("HashAggExecutor {:X}", args.executor_id),
                },
                input_pk_indices: input_info.pk_indices,
                input_schema: input_info.schema,
                group_key_indices: args.extra.group_key_indices,
                group_key_table_pk_projection: group_key_table_pk_projection.to_vec().into(),
                agg_calls: args.agg_calls,
                row_count_index: args.row_count_index,
                storages: args.storages,
                result_table: args.result_table,
                distinct_dedup_tables: args.distinct_dedup_tables,
                watermark_epoch: args.watermark_epoch,
                extreme_cache_size: args.extreme_cache_size,
                chunk_size: args.extra.chunk_size,
                emit_on_window_close: args.extra.emit_on_window_close,
                metrics: args.metrics,
            },
        })
    }

    /// Get visibilities that mask rows in the chunk for each group. The returned visibility
    /// is a `Bitmap` rather than `Option<Bitmap>` because it's likely to have multiple groups
    /// in one chunk.
    ///
    /// * `keys`: Hash Keys of rows.
    /// * `base_visibility`: Visibility of rows, `None` means all are visible.
    fn get_group_visibilities(keys: Vec<K>, base_visibility: Option<&Bitmap>) -> Vec<(K, Bitmap)> {
        let n_rows = keys.len();
        let mut vis_builders = HashMap::new();
        for (row_idx, key) in keys.into_iter().enumerate().filter(|(row_idx, _)| {
            base_visibility
                .map(|vis| vis.is_set(*row_idx))
                .unwrap_or(true)
        }) {
            vis_builders
                .entry(key)
                .or_insert_with(|| BitmapBuilder::zeroed(n_rows))
                .set(row_idx, true);
        }
        vis_builders
            .into_iter()
            .map(|(key, vis_builder)| (key, vis_builder.finish()))
            .collect()
    }

    async fn ensure_keys_in_cache(
        this: &mut ExecutorInner<K, S>,
        cache: &mut AggGroupCache<K, S>,
        keys: impl IntoIterator<Item = &K>,
        stats: &mut ExecutionStats,
    ) -> StreamExecutorResult<()> {
        let group_key_types = &this.info.schema.data_types()[..this.group_key_indices.len()];
        let futs = keys
            .into_iter()
            .filter_map(|key| {
                stats.total_lookup_count += 1;
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                let sampled = hasher.finish() % 10000 < SAMPLE_NUM_IN_TEN_K;
                let (exist, dis) = cache.contains_sampled(key, sampled);
                if let Some((distance, is_ghost)) = dis {
                    if is_ghost {
                        let bucket_index = if distance < stats.ghost_start as u32 {
                            0
                        } else if distance
                            > (stats.ghost_start + stats.ghost_bucket_size * BUCKET_NUMBER) as u32
                        {
                            BUCKET_NUMBER
                        } else {
                            (distance as usize - stats.ghost_start) / stats.ghost_bucket_size
                        };
                        stats.ghost_bucket_counts[bucket_index] += 1;
                    } else if sampled {
                        let bucket_index = if distance > (stats.bucket_size * BUCKET_NUMBER) as u32
                        {
                            BUCKET_NUMBER
                        } else {
                            distance as usize / stats.bucket_size
                        };
                        stats.bucket_counts[bucket_index] += 1;
                    }
                }
                if exist {
                    None
                } else {
                    stats.lookup_miss_count += 1;
                    Some(async {
                        // Create `AggGroup` for the current group if not exists. This will
                        // fetch previous agg result from the result table.
                        let agg_group = AggGroup::create(
                            Some(GroupKey::new(
                                key.deserialize(group_key_types)?,
                                Some(this.group_key_table_pk_projection.clone()),
                            )),
                            &this.agg_calls,
                            &this.storages,
                            &this.result_table,
                            &this.input_pk_indices,
                            this.row_count_index,
                            this.extreme_cache_size,
                            &this.input_schema,
                        )
                        .await?;
                        Ok::<_, StreamExecutorError>((key.clone(), agg_group))
                    })
                }
            })
            .collect_vec(); // collect is necessary to avoid lifetime issue of `agg_group_cache`

        stats.chunk_total_lookup_count += 1;
        if !futs.is_empty() {
            // If not all the required states/keys are in the cache, it's a chunk-level cache miss.
            stats.chunk_lookup_miss_count += 1;
            let mut buffered = stream::iter(futs).buffer_unordered(1).fuse();
            while let Some(result) = buffered.next().await {
                let (key, agg_group) = result?;
                if !agg_group.is_uninitialized() {
                    stats.lookup_real_miss_count += 1;
                    stats.ghost_bucket_counts[BUCKET_NUMBER] += 1;
                }
                cache.put(key, agg_group);
            }
        }
        Ok(())
    }

    async fn apply_chunk(
        this: &mut ExecutorInner<K, S>,
        vars: &mut ExecutionVars<K, S>,
        chunk: StreamChunk,
    ) -> StreamExecutorResult<()> {
        // Find groups in this chunk and generate visibility for each group key.
        let keys = K::build(&this.group_key_indices, chunk.data_chunk())?;
        let group_visibilities = Self::get_group_visibilities(keys, chunk.visibility());

        // Create `AggGroup` for each group if not exists.
        Self::ensure_keys_in_cache(
            this,
            &mut vars.agg_group_cache,
            group_visibilities.iter().map(|(k, _)| k),
            &mut vars.stats,
        )
        .await?;

        // Decompose the input chunk.
        let capacity = chunk.capacity();
        let (ops, columns, visibility) = chunk.into_inner();

        // Calculate the row visibility for every agg call.
        let mut call_visibilities = Vec::with_capacity(this.agg_calls.len());
        for agg_call in &this.agg_calls {
            let agg_call_filter_res = agg_call_filter_res(
                &this.actor_ctx,
                &this.info.identity,
                agg_call,
                &columns,
                visibility.as_ref(),
                capacity,
            )
            .await?;
            call_visibilities.push(agg_call_filter_res);
        }

        // Materialize input chunk if needed.
        this.storages
            .iter_mut()
            .zip_eq_fast(call_visibilities.iter().map(Option::as_ref))
            .for_each(|(storage, visibility)| {
                if let AggStateStorage::MaterializedInput { table, mapping } = storage {
                    let needed_columns = mapping
                        .upstream_columns()
                        .iter()
                        .map(|col_idx| columns[*col_idx].clone())
                        .collect();
                    table.write_chunk(StreamChunk::new(
                        ops.clone(),
                        needed_columns,
                        visibility.cloned(),
                    ));
                }
            });

        // Apply chunk to each of the state (per agg_call), for each group.
        for (key, visibility) in group_visibilities {
            let mut agg_group = vars.agg_group_cache.peek_mut(&key).unwrap();
            let visibilities = call_visibilities
                .iter()
                .map(Option::as_ref)
                .map(|call_vis| call_vis.map_or_else(|| visibility.clone(), |v| v & &visibility))
                .map(Some)
                .collect();
            let visibilities = vars
                .distinct_dedup
                .dedup_chunk(
                    &ops,
                    &columns,
                    visibilities,
                    &mut this.distinct_dedup_tables,
                    agg_group.group_key(),
                    this.actor_ctx.clone(),
                )
                .await?;
            agg_group.apply_chunk(&mut this.storages, &ops, &columns, visibilities)?;
            // Mark the group as changed.
            vars.group_change_set.insert(key);
        }

        Ok(())
    }

    #[try_stream(ok = StreamChunk, error = StreamExecutorError)]
    async fn flush_data<'a>(
        this: &'a mut ExecutorInner<K, S>,
        vars: &'a mut ExecutionVars<K, S>,
        epoch: EpochPair,
    ) {
        // Update metrics.
        let actor_id_str = this.actor_ctx.id.to_string();
        let table_id_str = this.result_table.table_id().to_string();

        this.metrics
            .lookup_new_count
            .with_label_values(&[&table_id_str, &actor_id_str, "agg"])
            .inc_by(vars.stats.lookup_miss_count - vars.stats.lookup_real_miss_count);
        this.metrics
            .agg_lookup_miss_count
            .with_label_values(&[&table_id_str, &actor_id_str])
            .inc_by(vars.stats.lookup_miss_count);
        vars.stats.lookup_miss_count = 0;
        this.metrics
            .agg_lookup_real_miss_count
            .with_label_values(&[&table_id_str, &actor_id_str])
            .inc_by(vars.stats.lookup_real_miss_count);
        vars.stats.lookup_real_miss_count = 0;
        this.metrics
            .agg_total_lookup_count
            .with_label_values(&[&table_id_str, &actor_id_str])
            .inc_by(vars.stats.total_lookup_count);
        vars.stats.total_lookup_count = 0;
        let cache_entry_count = vars.agg_group_cache.len();
        this.metrics
            .agg_cached_keys
            .with_label_values(&[&table_id_str, &actor_id_str])
            .set(vars.agg_group_cache.len() as i64);
        this.metrics
            .agg_chunk_lookup_miss_count
            .with_label_values(&[&table_id_str, &actor_id_str])
            .inc_by(vars.stats.chunk_lookup_miss_count);
        vars.stats.chunk_lookup_miss_count = 0;
        this.metrics
            .agg_chunk_total_lookup_count
            .with_label_values(&[&table_id_str, &actor_id_str])
            .inc_by(vars.stats.chunk_total_lookup_count);
        vars.stats.chunk_total_lookup_count = 0;

        for i in 0..=BUCKET_NUMBER {
            let count = vars.stats.bucket_counts[i];
            this.metrics
                .cache_real_resue_distance_bucket_count
                .with_label_values(&[
                    &actor_id_str,
                    &table_id_str,
                    "agg",
                    &vars.stats.bucket_ids[i],
                ])
                .inc_by(count as u64);
            vars.stats.bucket_counts[i] = 0;

            let ghost_count = vars.stats.ghost_bucket_counts[i];
            this.metrics
                .cache_ghost_resue_distance_bucket_count
                .with_label_values(&[
                    &actor_id_str,
                    &table_id_str,
                    "agg",
                    &vars.stats.bucket_ids[i],
                ])
                .inc_by(ghost_count as u64);
            vars.stats.ghost_bucket_counts[i] = 0;
        }

        this.metrics
            .mrc_bucket_info
            .with_label_values(&[&table_id_str, &actor_id_str, "agg", "bucket"])
            .set(vars.agg_group_cache.bucket_count() as i64);
        this.metrics
            .mrc_bucket_info
            .with_label_values(&[&table_id_str, &actor_id_str, "agg", "ghost_bucket"])
            .set(vars.agg_group_cache.ghost_bucket_count() as i64);
        this.metrics
            .mrc_bucket_info
            .with_label_values(&[&table_id_str, &actor_id_str, "agg", "ghost_start"])
            .set(vars.stats.ghost_start as i64);
        this.metrics
            .mrc_bucket_info
            .with_label_values(&[&table_id_str, &actor_id_str, "agg", "ghost_cap"])
            .set(vars.agg_group_cache.ghost_cap() as i64);

        Self::update_bucket_size(
            &mut vars.agg_group_cache,
            &mut vars.stats,
            cache_entry_count,
        );

        let window_watermark = vars.window_watermark.take();
        let n_dirty_group = vars.group_change_set.len();

        // Flush agg states if needed.
        for key in &vars.group_change_set {
            let agg_group = vars
                .agg_group_cache
                .peek_mut(key)
                .expect("changed group must have corresponding AggGroup");
            agg_group.flush_state_if_needed(&mut this.storages).await?;
        }

        let futs_of_all_groups = vars
            .group_change_set
            .drain()
            .map(|key| {
                // Get agg group of the key.
                vars.agg_group_cache
                    .peek_mut_unsafe(&key)
                    .expect("changed group must have corresponding AggGroup")
            })
            .map(|mut agg_group| {
                let storages = &this.storages;
                // SAFETY:
                // 1. `key`s in `keys_in_batch` are unique by nature, because they're
                // from `group_change_set` which is a set.
                //
                // 2. `MutGuard` should not be sent to other tasks.
                let mut agg_group = unsafe { agg_group.as_mut_guard() };
                async move {
                    // Get agg outputs and build change.
                    let curr_outputs = agg_group.get_outputs(storages).await?;
                    let change = agg_group.build_change(curr_outputs);
                    Ok::<_, StreamExecutorError>(change)
                }
            });

        // TODO(rc): figure out a more reasonable concurrency limit.
        const MAX_CONCURRENT_TASKS: usize = 100;
        let mut futs_batches = IterChunks::chunks(futs_of_all_groups, MAX_CONCURRENT_TASKS);
        while let Some(futs) = futs_batches.next() {
            // Compute agg result changes for each group, and emit changes accordingly.
            let changes = futures::future::try_join_all(futs).await?;

            // Emit from changes
            if this.emit_on_window_close {
                for change in changes.into_iter().flatten() {
                    // For EOWC, write change to the sort buffer.
                    vars.buffer.apply_change(change, &mut this.result_table);
                }
            } else {
                for change in changes.into_iter().flatten() {
                    // For EOU, write change to result table and directly yield the change.
                    this.result_table.write_record(change.as_ref());
                    if let Some(chunk) = vars.chunk_builder.append_record(change) {
                        yield chunk;
                    }
                }
            }
        }

        // Emit remaining results from result table.
        if this.emit_on_window_close {
            if let Some(watermark) = window_watermark.as_ref() {
                #[for_await]
                for row in vars
                    .buffer
                    .consume(watermark.clone(), &mut this.result_table)
                {
                    let row = row?;
                    if let Some(chunk) = vars.chunk_builder.append_row(Op::Insert, row) {
                        yield chunk;
                    }
                }
            }
        }

        // Yield the remaining rows in chunk builder.
        if let Some(chunk) = vars.chunk_builder.take() {
            yield chunk;
        }

        if n_dirty_group == 0 && window_watermark.is_none() {
            // Nothing is expected to be changed.
            this.all_state_tables_mut().for_each(|table| {
                table.commit_no_data_expected(epoch);
            });
        } else {
            if let Some(watermark) = window_watermark {
                // Update watermark of state tables, for state cleaning.
                this.all_state_tables_mut()
                    .for_each(|table| table.update_watermark(watermark.clone(), false));
            }
            // Commit all state tables.
            futures::future::try_join_all(
                this.all_state_tables_mut()
                    .map(|table| async { table.commit(epoch).await }),
            )
            .await?;
        }

        // Flush distinct dedup state.
        vars.distinct_dedup
            .flush(&mut this.distinct_dedup_tables, this.actor_ctx.clone())?;

        // Evict cache to target capacity.
        vars.agg_group_cache.evict();
    }

    #[try_stream(ok = Message, error = StreamExecutorError)]
    async fn execute_inner(self) {
        let HashAggExecutor {
            input,
            inner: mut this,
        } = self;

        let window_col_idx_in_group_key = this.result_table.pk_indices()[0];
        let window_col_idx = this.group_key_indices[window_col_idx_in_group_key];

        let agg_group_cache_metrics_info = MetricsInfo::new(
            this.metrics.clone(),
            this.result_table.table_id(),
            this.actor_ctx.id,
            "agg result table",
        );

        let cache = new_indexed_with_hasher(
            this.watermark_epoch.clone(),
            agg_group_cache_metrics_info,
            PrecomputedBuildHasher,
            INIT_GHOST_CAP,
            REAL_UPDATE_INTERVAL,
            BUCKET_NUMBER,
        );

        let mut vars = ExecutionVars {
            stats: ExecutionStats::new(),
            agg_group_cache: cache,
            group_change_set: HashSet::new(),
            distinct_dedup: DistinctDeduplicater::new(
                &this.agg_calls,
                &this.watermark_epoch,
                &this.distinct_dedup_tables,
                this.actor_ctx.id,
                this.metrics.clone(),
            ),
            buffered_watermarks: vec![None; this.group_key_indices.len()],
            window_watermark: None,
            chunk_builder: ChunkBuilder::new(this.chunk_size, &this.info.schema.data_types()),
            buffer: SortBuffer::new(window_col_idx_in_group_key, &this.result_table),
        };

        // TODO(rc): use something like a `ColumnMapping` type
        let group_key_invert_idx = {
            let mut group_key_invert_idx = vec![None; input.info().schema.len()];
            for (group_key_seq, group_key_idx) in this.group_key_indices.iter().enumerate() {
                group_key_invert_idx[*group_key_idx] = Some(group_key_seq);
            }
            group_key_invert_idx
        };

        // First barrier
        let mut input = input.execute();
        let barrier = expect_first_barrier(&mut input).await?;
        this.all_state_tables_mut().for_each(|table| {
            table.init_epoch(barrier.epoch);
        });
        vars.agg_group_cache.update_epoch(barrier.epoch.curr);
        vars.distinct_dedup.dedup_caches_mut().for_each(|cache| {
            cache.update_epoch(barrier.epoch.curr);
        });

        yield Message::Barrier(barrier);

        #[for_await]
        for msg in input {
            let msg = msg?;
            vars.agg_group_cache.evict_except_cur_epoch();
            match msg {
                Message::Watermark(watermark) => {
                    let group_key_seq = group_key_invert_idx[watermark.col_idx];
                    if let Some(group_key_seq) = group_key_seq {
                        if watermark.col_idx == window_col_idx {
                            vars.window_watermark = Some(watermark.val.clone());
                        }
                        vars.buffered_watermarks[group_key_seq] =
                            Some(watermark.with_idx(group_key_seq));
                    }
                }
                Message::Chunk(chunk) => {
                    Self::apply_chunk(&mut this, &mut vars, chunk).await?;
                }
                Message::Barrier(barrier) => {
                    #[for_await]
                    for chunk in Self::flush_data(&mut this, &mut vars, barrier.epoch) {
                        yield Message::Chunk(chunk?);
                    }

                    if this.emit_on_window_close {
                        // ignore watermarks on other columns
                        if let Some(watermark) =
                            vars.buffered_watermarks[window_col_idx_in_group_key].take()
                        {
                            yield Message::Watermark(watermark);
                        }
                    } else {
                        for buffered_watermark in &mut vars.buffered_watermarks {
                            if let Some(watermark) = buffered_watermark.take() {
                                yield Message::Watermark(watermark);
                            }
                        }
                    }

                    // Update the vnode bitmap for state tables of all agg calls if asked.
                    if let Some(vnode_bitmap) = barrier.as_update_vnode_bitmap(this.actor_ctx.id) {
                        let previous_vnode_bitmap = this.result_table.vnodes().clone();
                        this.all_state_tables_mut().for_each(|table| {
                            let _ = table.update_vnode_bitmap(vnode_bitmap.clone());
                        });

                        // Manipulate the cache if necessary.
                        if cache_may_stale(&previous_vnode_bitmap, &vnode_bitmap) {
                            vars.agg_group_cache.clear();
                            vars.distinct_dedup.dedup_caches_mut().for_each(|cache| {
                                cache.clear();
                            });
                        }
                    }

                    // Update the current epoch.
                    vars.agg_group_cache.update_epoch(barrier.epoch.curr);
                    vars.distinct_dedup.dedup_caches_mut().for_each(|cache| {
                        cache.update_epoch(barrier.epoch.curr);
                    });

                    if barrier.is_cache() {
                        if let Some(mutation) = barrier.mutation.as_deref() {
                            match mutation {
                                crate::executor::Mutation::Cache {
                                    new_fragment_cache_sizes,
                                } => {
                                    if let Some(table_cache_sizes) =
                                        new_fragment_cache_sizes.get(&this.actor_ctx.fragment_id)
                                    {
                                        let table_id = this.result_table.table_id();
                                        if let Some(cache_size) = table_cache_sizes.get(&table_id) {
                                            vars.agg_group_cache
                                                .update_size_limit(*cache_size as usize);
                                            tracing::info!(
                                                    "WKXLOG: Success update_size_limit table_id {}, to cache size: {}",
                                                    table_id,
                                                    cache_size
                                                );
                                        } else {
                                            tracing::warn!(
                                                    "WKXLOG: WARN!!! update_size_limit cannot find table_id {} in table_cache_size: {:?}",
                                                    table_id,
                                                    table_cache_sizes
                                                );
                                        }
                                        tracing::info!(
                                            "WKXNB! agg table cache updated! table_cache_sizes: {:?}, fragment_id: {}",
                                            table_cache_sizes,
                                            this.actor_ctx.fragment_id
                                        );
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    yield Message::Barrier(barrier);
                }
            }
        }
    }

    fn update_bucket_size(
        agg_group_cache: &mut AggGroupCache<K, S>,
        stats: &mut ExecutionStats,
        entry_count: usize,
    ) {
        let old_entry_count = stats.bucket_size * BUCKET_NUMBER;
        if (old_entry_count as f64 * 1.2 < entry_count as f64
            || old_entry_count as f64 * 0.7 > entry_count as f64)
            && entry_count > 100
        {
            let mut ghost_cap_multiple = DEFAULT_GHOST_CAP_MUTIPLE;
            let k_size = agg_group_cache.key_size.unwrap_or(HACK_JOIN_KEY_SIZE);
            if let Some(kv_size) = agg_group_cache.get_avg_kv_size() {
                let v_size = kv_size - k_size;
                let multiple = v_size / k_size;
                ghost_cap_multiple = usize::min(usize::max(multiple, 1), ghost_cap_multiple);
            }
            let ghost_cap = ghost_cap_multiple * entry_count;

            stats.bucket_size = std::cmp::max(
                (entry_count as f64 * 1.1 / BUCKET_NUMBER as f64).round() as usize,
                1,
            );
            stats.ghost_bucket_size = std::cmp::max(
                ((entry_count as f64 * 0.3 + ghost_cap as f64) / BUCKET_NUMBER as f64).round()
                    as usize,
                1,
            );
            stats.ghost_start = std::cmp::max((entry_count as f64 * 0.8).round() as usize, 1);
            info!(
                "WKXLOG ghost_start switch to {}, old_entry_count: {}, new_entry_count: {}",
                stats.ghost_start, old_entry_count, entry_count
            );
            agg_group_cache.set_ghost_cap(ghost_cap);
        }
    }
}
