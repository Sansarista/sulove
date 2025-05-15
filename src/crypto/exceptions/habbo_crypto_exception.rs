use std::error::Error;
use std::fmt;

/// HabboCryptoException represents an error that occurred during a cryptographic operation
#[derive(Debug)]
pub struct HabboCryptoException {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl HabboCryptoException {
    /// Create a new HabboCryptoException with a message
    pub fn new(message: &str) -> Self {
        HabboCryptoException {
            message: message.to_string(),
            source: None,
        }
    }

    /// Create a new HabboCryptoException with a message and a cause
    pub fn with_cause<E>(message: &str, cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        HabboCryptoException {
            message: message.to_string(),
            source: Some(Box::new(cause)),
        }
    }

    /// Create a new HabboCryptoException from a cause
    pub fn from_cause<E>(cause: E) -> Self
    where
        E: Error + Send + Sync + 'static,
    {
        HabboCryptoException {
            message: cause.to_string(),
            source: Some(Box::new(cause)),
        }
    }
}

impl fmt::Display for HabboCryptoException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HabboCryptoException: {}" , self.message)
    }
}

impl Error for HabboCryptoException {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}
