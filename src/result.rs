use std::io::Error;

pub type MultipartResult<T> = Result<T, MultipartError>;

#[derive(Debug)]
pub enum MultipartError {
    IoError(Error),
    NotUploaded,
    InvalidContentType,
    InvalidContentDisposition,
    NtexError(ntex_multipart::MultipartError),
    ValidationError(MultipartValidationError),
}

#[derive(Debug)]
pub enum MultipartValidationError {
    LowerSizeError,
    UpperSizeError,
    InvalidMimeType,
}

impl From<Error> for MultipartError {
    fn from(value: Error) -> Self {
        MultipartError::IoError(value)
    }
}
