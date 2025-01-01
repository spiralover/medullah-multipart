use crate::result::MultipartResult;
use crate::{FileInput, MultipartError};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InputError {
    pub name: String,
    pub error: ErrorMessage,
}

#[derive(Debug, Clone)]
pub enum ErrorMessage {
    NoFiles,
    FileTooSmall(usize),
    FileTooLarge(usize),
    TooFewFiles(usize),
    TooManyFiles(usize),
    InvalidFileExtension(Option<String>),
    InvalidContentType(String),
    MissingFileExtension(String),
}

#[derive(Debug, Clone, Default)]
pub struct Validator {
    rules: HashMap<String, FileRules>,
}

#[derive(Debug, Default, Clone)]
pub struct RequestRules {
    /// Min number of files, this only works when validating through `Multipart` struct
    pub min_files: Option<usize>,

    /// Max number of files, this only works when validating through `Multipart` struct
    pub max_files: Option<usize>,
}

// Struct for File Validation Rules
#[derive(Debug, Default, Clone)]
pub struct FileRules {
    /// Whether field is required
    pub required: bool,

    /// Whether file extension is required
    pub extension_required: bool,

    /// Min file size in bytes
    pub min_size: Option<usize>,

    /// Max file size in bytes
    pub max_size: Option<usize>,

    /// Allowed file extensions
    pub allowed_extensions: Option<Vec<&'static str>>,

    /// Allowed content types
    pub allowed_content_types: Option<Vec<&'static str>>,

    /// Min number of files, this only works when validating through `Multipart` struct
    pub min_files: Option<usize>,

    /// Max number of files, this only works when validating through `Multipart` struct
    pub max_files: Option<usize>,
}

impl Validator {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_rule(&mut self, field: &str, rules: FileRules) -> Self {
        let mut validator = self.clone();
        validator.rules.insert(field.to_string(), rules);
        validator
    }

    pub fn validate(&self, files: &HashMap<String, Vec<FileInput>>) -> MultipartResult<()> {
        for (field_name, rules) in &self.rules {
            let files = files.get(field_name);
            Self::validate_files(field_name.clone(), files, rules)
                .map_err(MultipartError::ValidationError)?;
        }

        Ok(())
    }

    fn validate_files(
        field_name: String,
        files: Option<&Vec<FileInput>>,
        rules: &FileRules,
    ) -> Result<(), InputError> {
        if files.is_none() {
            if rules.required {
                return Err(InputError {
                    name: field_name,
                    error: ErrorMessage::NoFiles,
                });
            }

            return Ok(());
        }

        let files = files.unwrap();
        let file_count = files.len();

        // Validate required
        if rules.required && file_count == 0 {
            return Err(InputError {
                name: field_name,
                error: ErrorMessage::NoFiles,
            });
        }

        if file_count < rules.min_files.unwrap_or(0) {
            return Err(InputError {
                name: field_name,
                error: ErrorMessage::TooFewFiles(file_count),
            });
        }

        if file_count > rules.max_files.unwrap_or(usize::MAX) {
            return Err(InputError {
                name: field_name,
                error: ErrorMessage::TooManyFiles(file_count),
            });
        }

        for file in files {
            Self::validate_file(rules.clone(), file)?;
        }

        // If all checks passed
        Ok(())
    }

    fn validate_file(rule: FileRules, file: &FileInput) -> Result<(), InputError> {
        // Validate file extension
        if rule.extension_required && file.extension.is_none() {
            return Err(InputError {
                name: file.field_name.to_string(),
                error: ErrorMessage::MissingFileExtension(file.file_name.clone()),
            });
        }

        // Validate file size
        if let Some(min_size) = rule.min_size {
            if file.size < min_size {
                return Err(InputError {
                    name: file.field_name.to_string(),
                    error: ErrorMessage::FileTooSmall(min_size),
                });
            }
        }

        if let Some(max_size) = rule.max_size {
            if file.size > max_size {
                return Err(InputError {
                    name: file.field_name.to_string(),
                    error: ErrorMessage::FileTooLarge(max_size),
                });
            }
        }

        // Validate file extension
        if let Some(allowed_extensions) = &rule.allowed_extensions {
            if let Some(extension) = &file.extension {
                if !allowed_extensions.contains(&&*extension.to_lowercase()) {
                    return Err(InputError {
                        name: file.field_name.to_string(),
                        error: ErrorMessage::InvalidFileExtension(file.extension.clone()),
                    });
                }
            } else {
                return Err(InputError {
                    name: file.field_name.to_string(),
                    error: ErrorMessage::MissingFileExtension(file.file_name.clone()),
                });
            }
        }

        // Validate content type
        if let Some(allowed_content_types) = &rule.allowed_content_types {
            if !allowed_content_types.contains(&&*file.content_type.to_lowercase()) {
                return Err(InputError {
                    name: file.field_name.to_string(),
                    error: ErrorMessage::InvalidContentType(format!(
                        "Invalid content type. Allowed content types are: {:?}",
                        allowed_content_types
                    )),
                });
            }
        }

        Ok(())
    }
}
