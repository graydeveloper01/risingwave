// Copyright 2024 RisingWave Labs
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

#![feature(lint_reasons)]
#![feature(let_chains)]
#![cfg_attr(coverage, feature(coverage_attribute))]

mod server;

use std::time::Duration;

use clap::Parser;
pub use error::{MetaError, MetaResult};
use redact::Secret;
use risingwave_common::config::OverrideConfig;
use risingwave_common::util::meta_addr::MetaAddressStrategy;
use risingwave_common::util::resource_util;
use risingwave_common::util::tokio_util::sync::CancellationToken;
use risingwave_common::{GIT_SHA, RW_VERSION};
use risingwave_common_heap_profiling::HeapProfiler;
use risingwave_meta::*;
use risingwave_meta_service::*;
pub use rpc::{ElectionClient, ElectionMember, EtcdElectionClient};
use server::rpc_serve;
pub use server::started::get as is_server_started;

use crate::manager::MetaOpts;

#[derive(Debug, Clone, Parser, OverrideConfig)]
#[command(version, about = "The central metadata management service")]
pub struct MetaNodeOpts {
    // TODO: use `SocketAddr`
    #[clap(long, env = "RW_LISTEN_ADDR", default_value = "127.0.0.1:5690")]
    pub listen_addr: String,

    /// The address for contacting this instance of the service.
    /// This would be synonymous with the service's "public address"
    /// or "identifying address".
    /// It will serve as a unique identifier in cluster
    /// membership and leader election. Must be specified for etcd backend.
    #[clap(long, env = "RW_ADVERTISE_ADDR", default_value = "127.0.0.1:5690")]
    pub advertise_addr: String,

    #[clap(long, env = "RW_DASHBOARD_HOST")]
    pub dashboard_host: Option<String>,

    /// We will start a http server at this address via `MetricsManager`.
    /// Then the prometheus instance will poll the metrics from this address.
    #[clap(long, env = "RW_PROMETHEUS_HOST", alias = "prometheus-host")]
    pub prometheus_listener_addr: Option<String>,

    #[clap(long, hide = true, env = "RW_ETCD_ENDPOINTS", default_value_t = String::from(""))]
    pub etcd_endpoints: String,

    /// Enable authentication with etcd. By default disabled.
    #[clap(long, hide = true, env = "RW_ETCD_AUTH")]
    pub etcd_auth: bool,

    /// Username of etcd, required when --etcd-auth is enabled.
    #[clap(long, hide = true, env = "RW_ETCD_USERNAME", default_value = "")]
    pub etcd_username: String,

    /// Password of etcd, required when --etcd-auth is enabled.
    #[clap(long, hide = true, env = "RW_ETCD_PASSWORD", default_value = "")]
    pub etcd_password: Secret<String>,

    /// Endpoint of the SQL service, make it non-option when SQL service is required.
    #[clap(long, hide = true, env = "RW_SQL_ENDPOINT")]
    pub sql_endpoint: Option<Secret<String>>,

    /// Username of sql backend, required when meta backend set to MySQL or PostgreSQL.
    #[clap(long, hide = true, env = "RW_SQL_USERNAME", default_value = "")]
    pub sql_username: String,

    /// Password of sql backend, required when meta backend set to MySQL or PostgreSQL.
    #[clap(long, hide = true, env = "RW_SQL_PASSWORD", default_value = "")]
    pub sql_password: Secret<String>,

    /// Database of sql backend, required when meta backend set to MySQL or PostgreSQL.
    #[clap(long, hide = true, env = "RW_SQL_DATABASE", default_value = "")]
    pub sql_database: String,

    /// The HTTP REST-API address of the Prometheus instance associated to this cluster.
    /// This address is used to serve `PromQL` queries to Prometheus.
    /// It is also used by Grafana Dashboard Service to fetch metrics and visualize them.
    #[clap(long, env = "RW_PROMETHEUS_ENDPOINT")]
    pub prometheus_endpoint: Option<String>,

    /// The additional selector used when querying Prometheus.
    ///
    /// The format is same as `PromQL`. Example: `instance="foo",namespace="bar"`
    #[clap(long, env = "RW_PROMETHEUS_SELECTOR")]
    pub prometheus_selector: Option<String>,

    /// Default tag for the endpoint created when creating a privatelink connection.
    /// Will be appended to the tags specified in the `tags` field in with clause in `create
    /// connection`.
    #[clap(long, hide = true, env = "RW_PRIVATELINK_ENDPOINT_DEFAULT_TAGS")]
    pub privatelink_endpoint_default_tags: Option<String>,

    #[clap(long, hide = true, env = "RW_VPC_ID")]
    pub vpc_id: Option<String>,

