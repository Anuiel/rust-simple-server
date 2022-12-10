#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Type for sending response back to client
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "response_status")]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    /// Server will send it after a successful store
    #[serde(rename = "success")]
    SuccessStore,
    #[serde(rename = "success")]
    /// Server will send it after a successful load
    /// This response also will send requested key
    SuccessLoad {
        #[serde(rename = "requested_key")]
        key: String,
        #[serde(rename = "requested_hash")]
        value: String,
    },
    /// Server will send it after a successful load
    /// This response means that client's key are not in server's storage
    #[serde(rename = "key not found")]
    KeyNotFound,
    /// Sever will send it if there was an incorrect request ot any other problem
    Error,
}
