use std::fmt::{Display, Formatter};
use std::io::Error;

pub type MultipartResult<T> = Result<T, MultipartError>;

#[derive(Debug)]
pub enum MultipartError {
    IoError(Error),
    NoFile,
    NoContentType(String),
    InvalidContentDisposition(String),
    NtexError(ntex_multipart::MultipartError),
    ValidationError(MultipartValidationError),
}

#[derive(Debug)]
pub enum MultipartValidationError {
    LowerSizeError(usize),
    UpperSizeError(usize),
    InvalidMimeType(String),
}

impl From<Error> for MultipartError {
    fn from(value: Error) -> Self {
        MultipartError::IoError(value)
    }
}

impl Display for MultipartError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::IoError(err) => {
                write!(f, "{}", err)
            }
            MultipartError::NoFile => {
                write!(f, "No file was uploaded")
            }
            MultipartError::NoContentType(ct) => {
                write!(f, "Invalid content type: {}", ct)
            }
            MultipartError::InvalidContentDisposition(err) => {
                write!(f, "Invalid content disposition: {}", err)
            }
            MultipartError::NtexError(err) => {
                write!(f, "{}", err)
            }
            MultipartError::ValidationError(err) => match err {
                MultipartValidationError::LowerSizeError(size) => {
                    write!(f, "File size is too small. Minimum size is {}", size)
                }
                MultipartValidationError::UpperSizeError(size) => {
                    write!(f, "File size is too big. Maximum size is {}", size)
                }
                MultipartValidationError::InvalidMimeType(mime) => {
                    write!(f, "Invalid mime type: {}", mime)
                }
            },
        }
    }
}
