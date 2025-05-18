
// aura_os/aura_core/src/error.rs

use std::fmt;

/// The primary error type for all operations within `aura_core`.
///
/// As AuraOS grows, more specific error variants will be added here.
/// Each variant can carry additional context about the error.
/// Deriving `Debug` allows these errors to be easily printed for debugging.
#[derive(Debug)]
pub enum AuraError {
    /// Error during the initialization of AuraOS core components.
    InitializationFailed(String),
    /// Errors related to communication (publishing, subscribing, service calls).
    CommunicationError(String),
    /// Errors when a requested parameter is not found or cannot be accessed.
    ParameterNotFound(String),
    /// Errors related to parameter configuration or type mismatches.
    ParameterConfigurationError(String),
    /// Errors originating from node operations (creation, lifecycle, etc.).
    NodeError(String),
    /// General configuration errors (e.g., invalid file format, missing settings).
    ConfigurationError(String),
    /// Errors related to serialization or deserialization of messages/data.
    SerializationError(String),
    /// Errors when an operation times out.
    TimeoutError(String),
    /// An error indicating that a feature is not yet implemented.
    NotImplemented(String),
    /// A wrapper for underlying I/O errors.
    IoError(std::io::Error),
    /// A generic, unspecified error. Avoid using this if a more specific variant fits.
    Other(String),
}

// Implement the `Display` trait to provide a user-friendly representation of the error.
// This is what users will typically see when an error is printed.
impl fmt::Display for AuraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuraError::InitializationFailed(s) => write!(f, "AuraOS Initialization Failed: {}", s),
            AuraError::CommunicationError(s) => write!(f, "AuraOS Communication Error: {}", s),
            AuraError::ParameterNotFound(s) => write!(f, "AuraOS Parameter Not Found: {}", s),
            AuraError::ParameterConfigurationError(s) => write!(f, "AuraOS Parameter Configuration Error: {}", s),
            AuraError::NodeError(s) => write!(f, "AuraOS Node Error: {}", s),
            AuraError::ConfigurationError(s) => write!(f, "AuraOS Configuration Error: {}", s),
            AuraError::SerializationError(s) => write!(f, "AuraOS Serialization Error: {}", s),
            AuraError::TimeoutError(s) => write!(f, "AuraOS Operation Timed Out: {}", s),
            AuraError::NotImplemented(s) => write!(f, "AuraOS Feature Not Implemented: {}", s),
            AuraError::IoError(e) => write!(f, "AuraOS I/O Error: {}", e),
            AuraError::Other(s) => write!(f, "AuraOS Error: {}", s),
        }
    }
}

// Implement the `Error` trait to allow `AuraError` to be used with other
// Rust error handling mechanisms (e.g., `?` operator, `Box<dyn Error>`).
impl std::error::Error for AuraError {
    // `source()` can be implemented to provide the underlying cause of an error,
    // useful for error chaining.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AuraError::IoError(e) => Some(e),
            // Other variants currently don't wrap another error, so they return None.
            _ => None,
        }
    }
}

// Implement `From<std::io::Error>` to allow easy conversion from standard I/O errors
// into `AuraError`. This is convenient when working with file operations or networking.
// Example: `fs::read_to_string("file.txt")?` can automatically convert an `io::Error`
//          into an `AuraError::IoError` if the function returns `aura_core::Result<()>`.
impl From<std::io::Error> for AuraError {
    fn from(err: std::io::Error) -> Self {
        AuraError::IoError(err)
    }
}

// Define a convenient `Result` type alias for functions within `aura_core`
// and for users of the crate. This avoids having to write `std::result::Result<T, AuraError>`
// everywhere.
pub type Result<T> = std::result::Result<T, AuraError>;