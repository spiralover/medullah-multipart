use std::convert::Infallible;
use std::path::Path;

use crate::content_disposition::ContentDisposition;
use crate::data_input::DataInput;
use crate::file_input::FileInput;
use crate::result::{MultipartError, MultipartResult};
use futures::StreamExt;
use ntex::http::Payload;
use ntex::util::{Bytes, HashMap};
use ntex::web::{FromRequest, HttpRequest};
use ntex_multipart::Multipart as NtexMultipart;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Multipart {
    multipart: NtexMultipart,
    file_inputs: HashMap<String, FileInput>, // Store all files
    data_inputs: HashMap<String, DataInput>, // Store all non-file form fields
}

impl<Err> FromRequest<Err> for Multipart {
    type Error = Infallible;

    async fn from_request(
        req: &HttpRequest,
        payload: &mut Payload,
    ) -> Result<Multipart, Infallible> {
        let multipart = NtexMultipart::new(req.headers(), payload.take());
        Ok(Multipart::new(multipart).await)
    }
}

impl<'a> Multipart {
    pub async fn new(multipart: NtexMultipart) -> Multipart {
        Self {
            multipart,
            file_inputs: Default::default(),
            data_inputs: Default::default(),
        }
    }

    pub async fn process(&mut self) -> Result<&mut Multipart, MultipartError> {
        while let Some(item) = self.multipart.next().await {
            let mut field = match item {
                Ok(item) => item,
                Err(err) => return Err(MultipartError::NtexError(err)),
            };

            if let Some(content_disposition) = field.headers().get("content-disposition") {
                let content_disposition = content_disposition.to_str().ok();
                if let Some(content_disposition) = content_disposition {
                    let content_disposition = ContentDisposition::create(content_disposition);

                    if !content_disposition.has_name_field() {
                        continue;
                    }

                    // Process form fields (non-file fields)
                    if !content_disposition.is_file_field() {
                        let value = self.collect_data_field_value(&mut field).await;
                        let field_name =
                            content_disposition.get_variable("name").unwrap_or_default();

                        self.data_inputs.insert(
                            field_name.to_string(),
                            DataInput {
                                value,
                                name: field_name.to_string(),
                                content_disposition_vars: content_disposition
                                    .get_variables()
                                    .clone(),
                            },
                        );

                        continue;
                    }

                    // Process file fields
                    let mut info = FileInput::create(field.headers(), content_disposition)?;
                    let mut total_size = 0;
                    let mut bytes: Vec<Bytes> = vec![];
                    while let Some(chunk) = field.next().await {
                        let data = chunk.unwrap();
                        total_size += data.len();
                        bytes.push(data);
                    }

                    info.size = total_size;
                    info.bytes = bytes;
                    self.file_inputs.insert(info.field_name.clone(), info);
                }
            }
        }

        // If no items were processed, return Ok
        Ok(self)
    }

    async fn collect_data_field_value(&self, field: &mut ntex_multipart::Field) -> String {
        let mut value = String::new();
        while let Some(chunk) = field.next().await {
            if let Ok(chunk_data) = chunk {
                value.push_str(&String::from_utf8_lossy(&chunk_data));
            }
        }

        value
    }

    pub async fn save(&self, field: &str, path: impl AsRef<Path>) -> MultipartResult<()> {
        match self.file(field) {
            Some(f) => Self::save_file(f, path).await,
            None => Err(MultipartError::NoFile),
        }
    }

    pub async fn save_file(file_input: &FileInput, path: impl AsRef<Path>) -> MultipartResult<()> {
        let mut file = File::create(path).await?;

        for byte in &file_input.bytes {
            file.write_all(byte).await?;
        }

        file.flush().await?;

        Ok(())
    }

    pub fn all_data_inputs(&self) -> &HashMap<String, DataInput> {
        &self.data_inputs
    }

    pub fn data_input(&self, field: &str) -> Option<&DataInput> {
        self.data_inputs.get(field)
    }

    pub fn all_files(&self) -> &HashMap<String, FileInput> {
        &self.file_inputs
    }

    pub fn file(&self, field: &str) -> Option<&FileInput> {
        self.file_inputs.get(field)
    }
}

#[cfg(test)]
mod test {
    use crate::data_input::DataInput;
    use crate::file_input::FileInput;
    use crate::result::MultipartError;
    use crate::Multipart;
    use ntex::http::header::HeaderMap;
    use ntex::http::Payload;
    use ntex::util::Bytes;
    use ntex_multipart::Multipart as NtexMultipart;
    use std::collections::HashMap;
    use tokio::fs;

    #[tokio::test]
    async fn test_multipart_new() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);

        let multipart_instance = Multipart::new(multipart).await;

        assert!(multipart_instance.all_data_inputs().is_empty());
        assert!(multipart_instance.all_files().is_empty());
    }

    #[tokio::test]
    async fn test_save_file() {
        let file_input = FileInput {
            field_name: "file".to_string(),
            file_name: "test.txt".to_string(),
            content_type: "text/plain".to_string(),
            size: 11,
            bytes: vec![Bytes::from("Hello World")],
            extension: None,
            content_disposition: Default::default(),
        };

        let path = "test_output.txt";
        let result = Multipart::save_file(&file_input, &path).await;

        assert!(result.is_ok());

        let content = fs::read_to_string(path).await.unwrap();
        assert_eq!(content, "Hello World");

        fs::remove_file(path).await.unwrap(); // Cleanup
    }

    #[tokio::test]
    async fn test_save_nonexistent_file() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let multipart_instance = Multipart::new(multipart).await;

        let result = multipart_instance
            .save("nonexistent_file", &"output.txt")
            .await;
        assert!(matches!(result, Err(MultipartError::NoFile)));
    }

    #[tokio::test]
    async fn test_data_inputs_and_files() {
        let headers = HeaderMap::new();
        let payload = Payload::None;
        let multipart = NtexMultipart::new(&headers, payload);
        let mut multipart_instance = Multipart::new(multipart).await;

        multipart_instance.data_inputs.insert(
            "key1".to_string(),
            DataInput {
                name: "key1".to_string(),
                value: "value1".to_string(),
                content_disposition_vars: HashMap::new(),
            },
        );

        multipart_instance.file_inputs.insert(
            "file1".to_string(),
            FileInput {
                field_name: "file1".to_string(),
                file_name: "file.txt".to_string(),
                content_type: "text/plain".to_string(),
                size: 11,
                bytes: vec![Bytes::from("Hello World")],
                extension: None,
                content_disposition: Default::default(),
            },
        );

        assert_eq!(
            multipart_instance.data_input("key1").unwrap().value,
            "value1"
        );
        assert_eq!(multipart_instance.file("file1").unwrap().size, 11);
    }
}
