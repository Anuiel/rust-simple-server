#![forbid(unsafe_code)]
//! Rules for server response
//! ## Example
//! This json-like text
//! ```
//! let response = ResponseType::SuccessStore;
//!
//! let ser_response = serde_json::to_string(response).unwrap();
//!
//! assert_eq!(ser_responce, r#"{"response_status":"success"#);
//! ```
//! Or
//! ```
//! let response = ResponseType::SuccessLoad {key: "key".into(), value: "hash".into()};
//!
//! let ser_response = serde_json::to_string(response).unwrap();
//!
//! assert_eq!(ser_responce,
//!     r#"
//!     {
//!         "response_status": "success",
//!         "requested_key": "key",
//!         "requested_hash": "hash"
//!     }"#
//!     .to_string()
//!     .replace(" ", "")
//!     .replace("\n", "")
//! );
//! ```

use serde::{Deserialize, Serialize};

/// Type for sending response back to client
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "response_status")]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    /// Server will send it after a successful store
    #[serde(rename = "success")]
    SuccessStore,
    #[serde(rename = "success")]
    /// Server will send it after a successful load
    /// This response also will send requested ```key```
    SuccessLoad {
        #[serde(rename = "requested_key")]
        key: String,
        #[serde(rename = "requested_hash")]
        value: String,
    },
    /// Server will send it after a successful load
    /// This response means that ```key``` are not in server's storage
    #[serde(rename = "key not found")]
    KeyNotFound,
    /// Sever will send it if there was an incorrect request ot any other problem
    Error,
}

#[cfg(test)]
mod tests {

    use super::ResponseType;

    trait Clear {
        fn clear_whitespace(&mut self) -> Self
        where
            Self: Sized;
    }

    impl Clear for String {
        fn clear_whitespace(&mut self) -> String {
            self.retain(|c| !c.is_whitespace());
            self.to_string()
        }
    }

    fn make_success_store_response() -> String {
        r#"
        {
            "response_status": "success"
        }
        "#
        .to_string()
        .clear_whitespace()
    }

    fn make_key_not_found_response() -> String {
        r#"
        {
            "response_status": "key-not-found"
        }
        "#
        .to_string()
        .clear_whitespace()
        .replace("-", " ")
    }

    fn make_success_load_response(key: &str, hash: &str) -> String {
        format!(
            r#"
        {{
            "response_status": "success",
            "requested_key": "{key}",
            "requested_hash": "{hash}"
        }}
        "#
        )
        .clear_whitespace()
    }

    fn make_error_response() -> String {
        r#"
        {
            "response_status": "error"
        }
        "#
        .to_string()
        .clear_whitespace()
    }

    #[test]
    fn response_store_works() {
        let response = ResponseType::SuccessStore;
        let serialize_response = serde_json::to_string(&response).unwrap();
        assert_eq!(serialize_response, make_success_store_response());

        let response = ResponseType::KeyNotFound;
        let serialize_response = serde_json::to_string(&response).unwrap();
        assert_eq!(serialize_response, make_key_not_found_response());

        let response = ResponseType::SuccessLoad {
            key: "key".into(),
            value: "value".into(),
        };
        let serialize_response = serde_json::to_string(&response).unwrap();
        assert_eq!(
            serialize_response,
            make_success_load_response("key", "value")
        );

        let response = ResponseType::Error;
        let serialize_response = serde_json::to_string(&response).unwrap();
        assert_eq!(serialize_response, make_error_response());
    }
}
