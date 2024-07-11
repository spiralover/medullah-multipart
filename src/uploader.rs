use std::convert::Infallible;
use std::path::Path;

use futures::StreamExt;
use ntex::http::Payload;
use ntex::util::Bytes;
use ntex::web::{FromRequest, HttpRequest};
use ntex_multipart::Multipart as NtexMultipart;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::file::FileInfo;
use crate::result::{MultipartError, MultipartResult};
use crate::result::MultipartError::{NotUploaded, ValidationError};
use crate::result::MultipartValidationError::{InvalidMimeType, LowerSizeError, UpperSizeError};

pub struct Uploader {
    multipart: NtexMultipart,
    bytes: Vec<Bytes>,
    file: FileInfo,
}

pub struct UploadData<'a> {
    pub field: &'a str,
    pub lower_size: usize,
    pub upper_size: Option<usize>,
    pub allowed_mimes: Vec<&'a str>,
}

impl<Err> FromRequest<Err> for Uploader {
    type Error = Infallible;

    async fn from_request(
        req: &HttpRequest,
        payload: &mut Payload,
    ) -> Result<Uploader, Infallible> {
        let multipart = NtexMultipart::new(req.headers(), payload.take());
        Ok(Uploader::new(multipart).await)
    }
}

impl<'a> Uploader {
    pub async fn new(multipart: NtexMultipart) -> Uploader {
        Self { multipart, bytes: vec![], file: FileInfo::default() }
    }

    pub async fn capture(&mut self, field: &str) -> Result<&mut Uploader, MultipartError> {
        self.capture_advance(UploadData {
            field,
            lower_size: 0,
            upper_size: None,
            allowed_mimes: vec![],
        }).await
    }

    pub async fn capture_advance(&mut self, ud: UploadData<'a>) -> Result<&mut Uploader, MultipartError> {
        while let Some(item) = self.multipart.next().await {
            let mut field = match item {
                Ok(item) => item,
                Err(err) => return Err(MultipartError::NtexError(err)),
            };

            let mut info = FileInfo::create(field.headers())?;
            if info.field == ud.field {
                if ud.allowed_mimes.contains(&&*info.content_type) {
                    return Err(ValidationError(InvalidMimeType));
                }

                let mut total_size = 0;
                let mut bytes: Vec<Bytes> = vec![];
                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    total_size += data.len();

                    if ud.upper_size.is_some() && total_size > ud.upper_size.unwrap() {
                        return Err(ValidationError(UpperSizeError));
                    }

                    bytes.push(data);
                }

                if total_size < ud.lower_size {
                    return Err(ValidationError(LowerSizeError));
                }

                info.size = total_size;
                self.bytes = bytes;
                self.file = info;

                return Ok(self);
            }
        }

        Err(NotUploaded)
    }

    pub async fn save<P: AsRef<Path>>(&self, path: &P) -> MultipartResult<()> {
        let mut file = File::create(path).await?;

        for byte in &self.bytes {
            file.write_all(byte).await?;
        }

        file.flush().await?;
        Ok(())
    }

    pub fn file(&self) -> &FileInfo {
        &self.file
    }
}

#[cfg(test)]
mod tests {
    use ntex::http::HeaderMap;

    use crate::file::FileInfo;

    #[tokio::test]
    async fn test_file_info_create() {
        let headers = generate_headers("attachment", "test.png", "image/png");
        let result = FileInfo::create(&headers);

        assert!(result.is_ok());
        let file_info = result.unwrap();
        assert_eq!(file_info.field, "attachment");
        assert_eq!(file_info.name, "test.png");
        assert_eq!(file_info.content_type, "image/png");
    }

    fn generate_headers(field: &str, filename: &str, content_type: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("content-disposition".parse().unwrap(), format!("form-data; app=\"naira\"; name=\"{}\"; filename=\"{}\"", field, filename).parse().unwrap());
        headers.insert("content-type".parse().unwrap(), content_type.parse().unwrap());
        headers
    }
}
