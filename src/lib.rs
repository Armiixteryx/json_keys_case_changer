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
    json_in: &'a JsonMap,

    /// The case to convert.
    case: Case,

    /// Map of list of manual renames.
    manual_renames: RenameMap<'a>,

    /// Rename either by key or by value.
    rename_behavior: RenameBehavior,
}

impl<'a> CaseChanger<'a> {
    pub fn new(json_obj: &'a serde_json::Value, new_case: Case) -> Result<Self, ()> {
        let obj = json_obj.as_object().ok_or(())?;
        Ok(Self {
            json_in: obj,
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
        let mut json_out = JsonMap::new();

        CaseChanger::<'a>::internal_convert(
            self.json_in,
            &mut json_out,
            self.case,
            &self.manual_renames,
            self.rename_behavior,
        );

        Value::Object(json_out)
    }

    fn internal_convert(
        actual_json: &'a JsonMap,
        new_json: &mut JsonMap,
        case: Case,
        manual_renames: &RenameMap,
        rename_behavior: RenameBehavior,
    ) {
        for (key, value) in actual_json.iter() {
            match (key, value) {
                (key, Value::Object(elem)) => {
                    let mut inner_obj = Map::<String, Value>::new();

                    CaseChanger::<'a>::internal_convert(
                        elem,
                        &mut inner_obj,
                        case,
                        manual_renames,
                        rename_behavior,
                    );

                    let manual_case = CaseChanger::<'a>::determine_manual_case(
                        key,
                        &manual_renames,
                        rename_behavior,
                    );
                    match manual_case {
                        Some(k) => new_json.insert(k.to_owned(), Value::Object(inner_obj)),
                        None => new_json.insert(key.to_case(case), Value::Object(inner_obj)),
                    };
                }
                (key, Value::Array(elem)) => {
                    let mut inner_arr: Vec<Value> = Vec::new();

                    for i in elem.iter() {
                        i.as_object().and_then(|actual| {
                            let mut inner_obj = Map::<String, Value>::new();

                            CaseChanger::<'a>::internal_convert(
                                actual,
                                &mut inner_obj,
                                case,
                                &manual_renames,
                                rename_behavior,
                            );

                            inner_arr.push(Value::Object(inner_obj));
                            Some(())
                        });
                    }
                    let manual_case = CaseChanger::<'a>::determine_manual_case(
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
                    let manual_case = CaseChanger::<'a>::determine_manual_case(
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
                .and_then(|(_, key)| Some(key.to_owned())),
        }
    }
}
