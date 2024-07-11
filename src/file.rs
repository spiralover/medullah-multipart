use std::collections::HashMap;

use ntex::http::HeaderMap;

use crate::result::{MultipartError, MultipartResult};

#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    pub name: String,
    pub field: String,
    pub size: usize,
    pub content_type: String,
    pub extension: Option<String>,
    pub content_disposition_vars: HashMap<String, String>,
}

impl FileInfo {
    pub fn create(headers: &HeaderMap) -> MultipartResult<Self> {
        let content_type = Self::get_content_type(headers)?;
        let content_disposition = Self::get_content_disposition(headers)?;

        let variables = Self::parse_content_disposition(&content_disposition);
        if !variables.contains_key("name") || !variables.contains_key("filename") {
            return Err(MultipartError::InvalidContentDisposition);
        }

        let field = variables.get("name").cloned().unwrap();
        let name = variables.get("filename").cloned().unwrap();

        let binding = name.clone();
        let split_name: Vec<&str> = binding.split('.').collect();

        Ok(Self {
            name,
            field,
            content_type,
            size: 0,
            extension: split_name.last().map(|e| e.to_string()),
            content_disposition_vars: variables,
        })
    }
    fn parse_content_disposition(content_disposition: &str) -> HashMap<String, String> {
        let mut variables = HashMap::new();

        for part in content_disposition.split(';') {
            let part = part.trim();
            if let Some((key, value)) = part.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                variables.insert(key, value);
            }
        }

        variables
    }

    fn get_content_type(headers: &HeaderMap) -> MultipartResult<String> {
        match headers.get("content-type") {
            None => Err(MultipartError::InvalidContentType),
            Some(header) => header.to_str()
                .map(|v| v.to_string())
                .map_err(|_| MultipartError::InvalidContentType)
        }
    }

    fn get_content_disposition(headers: &HeaderMap) -> MultipartResult<String> {
        match headers.get("content-disposition") {
            None => Err(MultipartError::InvalidContentDisposition),
            Some(header) => header.to_str()
                .map(|v| v.to_string())
                .map_err(|_| MultipartError::InvalidContentDisposition)
        }
    }
}

#[cfg(test)]
mod tests {
    use ntex::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};

    use super::*;

    #[tokio::test]
    async fn test_create_from_valid_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "image/jpeg".parse().unwrap(),
        );
        headers.insert(
            CONTENT_DISPOSITION,
            "form-data; name=\"image\"; filename=\"image.jpg\"".parse().unwrap(),
        );

        let file_info = FileInfo::create(&headers).unwrap();
        assert_eq!(file_info.name, "image.jpg");
        assert_eq!(file_info.field, "image");
        assert_eq!(file_info.content_type, "image/jpeg");
    }

    #[tokio::test]
    async fn test_create_from_missing_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "image/jpeg".parse().unwrap());

        assert!(matches!(
            FileInfo::create(&headers),
            Err(MultipartError::InvalidContentDisposition)
        ));
    }

    #[tokio::test]
    async fn test_create_from_invalid_content_disposition() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            "image/jpeg".parse().unwrap(),
        );
        headers.insert(
            CONTENT_DISPOSITION,
            "invalid".parse().unwrap(),
        );

        assert!(matches!(
            FileInfo::create(&headers),
            Err(MultipartError::InvalidContentDisposition)
        ));
    }

    #[tokio::test]
    async fn test_parse_content_disposition() {
        let content_disposition = "form-data; name=\"image\"; filename=\"image.jpg\"";
        let variables = FileInfo::parse_content_disposition(content_disposition);

        assert_eq!(variables.get("name"), Some(&"image".to_string()));
        assert_eq!(variables.get("filename"), Some(&"image.jpg".to_string()));
    }
}
