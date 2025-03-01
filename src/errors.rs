use std::fmt::Formatter;

#[derive(Debug)]
pub enum JavaError {
    ConstantTypeError(String),
    InvalidConstantId(u16),
    StringNotFound,
}

impl std::fmt::Display for JavaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            JavaError::ConstantTypeError(message) => write!(f, "{}", message),
            JavaError::InvalidConstantId(id) => write!(f, "Invalid constant #{id}"),
            JavaError::StringNotFound => write!(f, "String not found"),
        }
    }
}

impl std::error::Error for JavaError {}
