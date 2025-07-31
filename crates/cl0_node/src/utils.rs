use thiserror::Error;

/// The possible values a condition variable can take in the system.
///
/// - `True` and `False` are concrete boolean states.
/// - `Unknown` represents a value that is not yet determined; callers can choose to
///   treat it differently (e.g., continue or fail) depending on context.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarValue {
    True,
    False, // Inactive
    Unknown,
}

impl VarValue {
    /// Returns `Some(bool)` for concrete values, or `None` for `Unknown`.
    pub fn as_option_bool(&self) -> Option<bool> {
        match self {
            VarValue::True => Some(true),
            VarValue::False => Some(false),
            VarValue::Unknown => None,
        }
    }

    /// Converts into a boolean, returning an error for ambiguous states.
    pub fn to_bool(&self) -> Result<bool, VarError> {
        self.as_option_bool()
            .ok_or(VarError::UnsupportedValue(self.clone()))
    }
}

/// Precise errors related to `VarValue` conversions or usage.
#[derive(Debug, Error)]
pub enum VarError {
    #[error("unsupported variable value: {0:?}")]
    UnsupportedValue(VarValue),
}
