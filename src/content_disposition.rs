use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct ContentDisposition {
    variables: HashMap<String, String>,
    is_file_field: bool,
    has_name_field: bool,
}

pub struct ContentDispositionParseResult {
    pub variables: HashMap<String, String>,
    pub is_file_field: bool,
    pub has_name_field: bool,
}

#[allow(dead_code)]
impl ContentDisposition {
    /// Constructs a `ContentDisposition` from a `content_disposition` string.
    pub fn create(content_disposition: &str) -> Self {
        let result = Self::parse(content_disposition);
        Self {
            variables: result.variables,
            is_file_field: result.is_file_field,
            has_name_field: result.has_name_field,
        }
    }

    /// Retrieves a reference to the value associated with the given key, if it exists.
    pub fn get_variable(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(|v| v.as_str())
    }

    /// Returns a reference to the entire map of variables.
    pub fn get_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Checks if the content disposition represents a file field.
    pub fn is_file_field(&self) -> bool {
        self.is_file_field
    }

    /// Checks if the content disposition contains a "name" field.
    pub fn has_name_field(&self) -> bool {
        self.has_name_field
    }

    pub fn get_name(&self) -> Option<&str> {
        self.get_variable("name")
    }

    pub fn get_filename(&self) -> Option<&str> {
        self.get_variable("filename")
    }

    /// Parses a content disposition string into a HashMap of variables.
    pub fn parse(content_disposition: &str) -> ContentDispositionParseResult {
        let mut variables = HashMap::new();

        for part in content_disposition.split(';') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                // Trim whitespace and remove any surrounding quotes from the value
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                variables.insert(key, value);
            }
        }

        ContentDispositionParseResult {
            is_file_field: variables.contains_key("filename"),
            has_name_field: variables.contains_key("name"),
            variables,
        }
    }
}
