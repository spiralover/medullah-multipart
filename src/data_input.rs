use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Default, Clone)]
pub struct DataInput {
    pub name: String,
    pub value: String,
    pub content_disposition_vars: HashMap<String, String>,
}

impl DataInput {
    pub fn get<T: FromStr>(&self) -> Result<T, T::Err> {
        self.value.parse::<T>()
    }
}
