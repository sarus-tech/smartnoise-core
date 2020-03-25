use crate::errors::*;


use std::collections::HashMap;

use crate::{proto, base};
use crate::hashmap;
use crate::components::{Component, Expandable, Report};
use crate::utilities::{prepend, broadcast_privacy_usage, get_ith_release};

use crate::base::{NodeProperties, Value, ValueProperties, ArrayND};
use crate::utilities::json::{JSONRelease, AlgorithmInfo, privacy_usage_to_json, value_to_json};


impl Component for proto::DpVariance {
    // modify min, max, n, categories, is_public, non-null, etc. based on the arguments and component
    fn propagate_property(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        _public_arguments: &HashMap<String, Value>,
        _properties: &base::NodeProperties,
    ) -> Result<ValueProperties> {
        Err("DPVariance is abstract, and has no property propagation".into())
    }

    fn get_names(
        &self,
        _properties: &NodeProperties,
    ) -> Result<Vec<String>> {
        Err("get_names not implemented".into())
    }
}

impl Expandable for proto::DpVariance {
    fn expand_component(
        &self,
        _privacy_definition: &proto::PrivacyDefinition,
        component: &proto::Component,
        _properties: &base::NodeProperties,
        component_id: &u32,
        maximum_id: &u32,
    ) -> Result<proto::ComponentExpansion> {
        let mut current_id = maximum_id.clone();
        let mut computation_graph: HashMap<u32, proto::Component> = HashMap::new();

        // variance
        current_id += 1;
        let id_variance = current_id.clone();
        computation_graph.insert(id_variance, proto::Component {
            arguments: hashmap!["data".to_owned() => *component.arguments.get("data").ok_or::<Error>("data must be provided as an argument".into())?],
            variant: Some(proto::component::Variant::from(proto::Variance {
                finite_sample_correction: self.finite_sample_correction
            })),
            omit: true,
            batch: component.batch,
        });

        // noising
        computation_graph.insert(component_id.clone(), proto::Component {
            arguments: hashmap!["data".to_owned() => id_variance],
            variant: Some(proto::component::Variant::from(proto::LaplaceMechanism {
                privacy_usage: self.privacy_usage.clone()
            })),
            omit: false,
            batch: component.batch,
        });

        Ok(proto::ComponentExpansion {
            computation_graph,
            properties: HashMap::new(),
            releases: HashMap::new(),
            traversal: vec![id_variance]
        })
    }
}

impl Report for proto::DpVariance {
    fn summarize(
        &self,
        node_id: &u32,
        component: &proto::Component,
        _public_arguments: &HashMap<String, Value>,
        properties: &NodeProperties,
        release: &Value,
    ) -> Result<Option<Vec<JSONRelease>>> {
        let data_property = properties.get("data")
            .ok_or("data: missing")?.get_arraynd()
            .map_err(prepend("data:"))?.clone();

        let mut releases = Vec::new();

        let minimums = data_property.get_min_f64()?;
        let maximums = data_property.get_max_f64()?;
        let num_records = data_property.get_num_records()?;

        let num_columns = data_property.get_num_columns()?;
        let privacy_usages = broadcast_privacy_usage(&self.privacy_usage, num_columns as usize)?;

        for column_number in 0..num_columns {
            releases.push(JSONRelease {
                description: "DP release information".to_string(),
                statistic: "DPVariance".to_string(),
                variables: serde_json::json!(Vec::<String>::new()),
                release_info: match release.get_arraynd()? {
                    ArrayND::F64(v) => value_to_json(&get_ith_release(v, &(column_number as usize))?.into())?,
                    ArrayND::I64(v) => value_to_json(&get_ith_release(v, &(column_number as usize))?.into())?,
                    _ => return Err("maximum must be numeric".into())
                },
                privacy_loss: privacy_usage_to_json(&privacy_usages[column_number as usize].clone()),
                accuracy: None,
                batch: component.batch as u64,
                node_id: node_id.clone() as u64,
                postprocess: false,
                algorithm_info: AlgorithmInfo {
                    name: "".to_string(),
                    cite: "".to_string(),
                    mechanism: self.implementation.clone(),
                    argument: serde_json::json!({
                            "n": num_records,
                            "constraint": {
                                "lowerbound": minimums[column_number as usize],
                                "upperbound": maximums[column_number as usize]
                            }
                        }),
                },
            });
        }
        Ok(Some(releases))
    }
}
