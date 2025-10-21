use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    pub user: String,
    pub message: String,
}

impl Message {
    pub fn new(user: String, msg: String) -> Self {
        Self { user, message: msg }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("[{}] {}", self.user, self.message))
    }
}
