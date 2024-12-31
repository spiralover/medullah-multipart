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
    pub content_disposition: ContentDisposition,
}

impl FileInput {
    pub fn create(headers: &HeaderMap, cd: ContentDisposition) -> MultipartResult<Self> {
        let content_type = Self::get_content_type(headers)?;

        let variables = cd.get_variables();
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
            content_disposition: cd,
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
}
