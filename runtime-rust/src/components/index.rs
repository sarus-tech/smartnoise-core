use yarrow_validator::errors::*;

use crate::base::NodeArguments;
use yarrow_validator::base::{Value, ArrayND};
use crate::components::Evaluable;
use yarrow_validator::proto;



impl Evaluable for proto::Index {
    fn evaluate(&self, arguments: &NodeArguments) -> Result<Value> {
        let data = arguments.get("data").unwrap();
        let columns = arguments.get("columns").unwrap();

        match data {
            Value::HashmapString(dataframe) => match columns {
                Value::ArrayND(array) => match array {
                    ArrayND::Str(column_names) => match column_names.ndim() {
                        0 => Ok(dataframe.get(column_names.first().unwrap()).unwrap().to_owned()),
//                1 => match column_names.into_dimensionality::<Ix1>() {
//                    Ok(column_names) =>
//                        Value::Str(stack(Axis(0), column_names.to_vec().iter()
//                            .map(|column_name| match dataframe.get(column_names.first().unwrap()).unwrap() {
//                                Value::Str(array) => array,
//                                _ => panic!("selected data frame columns are not of a homogenous type".to_string())
//                            }).collect()).unwrap())
//                            .collect::<Vec<ArrayD<str>>>(),
//                    _ => Err("column names must be at most 1-dimensional".to_owned()),
//                },
                        _ => Err("column names must be at most 1-dimensional".into())
                    },
                    _ => Err("column names must be strings".into())
                },
                _ => Err("column names must an array".into())
            },
            _ => Err("indexing is only implemented for hashmaps".into())
        }
    }
}