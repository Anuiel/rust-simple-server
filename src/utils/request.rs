#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Type for getting request from client
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request_type")]
#[serde(rename_all = "lowercase")]
pub enum RequestType {
    /// Store request.
    /// Gets pair (```key```, ```value```) which client wants server yo store.
    Store {
        key: String,
        #[serde(rename = "hash")]
        value: String,
    },
    /// Load request.
    /// Gets which ```key``` client wants to load from server's storage.
    Load { key: String },
}
