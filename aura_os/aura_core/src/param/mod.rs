// aura_os/aura_core/src/param/mod.rs

use crate::error::{AuraError, Result};
use crate::aura_log; // Internal logging macro
use std::collections::HashMap;
use std::sync::{Arc, RwLock}; // RwLock for efficient read-heavy access to parameters

/// Represents the possible types of values a parameter can hold.
///
/// In a more complete system, this enum would support more complex types like:
/// - `Vec<ParamValue>` for lists/arrays of parameters.
/// - `HashMap<String, ParamValue>` for nested parameter structures/dictionaries.
/// - Byte arrays for arbitrary binary data.
/// It would also likely be backed by a serialization framework (e.g., `serde`)
/// to allow easy conversion to/from common configuration file formats (YAML, JSON, TOML).
#[derive(Debug, Clone, PartialEq)] // PartialEq for easy comparison in tests and logic
pub enum ParamValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    // Future extensions:
    // StringArray(Vec<String>),
    // IntArray(Vec<i64>),
    // FloatArray(Vec<f64>),
    // BoolArray(Vec<bool>),
    // ByteArray(Vec<u8>),
}

impl ParamValue {
    // --- Helper methods for convenient type casting ---
    // These methods provide a safer way to get the underlying value,
    // returning an `Option` or a `Result` to handle potential type mismatches.

    pub fn as_string(&self) -> Option<&String> {
        if let ParamValue::String(s) = self { Some(s) } else { None }
    }

    pub fn as_i64(&self) -> Option<i64> {
        if let ParamValue::Int(i) = self { Some(*i) } else { None }
    }

    pub fn as_f64(&self) -> Option<f64> {
        if let ParamValue::Float(f) = self { Some(*f) } else { None }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let ParamValue::Bool(b) = self { Some(*b) } else { None }
    }

    // Example of a more robust getter that returns a Result for better error handling
    pub fn get_string(&self) -> Result<&String> {
        match self {
            ParamValue::String(s) => Ok(s),
            _ => Err(AuraError::ParameterConfigurationError(format!(
                "Expected String, found {:?}",
                self
            ))),
        }
    }
    // Similar `get_i64()`, `get_f64()`, `get_bool()` methods could be added.
}

/// Manages parameters for a specific scope (e.g., a node or a global context).
///
/// This `ParameterManager` provides an API to declare, set, and get parameters.
/// In a complete AuraOS system:
/// - It would interact with a distributed parameter server or load parameters
///   from configuration files (YAML, TOML, JSON).
/// - It would support parameter descriptors (defining type, range, description).
/// - It would allow registering callbacks for parameter change events.
/// - It would integrate with the `aura` CLI for `aura param list/get/set/load` commands.
#[derive(Debug)]
pub struct ParameterManager {
    /// The scope or owner of these parameters (e.g., a fully qualified node name).
    /// Used for logging and potentially for namespacing in a global parameter server.
    scope_name: String,
    /// The internal storage for parameters.
    /// `Arc<RwLock<...>>` allows multiple readers or one writer, suitable for
    /// parameters that are read frequently and written less often.
    parameters: Arc<RwLock<HashMap<String, ParamValue>>>,
}

impl ParameterManager {
    /// Creates a new `ParameterManager` for a given scope.
    pub fn new(scope_name: &str) -> Self {
        aura_log!(debug, "Initializing ParameterManager for scope: '{}'", scope_name);
        Self {
            scope_name: scope_name.to_string(),
            parameters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Declares a parameter with a name and a default value.
    ///
    /// If the parameter has already been declared (e.g., by loading from a file
    /// or set externally), its value is not typically overwritten by the default,
    /// unless an override mechanism is implemented.
    ///
    /// # Arguments
    /// * `name`: The name of the parameter (e.g., "publish_rate_hz", "sensor_topic_name").
    /// * `default_value`: The `ParamValue` to use if the parameter is not already set.
    ///
    /// # Returns
    /// `Ok(())` if successful, or an `AuraError` if declaration fails (e.g., lock poisoning).
    pub fn declare_parameter(&self, name: &str, default_value: ParamValue) -> Result<()> {
        aura_log!(
            info,
            "[{}] Declaring parameter '{}' with default: {:?}",
            self.scope_name,
            name,
            default_value
        );
        let mut params_writer = self
            .parameters
            .write()
            .map_err(|_| AuraError::ParameterConfigurationError(
                format!("[{}] Failed to acquire write lock for parameters.", self.scope_name)
            ))?;

        // `entry(key).or_insert(value)` is an idiomatic way to insert if not present.
        params_writer.entry(name.to_string()).or_insert(default_value);
        Ok(())
    }

    /// Sets the value of a parameter.
    ///
    /// If the parameter was not previously declared, this method might create it
    /// or return an error, depending on the desired policy. For this sketch,
    /// we assume it must be declared first or we allow dynamic creation.
    /// Let's allow dynamic creation for simplicity in the sketch.
    ///
    /// # Arguments
    /// * `name`: The name of the parameter.
    /// * `value`: The new `ParamValue` for the parameter.
    ///
    /// # Returns
    /// `Ok(())` if successful, or an `AuraError`.
    pub fn set_parameter(&self, name: &str, value: ParamValue) -> Result<()> {
        aura_log!(
            info,
            "[{}] Setting parameter '{}' to: {:?}",
            self.scope_name,
            name,
            value
        );
        let mut params_writer = self
            .parameters
            .write()
            .map_err(|_| AuraError::ParameterConfigurationError(
                format!("[{}] Failed to acquire write lock for parameters.", self.scope_name)
            ))?;

        params_writer.insert(name.to_string(), value);
        // In a real system, this might trigger parameter change callbacks.
        Ok(())
    }

    /// Gets the value of a parameter.
    ///
    /// # Arguments
    /// * `name`: The name of the parameter to retrieve.
    ///
    /// # Returns
    /// A `Result` containing the `ParamValue` if found, or an `AuraError::ParameterNotFound`
    /// if the parameter does not exist.
    pub fn get_parameter(&self, name: &str) -> Result<ParamValue> {
        let params_reader = self
            .parameters
            .read()
            .map_err(|_| AuraError::ParameterConfigurationError(
                format!("[{}] Failed to acquire read lock for parameters.", self.scope_name)
            ))?;

        match params_reader.get(name) {
            Some(value) => {
                aura_log!(trace, "[{}] Getting parameter '{}': {:?}", self.scope_name, name, value);
                Ok(value.clone()) // Clone the value to return ownership
            }
            None => {
                aura_log!(warn, "[{}] Parameter '{}' not found.", self.scope_name, name);
                Err(AuraError::ParameterNotFound(format!(
                    "[{}] Parameter '{}' not found.",
                    self.scope_name, name
                )))
            }
        }
    }

    /// Checks if a parameter exists.
    pub fn has_parameter(&self, name: &str) -> Result<bool> {
        let params_reader = self
            .parameters
            .read()
            .map_err(|_| AuraError::ParameterConfigurationError(
                format!("[{}] Failed to acquire read lock for parameters.", self.scope_name)
            ))?;
        Ok(params_reader.contains_key(name))
    }

    // --- Future Enhancements ---
    // - `load_from_file(path: &Path, format: ConfigFormat) -> Result<()>`
    // - `get_parameter_with_descriptor(name: &str) -> Result<(ParamValue, ParameterDescriptor)>`
    // - `add_on_parameter_changed_callback(names: Vec<String>, callback: Arc<dyn Fn(Vec<ParameterEvent>)>)`
    // - `list_parameters(prefix: &str) -> Result<Vec<(String, ParamValue)>>`
}