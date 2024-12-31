use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct DataInput {
    pub name: String,
    pub value: String,
    pub content_type: String,
    pub content_disposition_vars: HashMap<String, String>,
}