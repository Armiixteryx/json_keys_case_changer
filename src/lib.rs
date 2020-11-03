use convert_case::{Case, Casing};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub type JsonMap = Map<String, Value>;
pub type RenameMap<'a> = HashMap<&'a str, &'a str>;

#[derive(Copy, Clone)]
pub enum RenameBehavior {
    ByKey,
    ByValue,
}

impl Default for RenameBehavior {
    fn default() -> Self {
        Self::ByKey
    }
}

pub struct CaseChanger<'a> {
    /// The input JSON.
    json_in: Value,

    /// The case to convert.
    case: Case,

    /// Map of list of manual renames.
    manual_renames: RenameMap<'a>,

    /// Rename either by key or by value.
    rename_behavior: RenameBehavior,
}

impl<'a> CaseChanger<'a> {
    pub fn new(json_obj: serde_json::Value, new_case: Case) -> Result<Self, ()> {
        Ok(Self {
            json_in: json_obj,
            case: new_case,
            manual_renames: RenameMap::default(),
            rename_behavior: RenameBehavior::default(),
        })
    }

    pub fn with_manual_renames(&mut self, rename_list: RenameMap<'a>) {
        self.manual_renames = rename_list;
    }

    pub fn with_custom_rename_behavior(&mut self, rename_behavior: RenameBehavior) {
        self.rename_behavior = rename_behavior;
    }

    pub fn convert(&mut self) -> Value {
        let json_out = CaseChanger::internal_convert(
            self.json_in.clone(),
            self.case,
            &self.manual_renames,
            self.rename_behavior,
        );

        json_out
    }

    fn internal_convert(
        actual_json: Value,
        case: Case,
        manual_renames: &RenameMap,
        rename_behavior: RenameBehavior,
    ) -> Value {
        match actual_json {
            Value::Array(arr) => {
                let mut deep_arr: Vec<Value> = Vec::new();

                for deep_value in arr {
                    deep_arr.push(CaseChanger::internal_convert(
                        deep_value,
                        case,
                        manual_renames,
                        rename_behavior,
                    ));
                }

                Value::Array(deep_arr)
            }
            Value::Object(actual_json) => {
                let mut new_json = JsonMap::new();
                for (key, value) in actual_json.iter() {
                    match (key, value) {
                        (key, Value::Object(elem)) => {
                            let inner_obj = CaseChanger::internal_convert(
                                Value::Object(elem.clone()),
                                case,
                                manual_renames,
                                rename_behavior,
                            );

                            let manual_case = CaseChanger::determine_manual_case(
                                key,
                                &manual_renames,
                                rename_behavior,
                            );
                            match manual_case {
                                Some(k) => new_json.insert(k.to_owned(), inner_obj),
                                None => new_json.insert(key.to_case(case), inner_obj),
                            };
                        }
                        (key, Value::Array(elem)) => {
                            let mut inner_arr: Vec<Value> = Vec::new();

                            for obj in elem.iter() {
                                let inner_obj = CaseChanger::internal_convert(
                                    obj.clone(),
                                    case,
                                    manual_renames,
                                    rename_behavior,
                                );

                                inner_arr.push(inner_obj);
                            }
                            let manual_case = CaseChanger::determine_manual_case(
                                key,
                                &manual_renames,
                                rename_behavior,
                            );
                            match manual_case {
                                Some(k) => new_json.insert(k.to_owned(), Value::Array(inner_arr)),
                                None => new_json.insert(key.to_case(case), Value::Array(inner_arr)),
                            };
                        }
                        (key, value) => {
                            let manual_case = CaseChanger::determine_manual_case(
                                key,
                                &manual_renames,
                                rename_behavior,
                            );
                            match manual_case {
                                Some(k) => new_json.insert(k.to_owned(), value.clone()),
                                None => new_json.insert(key.to_case(case), value.clone()),
                            };
                        }
                    }
                }

                Value::Object(new_json)
            }
            value => value.clone(),
        }
    }

    fn determine_manual_case<'b>(
        key: &'a str,
        manual_renames: &'b RenameMap,
        rename_behavior: RenameBehavior,
    ) -> Option<&'b str> {
        match rename_behavior {
            RenameBehavior::ByKey => manual_renames
                .get(key)
                .and_then(|found| Some(found.to_owned())),
            RenameBehavior::ByValue => manual_renames
                .iter()
                .find(|(_, rename_value)| **rename_value == key)
                .and_then(|(key, _)| Some(key.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::*;

    #[test]
    fn root_array() {
        let value = json!([{"myCamel": 1}, {"myCamel": 2}]);
        let expected = json!([{"my_camel": 1}, {"my_camel": 2}]);

        let case_changed = CaseChanger::new(value, Case::Snake).unwrap().convert();

        assert_eq!(expected, case_changed);
    }

    #[test]
    fn array_of_strings_in_map() {
        let value = json!({"anArray": ["ObjectOne", "ObjectTwo"]});

        // We expect to keep the array of strings (value of key) without
        // modifications.
        let expected = json!({"an_array": ["ObjectOne", "ObjectTwo"]});

        let case_changed = CaseChanger::new(value, Case::Snake).unwrap().convert();

        assert_eq!(expected, case_changed);
    }
}