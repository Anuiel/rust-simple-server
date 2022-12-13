#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Type for getting request from client
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::RequestType;

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
        let request: RequestType =
            serde_json::from_str(make_store_request("key", "1234567890").as_str()).unwrap();
        assert_eq!(
            request,
            RequestType::Store {
                key: "key".into(),
                value: "1234567890".into()
            }
        );

        let request: RequestType =
            serde_json::from_str(make_store_request("😀", "😡😡").as_str()).unwrap();
        assert_eq!(
            request,
            RequestType::Store {
                key: "😀".into(),
                value: "😡😡".into()
            }
        );
    }

    #[test]
    fn request_load_works() {
        let request: RequestType = serde_json::from_str(make_load_request("key").as_str()).unwrap();
        assert_eq!(request, RequestType::Load { key: "key".into() });
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

        let _: RequestType = serde_json::from_str(request_str).unwrap();
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

        let _: RequestType = serde_json::from_str(request_str).unwrap();
    }
}