    #[clap(long, hide = true, env = "RW_VPC_SECURITY_GROUP_ID")]
    pub security_group_id: Option<String>,

    /// The path of `risingwave.toml` configuration file.
    ///
    /// If empty, default configuration values will be used.
    #[clap(long, env = "RW_CONFIG_PATH", default_value = "")]
    pub config_path: String,

    #[clap(long, hide = true, env = "RW_BACKEND", value_enum)]
    #[override_opts(path = meta.backend)]
    pub backend: Option<MetaBackend>,

    /// The interval of periodic barrier.
    #[clap(long, hide = true, env = "RW_BARRIER_INTERVAL_MS")]
    #[override_opts(path = system.barrier_interval_ms)]
    pub barrier_interval_ms: Option<u32>,

    /// Target size of the Sstable.
    #[clap(long, hide = true, env = "RW_SSTABLE_SIZE_MB")]
    #[override_opts(path = system.sstable_size_mb)]
    pub sstable_size_mb: Option<u32>,

    /// Size of each block in bytes in SST.
    #[clap(long, hide = true, env = "RW_BLOCK_SIZE_KB")]
    #[override_opts(path = system.block_size_kb)]
    pub block_size_kb: Option<u32>,

    /// False positive probability of bloom filter.
    #[clap(long, hide = true, env = "RW_BLOOM_FALSE_POSITIVE")]
    #[override_opts(path = system.bloom_false_positive)]
    pub bloom_false_positive: Option<f64>,

    /// State store url
    #[clap(long, hide = true, env = "RW_STATE_STORE")]
    #[override_opts(path = system.state_store)]
    pub state_store: Option<String>,

    /// Remote directory for storing data and metadata objects.
    #[clap(long, hide = true, env = "RW_DATA_DIRECTORY")]
    #[override_opts(path = system.data_directory)]
    pub data_directory: Option<String>,

    /// Whether config object storage bucket lifecycle to purge stale data.
    #[clap(long, hide = true, env = "RW_DO_NOT_CONFIG_BUCKET_LIFECYCLE")]
    #[override_opts(path = meta.do_not_config_object_storage_lifecycle)]
    pub do_not_config_object_storage_lifecycle: Option<bool>,

    /// Remote storage url for storing snapshots.
    #[clap(long, hide = true, env = "RW_BACKUP_STORAGE_URL")]
    #[override_opts(path = system.backup_storage_url)]
    pub backup_storage_url: Option<String>,

    /// Remote directory for storing snapshots.
    #[clap(long, hide = true, env = "RW_BACKUP_STORAGE_DIRECTORY")]
    #[override_opts(path = system.backup_storage_directory)]
    pub backup_storage_directory: Option<String>,

    /// Enable heap profile dump when memory usage is high.
    #[clap(long, hide = true, env = "RW_HEAP_PROFILING_DIR")]
    #[override_opts(path = server.heap_profiling.dir)]
    pub heap_profiling_dir: Option<String>,

    /// Exit if idle for a certain period of time.
    #[clap(long, hide = true, env = "RW_DANGEROUS_MAX_IDLE_SECS")]
    #[override_opts(path = meta.dangerous_max_idle_secs)]
    pub dangerous_max_idle_secs: Option<u64>,

    /// Endpoint of the connector node.
    #[deprecated = "connector node has been deprecated."]
    #[clap(long, hide = true, env = "RW_CONNECTOR_RPC_ENDPOINT")]
    pub connector_rpc_endpoint: Option<String>,
}

impl risingwave_common::opts::Opts for MetaNodeOpts {
    fn name() -> &'static str {
        "meta"
    }

    fn meta_addr(&self) -> MetaAddressStrategy {
        format!("http://{}", self.listen_addr)
            .parse()
            .expect("invalid listen address")
    }
}

use std::future::Future;
use std::pin::Pin;

use risingwave_common::config::{load_config, MetaBackend, RwConfig};
use tracing::info;

