use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug)]
pub struct TurnstileError {
    pub message: String,
}

impl Display for TurnstileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TurnstileError: {}", self.message)
    }
}

impl std::error::Error for TurnstileError {}
