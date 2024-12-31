use std::collections::HashMap;

use crate::content_disposition::ContentDisposition;
use crate::result::{MultipartError, MultipartResult};
use ntex::http::HeaderMap;
use ntex::util::Bytes;

#[derive(Debug, Default, Clone)]
pub struct FileInput {
    pub file_name: String,
    pub field_name: String,
    pub size: usize,
    pub content_type: String,
    pub bytes: Vec<Bytes>,
    pub extension: Option<String>,
    pub content_disposition_vars: HashMap<String, String>,
}

impl FileInput {
    pub fn create(headers: &HeaderMap) -> MultipartResult<Self> {
        let content_type = Self::get_content_type(headers)?;
        let content_disposition = Self::get_content_disposition(headers)?;

        let variables = ContentDisposition::parse(&content_disposition).variables;
        if !variables.contains_key("name") || !variables.contains_key("filename") {
            return Err(MultipartError::InvalidContentDisposition(
                content_disposition.to_string(),
            ));
        }

        let field = variables.get("name").cloned().unwrap();
        let name = variables.get("filename").cloned().unwrap();

        let binding = name.clone();
        let split_name: Vec<&str> = binding.split('.').collect();

        Ok(Self {
            content_type,
            size: 0,
            bytes: vec![],
            file_name: name,
            field_name: field,
            extension: split_name.last().map(|e| e.to_string()),
            content_disposition_vars: variables,
        })
    }

    fn get_content_type(headers: &HeaderMap) -> MultipartResult<String> {
        match headers.get("content-type") {
            None => Err(MultipartError::NoContentType(
                "Empty content type".to_string(),
            )),
            Some(header) => header
                .to_str()
                .map(|v| v.to_string())
                .map_err(|err| MultipartError::NoContentType(err.to_string())),
        }
    }

    fn get_content_disposition(headers: &HeaderMap) -> MultipartResult<String> {
        match headers.get("content-disposition") {
            None => Err(MultipartError::InvalidContentDisposition(
                "Empty content disposition".to_string(),
            )),
            Some(header) => header
                .to_str()
                .map(|v| v.to_string())
                .map_err(|err| MultipartError::InvalidContentDisposition(err.to_string())),
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
        headers.insert(CONTENT_TYPE, "image/jpeg".parse().unwrap());
        headers.insert(
            CONTENT_DISPOSITION,
            "form-data; name=\"image\"; filename=\"image.jpg\""
                .parse()
                .unwrap(),
        );

        let file_info = FileInput::create(&headers).unwrap();
        assert_eq!(file_info.file_name, "image.jpg");
        assert_eq!(file_info.field_name, "image");
        assert_eq!(file_info.content_type, "image/jpeg");
    }

    #[tokio::test]
    async fn test_create_from_missing_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "image/jpeg".parse().unwrap());

        assert!(matches!(
            FileInput::create(&headers),
            Err(MultipartError::InvalidContentDisposition(_))
        ));
    }

    #[tokio::test]
    async fn test_create_from_invalid_content_disposition() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "image/jpeg".parse().unwrap());
        headers.insert(CONTENT_DISPOSITION, "invalid".parse().unwrap());

        assert!(matches!(
            FileInput::create(&headers),
            Err(MultipartError::InvalidContentDisposition(_))
        ));
    }

    #[tokio::test]
    async fn test_parse_content_disposition() {
        let content_disposition = "form-data; name=\"image\"; filename=\"image.jpg\"";
        let variables = ContentDisposition::parse(content_disposition).variables;

        assert_eq!(variables.get("name"), Some(&"image".to_string()));
        assert_eq!(variables.get("filename"), Some(&"image.jpg".to_string()));
    }
}
