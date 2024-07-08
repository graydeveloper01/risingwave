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

use std::time::Duration;

use anyhow::Context as _;
use async_nats::jetstream::consumer;
use async_trait::async_trait;
use futures::StreamExt;
use futures_async_stream::try_stream;
use risingwave_common::bail;

use super::message::NatsMessage;
use super::{NatsOffset, NatsSplit};
use crate::error::ConnectorResult as Result;
use crate::parser::ParserConfig;
use crate::source::common::{into_chunk_stream, CommonSplitReader};
use crate::source::nats::NatsProperties;
use crate::source::{
    BoxChunkSourceStream, Column, SourceContextRef, SourceMessage, SplitId, SplitReader,
};

pub struct NatsSplitReader {
    consumer: consumer::Consumer<consumer::pull::Config>,
    properties: NatsProperties,
    parser_config: ParserConfig,
    source_ctx: SourceContextRef,
    start_position: NatsOffset,
    split_id: SplitId,
}

#[async_trait]
impl SplitReader for NatsSplitReader {
    type Properties = NatsProperties;
    type Split = NatsSplit;

    async fn new(
        properties: NatsProperties,
        splits: Vec<NatsSplit>,
        parser_config: ParserConfig,
        source_ctx: SourceContextRef,
        _columns: Option<Vec<Column>>,
    ) -> Result<Self> {
        // TODO: to simplify the logic, return 1 split for first version
        assert!(splits.len() == 1);
        let split = splits.into_iter().next().unwrap();
        let split_id = split.split_id;
        let start_position = match &split.start_sequence {
            NatsOffset::None => match &properties.scan_startup_mode {
                None => NatsOffset::Earliest,
                Some(mode) => match mode.as_str() {
                    "latest" => NatsOffset::Latest,
                    "earliest" => NatsOffset::Earliest,
                    "timestamp_millis" => {
                        if let Some(time) = &properties.start_time {
                            NatsOffset::Timestamp(time.parse().context(
                                "failed to parse the start time as nats offset timestamp",
                            )?)
                        } else {
                            bail!("scan_startup_timestamp_millis is required");
                        }
                    }
                    _ => {
                        bail!("invalid scan_startup_mode, accept earliest/latest/timestamp_millis")
                    }
                },
            },
            start_position => start_position.to_owned(),
        };

        let consumer = properties
            .common
            .build_consumer(
                properties.stream.clone(),
                split_id.to_string(),
                start_position.clone(),
                properties.durable_name.clone(),
                properties.description.clone(),
                properties.ack_policy.clone(),
                properties.ack_wait.clone().map(|s| {
                    Duration::from_secs(s.parse::<u64>().expect("failed to parse ack_wait to u64"))
                }),
                properties.max_deliver.clone().map(|s| {
                    s.parse::<i64>()
                        .expect("failed to parse max_deliver to i64")
                }),
                properties.filter_subject.clone(),
                properties
                    .filter_subjects
                    .clone()
                    .map(|s| s.split(',').map(|s| s.to_string()).collect()),
                properties.replay_policy.clone(),
                properties
                    .rate_limit
                    .clone()
                    .map(|s| s.parse::<u64>().expect("failed to parse rate_limit to u64")),
                properties.sample_frequency.clone().map(|s| {
                    s.parse::<u8>()
                        .expect("failed to parse sample_frequency to u8")
                }),
                properties.max_waiting.clone().map(|s| {
                    s.parse::<i64>()
                        .expect("failed to parse max_waiting to i64")
                }),
                properties.max_ack_pending.clone().map(|s| {
                    s.parse::<i64>()
                        .expect("failed to parse max_ack_pending to i64")
                }),
                properties.idle_heartbeat.clone().map(|s| {
                    Duration::from_secs(
                        s.parse::<u64>()
                            .expect("failed to parse idle_heartbeat to u64"),
                    )
                }),
                properties
                    .max_batch
                    .clone()
                    .map(|s| s.parse::<i64>().expect("failed to parse max_batch to i64")),
                properties
                    .max_bytes
                    .clone()
                    .map(|s| s.parse::<i64>().expect("failed to parse max_bytes to i64")),
                properties.max_expires.clone().map(|s| {
                    Duration::from_secs(s.parse::<u64>().expect("failed to parse ack_wait to u64"))
                }),
                properties.inactive_threshold.clone().map(|s| {
                    Duration::from_secs(
                        s.parse::<u64>()
                            .expect("failed to parse inactive_threshold to u64"),
                    )
                }),
                properties.num_replicas.clone().map(|s| {
                    s.parse::<usize>()
                        .expect("failed to parse num_replicas to usize")
                }),
                properties.memory_storage.clone().map(|s| {
                    s.parse::<bool>()
                        .expect("failed to parse memory_storage to bool")
                }),
                properties.backoff.clone().map(|s| {
                    s.split(',')
                        .map(|s| {
                            Duration::from_secs(
                                s.parse::<u64>().expect("failed to parse backoff to u64"),
                            )
                        })
                        .collect()
                }),
            )
            .await?;

        Ok(Self {
            consumer,
            properties,
            parser_config,
            source_ctx,
            start_position,
            split_id,
        })
    }

    fn into_stream(self) -> BoxChunkSourceStream {
        let parser_config = self.parser_config.clone();
        let source_context = self.source_ctx.clone();
        into_chunk_stream(self, parser_config, source_context)
    }
}

impl CommonSplitReader for NatsSplitReader {
    #[try_stream(ok = Vec<SourceMessage>, error = crate::error::ConnectorError)]
    async fn into_data_stream(self) {
        let capacity = self.source_ctx.source_ctrl_opts.chunk_size;
        let messages = self.consumer.messages().await?;
        #[for_await]
        for msgs in messages.ready_chunks(capacity) {
            let mut msg_vec = Vec::with_capacity(capacity);
            for msg in msgs {
                msg_vec.push(SourceMessage::from(NatsMessage::new(
                    self.split_id.clone(),
                    msg?,
                )));
            }
            yield msg_vec;
        }
    }
}
