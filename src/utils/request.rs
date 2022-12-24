#![forbid(unsafe_code)]
//! Regulates types of request that server can receive
//! ## Example
//! This json-like text
//! ```
//! let store = r#"{
//!         "request_type": "store",
//!             "key": "key",
//!             "hash": "hash"
//!         }"#;
//! let request = serde_json::from_str(&store).unwrap();
//!
//! assert_eq!(request,
//!     Request::Store {
//!         key: "key".to_string(),
//!         value: "hash".to_string(),
//!     }
//! );
//! ```
//! Or
//! ```
//! let load = r#"{
//!         "request_type": "load",
//!             "key": "key"
//!         }"#;
//! let request = serde_json::from_str(&load).unwrap();
//!
//! assert_eq!(request,
//!     Request::Load {
//!         key: "key".to_string()
//!     }
//! );
//! ```
use serde::{Deserialize, Serialize};

/// Type for getting request from client
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "request_type")]
#[serde(rename_all = "lowercase")]
pub enum Request {
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

#[cfg(test)]
mod tests {
    use super::Request;

    fn make_store_request(key: &str, hash: &str) -> String {
        format!(
            r#"
        {{
            "request_type": "store",
            "key": "{key}",
            "hash": "{hash}"
        }}
        "#
        )
    }

    fn make_load_request(key: &str) -> String {
        format!(
            r#"
        {{
            "request_type": "load",
            "key": "{key}"
        }}
        "#
        )
    }

    #[test]
    fn request_store_works() {
        let request: Request =
            serde_json::from_str(make_store_request("key", "1234567890").as_str()).unwrap();
        assert_eq!(
            request,
            Request::Store {
                key: "key".into(),
                value: "1234567890".into()
            }
        );

        let request: Request =
            serde_json::from_str(make_store_request("😀", "😡😡").as_str()).unwrap();
        assert_eq!(
            request,
            Request::Store {
                key: "😀".into(),
                value: "😡😡".into()
            }
        );
    }

    #[test]
    fn request_load_works() {
        let request: Request = serde_json::from_str(make_load_request("key").as_str()).unwrap();
        assert_eq!(request, Request::Load { key: "key".into() });
    }

    #[test]
    #[should_panic]
    fn incorrect_request1() {
        let request_str = r#"
        {
            "request_type": "aboba",
            "key": "key"
        }
        "#;

        let _: Request = serde_json::from_str(request_str).unwrap();
    }

    #[test]
    #[should_panic]
    fn incorrect_request2() {
        let request_str = r#"
        {
            "request_type": "load",
            "keys": "key",
        }
        "#;

        let _: Request = serde_json::from_str(request_str).unwrap();
    }
}
