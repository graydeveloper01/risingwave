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

use super::docker_service::{DockerService, DockerServiceConfig};
use crate::SchemaRegistryConfig;

impl DockerServiceConfig for SchemaRegistryConfig {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn is_user_managed(&self) -> bool {
        self.user_managed
    }

    fn image(&self) -> String {
        self.image.clone()
    }

    fn envs(&self) -> Vec<(String, String)> {
        // https://docs.confluent.io/platform/current/installation/docker/config-reference.html#sr-long-configuration
        // https://docs.confluent.io/platform/current/schema-registry/installation/config.html
        let kafka = self
            .provide_kafka
            .as_ref()
            .expect("Kafka is required for Schema Registry");
        if kafka.len() != 1 {
            panic!("More than one Kafka is not supported yet");
        }
        let kafka = &kafka[0];
        vec![
            (
                "SCHEMA_REGISTRY_HOST_NAME".to_owned(),
                "localhost".to_owned(),
            ),
            (
                "SCHEMA_REGISTRY_LISTENERS".to_owned(),
                format!("http://0.0.0.0:{}", self.port),
            ),
            (
                "SCHEMA_REGISTRY_KAFKASTORE_BOOTSTRAP_SERVERS".to_owned(),
                format!("PLAINTEXT://localhost:{}", kafka.port),
            ),
            ("SCHEMA_REGISTRY_DEBUG".to_owned(), "true".to_owned()),
        ]
    }

    fn ports(&self) -> Vec<(String, String)> {
        vec![]
    }

    fn data_path(&self) -> Option<String> {
        None
    }

    fn use_host_network(&self) -> bool {
        true
    }
}

/// Docker-backed Schema Registry service.
pub type SchemaRegistryService = DockerService<SchemaRegistryConfig>;