/// Start meta node
pub fn start(
    opts: MetaNodeOpts,
    shutdown: CancellationToken,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    // WARNING: don't change the function signature. Making it `async fn` will cause
    // slow compile in release mode.
    Box::pin(async move {
        info!("Starting meta node");
        info!("> options: {:?}", opts);
        let config = load_config(&opts.config_path, &opts);
        info!("> config: {:?}", config);
        info!("> version: {} ({})", RW_VERSION, GIT_SHA);
        let listen_addr = opts.listen_addr.parse().unwrap();
        let dashboard_addr = opts.dashboard_host.map(|x| x.parse().unwrap());
        let prometheus_addr = opts.prometheus_listener_addr.map(|x| x.parse().unwrap());
        let backend = match config.meta.backend {
            MetaBackend::Etcd => MetaStoreBackend::Etcd {
                endpoints: opts
                    .etcd_endpoints
                    .split(',')
                    .map(|x| x.to_string())
                    .collect(),
                credentials: match opts.etcd_auth {
                    true => Some((
                        opts.etcd_username,
                        opts.etcd_password.expose_secret().to_string(),
                    )),
                    false => None,
                },
            },
            MetaBackend::Mem => MetaStoreBackend::Mem,
            MetaBackend::Sql => MetaStoreBackend::Sql {
                endpoint: opts
                    .sql_endpoint
                    .expect("sql endpoint is required")
                    .expose_secret()
                    .to_string(),
            },
            MetaBackend::Sqlite => MetaStoreBackend::Sql {
                endpoint: format!(
                    "sqlite://{}?mode=rwc",
                    opts.sql_endpoint
                        .expect("sql endpoint is required")
                        .expose_secret()
                ),
            },
            MetaBackend::Postgres => MetaStoreBackend::Sql {
                endpoint: format!(
                    "postgres://{}:{}@{}/{}",
                    opts.sql_username,
                    opts.sql_password.expose_secret(),
                    opts.sql_endpoint
                        .expect("sql endpoint is required")
                        .expose_secret(),
                    opts.sql_database
                ),
            },
            MetaBackend::Mysql => MetaStoreBackend::Sql {
                endpoint: format!(
                    "mysql://{}:{}@{}/{}",
                    opts.sql_username,
                    opts.sql_password.expose_secret(),
                    opts.sql_endpoint
                        .expect("sql endpoint is required")
                        .expose_secret(),
                    opts.sql_database
                ),
            },
        };

        validate_config(&config);

        let total_memory_bytes = resource_util::memory::system_memory_available_bytes();
        let heap_profiler =
            HeapProfiler::new(total_memory_bytes, config.server.heap_profiling.clone());
        // Run a background heap profiler
        heap_profiler.start();

        let max_heartbeat_interval =
            Duration::from_secs(config.meta.max_heartbeat_interval_secs as u64);
        let max_idle_ms = config.meta.dangerous_max_idle_secs.unwrap_or(0) * 1000;
        let in_flight_barrier_nums = config.streaming.in_flight_barrier_nums;
        let privatelink_endpoint_default_tags =
            opts.privatelink_endpoint_default_tags.map(|tags| {
                tags.split(',')
                    .map(|s| {
                        let key_val = s.split_once('=').unwrap();
                        (key_val.0.to_string(), key_val.1.to_string())
                    })
                    .collect()
            });

        let add_info = AddressInfo {
            advertise_addr: opts.advertise_addr.to_owned(),
            listen_addr,
            prometheus_addr,
            dashboard_addr,
        };

        const MIN_TIMEOUT_INTERVAL_SEC: u64 = 20;
        let compaction_task_max_progress_interval_secs = {
            let retry_config = &config.storage.object_store.retry;
            let max_streming_read_timeout_ms = (retry_config.streaming_read_attempt_timeout_ms
                + retry_config.req_backoff_max_delay_ms)
                * retry_config.streaming_read_retry_attempts as u64;
            let max_streaming_upload_timeout_ms = (retry_config
                .streaming_upload_attempt_timeout_ms
                + retry_config.req_backoff_max_delay_ms)
                * retry_config.streaming_upload_retry_attempts as u64;
            let max_upload_timeout_ms = (retry_config.upload_attempt_timeout_ms
                + retry_config.req_backoff_max_delay_ms)
                * retry_config.upload_retry_attempts as u64;
            let max_read_timeout_ms = (retry_config.read_attempt_timeout_ms
                + retry_config.req_backoff_max_delay_ms)
                * retry_config.read_retry_attempts as u64;
            let max_timeout_ms = max_streming_read_timeout_ms
                .max(max_upload_timeout_ms)
                .max(max_streaming_upload_timeout_ms)
                .max(max_read_timeout_ms)
                .max(config.meta.compaction_task_max_progress_interval_secs * 1000);
            max_timeout_ms / 1000
        } + MIN_TIMEOUT_INTERVAL_SEC;

        rpc_serve(
            add_info,
            backend,
            max_heartbeat_interval,
            config.meta.meta_leader_lease_secs,
            MetaOpts {
                enable_recovery: !config.meta.disable_recovery,
                disable_automatic_parallelism_control: config
                    .meta
                    .disable_automatic_parallelism_control,
                parallelism_control_batch_size: config.meta.parallelism_control_batch_size,
                parallelism_control_trigger_period_sec: config
                    .meta
                    .parallelism_control_trigger_period_sec,
                parallelism_control_trigger_first_delay_sec: config
                    .meta
                    .parallelism_control_trigger_first_delay_sec,
                in_flight_barrier_nums,
                max_idle_ms,
                compaction_deterministic_test: config.meta.enable_compaction_deterministic,
                default_parallelism: config.meta.default_parallelism,
                vacuum_interval_sec: config.meta.vacuum_interval_sec,
                vacuum_spin_interval_ms: config.meta.vacuum_spin_interval_ms,
                hummock_version_checkpoint_interval_sec: config
                    .meta
                    .hummock_version_checkpoint_interval_sec,
                enable_hummock_data_archive: config.meta.enable_hummock_data_archive,
                min_delta_log_num_for_hummock_version_checkpoint: config
                    .meta
                    .min_delta_log_num_for_hummock_version_checkpoint,
                min_sst_retention_time_sec: config.meta.min_sst_retention_time_sec,
                full_gc_interval_sec: config.meta.full_gc_interval_sec,
                collect_gc_watermark_spin_interval_sec: config
                    .meta
                    .collect_gc_watermark_spin_interval_sec,
                enable_committed_sst_sanity_check: config.meta.enable_committed_sst_sanity_check,
                periodic_compaction_interval_sec: config.meta.periodic_compaction_interval_sec,
                node_num_monitor_interval_sec: config.meta.node_num_monitor_interval_sec,
                prometheus_endpoint: opts.prometheus_endpoint,
                prometheus_selector: opts.prometheus_selector,
                vpc_id: opts.vpc_id,
                security_group_id: opts.security_group_id,
                privatelink_endpoint_default_tags,
                periodic_space_reclaim_compaction_interval_sec: config
                    .meta
                    .periodic_space_reclaim_compaction_interval_sec,
                telemetry_enabled: config.server.telemetry_enabled,
                periodic_ttl_reclaim_compaction_interval_sec: config
                    .meta
                    .periodic_ttl_reclaim_compaction_interval_sec,
                periodic_tombstone_reclaim_compaction_interval_sec: config
                    .meta
                    .periodic_tombstone_reclaim_compaction_interval_sec,
                periodic_split_compact_group_interval_sec: config
                    .meta
                    .periodic_split_compact_group_interval_sec,
                split_group_size_limit: config.meta.split_group_size_limit,
                min_table_split_size: config.meta.move_table_size_limit,
                table_write_throughput_threshold: config.meta.table_write_throughput_threshold,
                min_table_split_write_throughput: config.meta.min_table_split_write_throughput,
                partition_vnode_count: config.meta.partition_vnode_count,
                compact_task_table_size_partition_threshold_low: config
                    .meta
                    .compact_task_table_size_partition_threshold_low,
                compact_task_table_size_partition_threshold_high: config
                    .meta
                    .compact_task_table_size_partition_threshold_high,
                do_not_config_object_storage_lifecycle: config
                    .meta
                    .do_not_config_object_storage_lifecycle,
                compaction_task_max_heartbeat_interval_secs: config
                    .meta
                    .compaction_task_max_heartbeat_interval_secs,
                compaction_task_max_progress_interval_secs,
                compaction_config: Some(config.meta.compaction_config),
                cut_table_size_limit: config.meta.cut_table_size_limit,
                hybrid_partition_node_count: config.meta.hybrid_partition_vnode_count,
                event_log_enabled: config.meta.event_log_enabled,
                event_log_channel_max_size: config.meta.event_log_channel_max_size,
                advertise_addr: opts.advertise_addr,
                cached_traces_num: config.meta.developer.cached_traces_num,
                cached_traces_memory_limit_bytes: config
                    .meta
                    .developer
                    .cached_traces_memory_limit_bytes,
                enable_trivial_move: config.meta.developer.enable_trivial_move,
                enable_check_task_level_overlap: config
                    .meta
                    .developer
                    .enable_check_task_level_overlap,
                enable_dropped_column_reclaim: config.meta.enable_dropped_column_reclaim,
                object_store_config: config.storage.object_store,
                max_trivial_move_task_count_per_loop: config
                    .meta
                    .developer
                    .max_trivial_move_task_count_per_loop,
                max_get_task_probe_times: config.meta.developer.max_get_task_probe_times,
                secret_store_private_key: config.meta.secret_store_private_key,
                table_info_statistic_history_times: config
                    .storage
                    .table_info_statistic_history_times,
            },
            config.system.into_init_system_params(),
            Default::default(),
            shutdown,
        )
        .await
        .unwrap();
    })
}

fn validate_config(config: &RwConfig) {
    if config.meta.meta_leader_lease_secs <= 2 {
        let error_msg = "meta leader lease secs should be larger than 2";
        tracing::error!(error_msg);
        panic!("{}", error_msg);
    }
}
