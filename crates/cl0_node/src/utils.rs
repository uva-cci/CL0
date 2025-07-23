use std::{error::Error, sync::Arc};
use tokio::sync::Mutex;

pub type Shared<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]pub enum VarValue {
    True,
    False, // Inactive
    Null,
    Unknown,
}

impl VarValue {
    pub fn to_bool(&self) -> Result<bool, Box<dyn Error>> {
        match self {
            VarValue::True => Ok(true),
            VarValue::False => Ok(false),
            VarValue::Null | VarValue::Unknown => Err(Box::<dyn Error>::from("Unsupported variable value")), // Need to handle these cases
        }
    }
}
