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

use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use yaml_rust::yaml::{self, Hash};
use yaml_rust::Yaml;

/// Expands `inherit: xxx` to reuse steps between profiles.
pub struct InheritExpander<'a> {
    all_profile_section: &'a Hash,
}

impl InheritExpander<'_> {
    pub fn new(all_profile_section: &Hash) -> Result<Self> {
        Ok(Self {
            all_profile_section,
        })
    }

    /// Overrides values in `default` with values from `provided`.
    fn merge(use_id: &str, default: &yaml::Hash, provided: &yaml::Hash) -> yaml::Hash {
        let mut result = yaml::Hash::new();
        // put `use` as the first element to make the generated yaml more readable.
        result.insert(Yaml::String("use".into()), Yaml::String(use_id.into()));
        result.extend(default.clone());
        for (k, new_v) in provided {
            match result.get_mut(k) {
                Some(v) => {
                    // update the value, but do not change the order.
                    *v = new_v.clone()
                }
                None => {
                    // For keys not defined in the template (optional keys), we just append them
                    // here. It may be rejected later when deserializing to
                    // specific `ServiceConfig` if it's invalid.
                    result.insert(k.clone(), new_v.clone());
                }
            };
        }
        result
    }

    pub fn visit(&mut self, inherit: &str, yaml: Yaml) -> Result<Yaml> {
        let inherited_profile_section = self
            .all_profile_section
            .get(&Yaml::String(inherit.to_owned()))
            .with_context(|| format!("inherited profile '{}' not found", inherit))?
            .as_hash()
            .context("expect `profile` to be a hashmap")?;
        let inherited_steps = inherited_profile_section
            .get(&Yaml::String("steps".to_string()))
            .context("expect `steps` section in inherited profile")?
            .as_vec()
            .context("expect `steps` to be an array")?;

        let yaml = yaml
            .as_vec()
            .ok_or_else(|| anyhow!("expect an array for use and override"))?;

        for item in yaml.iter() {
            let map = item
                .as_hash()
                .ok_or_else(|| anyhow!("expect a hashmap for use"))?;

            let Some(override_id_yaml) = map.get(&Yaml::String("override".into())) else {
                result.insert(k, v)
            };

            let use_id_yaml = map
                .get(&Yaml::String("use".into()))
                .ok_or_else(|| anyhow!("expect `use` in hashmap"))?;
            let use_id = use_id_yaml
                .as_str()
                .ok_or_else(|| anyhow!("expect `use` to be a string"))?;
            let use_data = self
                .template
                .get(use_id)
                .ok_or_else(|| anyhow!("use source {} not found", use_id))?;

            if map.get(&Yaml::String("config-path".into())).is_some() {
                return Err(anyhow!(
                    "`config-path` should not be put inside a `use` step. \
                            Put `config-path` as a property parallel to `steps` instead."
                ));
            }
        }

        let array = yaml.iter().map(|item| {
            let map = item
                .as_hash()
                .ok_or_else(|| anyhow!("expect a hashmap for use"))?;

            let Some(use_id_yaml) = map.get(&Yaml::String("use".into())) else {
                continue;
            };

            let use_id_yaml = map
                .get(&Yaml::String("use".into()))
                .ok_or_else(|| anyhow!("expect `use` in hashmap"))?;
            let use_id = use_id_yaml
                .as_str()
                .ok_or_else(|| anyhow!("expect `use` to be a string"))?;
            let use_data = self
                .template
                .get(use_id)
                .ok_or_else(|| anyhow!("use source {} not found", use_id))?;

            if map.get(&Yaml::String("config-path".into())).is_some() {
                return Err(anyhow!(
                    "`config-path` should not be put inside a `use` step. \
                            Put `config-path` as a property parallel to `steps` instead."
                ));
            }

            Ok::<_, anyhow::Error>(Yaml::Hash(Self::merge(use_id, use_data, map)))
        });
        Ok(Yaml::Array(array.try_collect()?))
    }
}

#[cfg(test)]
mod tests {
    use yaml_rust::YamlLoader;

    use super::*;
    #[test]
    fn test_expand_use() {
        let template = YamlLoader::load_from_str(
            "
test:
  a: 2333
  b: 23333
test2:
  a: 23333
  b: 233333
      ",
        )
        .unwrap()
        .remove(0);

        let use_expand = YamlLoader::load_from_str(
            "
- use: test
  a: 23333
  c: 23333
- use: test2
  d: 23333",
        )
        .unwrap()
        .remove(0);

        let expected_result = YamlLoader::load_from_str(
            "
- use: test
  a: 23333
  b: 23333
  c: 23333
- use: test2
  a: 23333
  b: 233333
  d: 23333",
        )
        .unwrap()
        .remove(0);

        let mut visitor = InheritExpander::new(&template).unwrap();

        assert_eq!(visitor.visit(use_expand).unwrap(), expected_result);
    }
}
