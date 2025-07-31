use std::{error::Error, fmt};
use tokio::task::JoinHandle;
use futures::future::join_all;

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
    pub fn to_bool(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.as_option_bool()
            .ok_or(Box::<dyn Error + Send + Sync>::from(format!("Not a boolean value: {}", self)))
    }
}

impl fmt::Display for VarValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarValue::True => write!(f, "True"),
            VarValue::False => write!(f, "False"),
            VarValue::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Awaits a collection of `JoinHandle<Result<bool, E>>`, returns the conjunction
/// of all their successful boolean results, or the first error encountered.
pub async fn collect_conjunction(
    handles: Vec<JoinHandle<Result<bool, Box<dyn std::error::Error + Send + Sync>>>>,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut overall = true;
    for join_res in join_all(handles).await {
        match join_res {
            Ok(inner) => match inner {
                Ok(val) => overall &= val,
                Err(e) => return Err(e),
            },
            Err(join_err) => return Err(Box::<dyn Error + Send + Sync>::from(join_err)),
        }
    }
    Ok(overall)
}