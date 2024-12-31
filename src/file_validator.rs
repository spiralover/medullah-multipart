use crate::{FileInput, MultipartValidationError};

// Struct for File Validation Rules
#[derive(Debug, Default)]
pub struct FileValidationRules {
    pub min_size: Option<usize>,                    // Min file size in bytes
    pub max_size: Option<usize>,                    // Max file size in bytes
    pub allowed_extensions: Option<Vec<&'static str>>,    // Allowed file extensions
    pub allowed_content_types: Option<Vec<&'static str>>, // Allowed content types
}

impl FileValidationRules {
    pub fn validate(&self, file: &FileInput) -> Result<(), MultipartValidationError> {
        // Validate file size
        if let Some(min_size) = self.min_size {
            if file.size < min_size {
                return Err(MultipartValidationError::FileTooSmall(min_size));
            }
        }

        if let Some(max_size) = self.max_size {
            if file.size > max_size {
                return Err(MultipartValidationError::FileTooLarge(max_size));
            }
        }

        // Validate file extension
        if let Some(allowed_extensions) = &self.allowed_extensions {
            if let Some(extension) = &file.extension {
                if !allowed_extensions.contains(&&*extension.to_lowercase()) {
                    return Err(MultipartValidationError::InvalidFileExtension(Some(
                        extension.to_owned(),
                    )));
                }
            } else {
                return Err(MultipartValidationError::InvalidFileExtension(None));
            }
        }

        // Validate content type
        if let Some(allowed_content_types) = &self.allowed_content_types {
            if !allowed_content_types.contains(&&*file.content_type.to_lowercase()) {
                return Err(MultipartValidationError::InvalidContentType(format!(
                    "Invalid content type. Allowed content types are: {:?}",
                    allowed_content_types
                )));
            }
        }

        // If all checks passed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_validation() {
        let file_input = FileInput {
            file_name: "test.jpg".to_string(),
            size: 1048576, // 1MB in bytes
            content_type: "image/jpeg".to_string(),
            extension: Some("jpg".to_string()),
            ..Default::default()
        };

        let rules = FileValidationRules {
            min_size: Some(1024),            // Minimum size 1KB
            max_size: Some(2 * 1024 * 1024), // Maximum size 2MB
            allowed_extensions: Some(vec!["jpg", "png"]),
            allowed_content_types: Some(vec!["image/jpeg", "image/png"]),
        };

        // Validate file against rules
        assert!(rules.validate(&file_input).is_ok());

        // Invalid file (too large)
        let file_input_large = FileInput {
            size: 3 * 1024 * 1024, // 3MB
            ..file_input.clone()
        };

        assert!(rules.validate(&file_input_large).is_err());

        // Invalid extension
        let file_input_invalid_ext = FileInput {
            extension: Some("txt".to_string()),
            ..file_input.clone()
        };

        assert!(rules.validate(&file_input_invalid_ext).is_err());

        // Invalid content type
        let file_input_invalid_type = FileInput {
            content_type: "application/pdf".to_string(),
            ..file_input.clone()
        };

        assert!(rules.validate(&file_input_invalid_type).is_err());
    }
}
