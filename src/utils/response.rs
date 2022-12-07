use std::net::TcpStream;

use serde::{Deserialize, Serialize};

use super::protocol::Protocol;

#[derive(Debug)]
pub enum ResponseType {
    StoreSuccess,
    LoadSuccess { key: String, value: String },
    LoadNotFound,
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Response {
    response_status: String,
    #[serde(rename = "requested_key")]
    key: Option<String>,
    #[serde(rename = "requested_hash")]
    value: Option<String>,
}

impl From<&mut ResponseType> for Response {
    fn from(response_type: &mut ResponseType) -> Response {
        match response_type {
            ResponseType::StoreSuccess => Response {
                response_status: "success".to_string(),
                key: None,
                value: None,
            },
            ResponseType::LoadSuccess { key, value } => Response {
                response_status: "success".to_string(),
                key: Some(key.to_string()),
                value: Some(value.to_string()),
            },
            ResponseType::LoadNotFound => Response {
                response_status: "key not found".to_string(),
                key: None,
                value: None,
            },
            ResponseType::Error => todo!(),
        }
    }
}

impl From<Response> for ResponseType {
    fn from(response: Response) -> ResponseType {
        match response.response_status.as_str() {
            "load" => {
                if response.value == None || response.key == None {
                    panic!()
                }
                ResponseType::LoadSuccess {
                    key: response.key.unwrap(),
                    value: response.value.unwrap(),
                }
            }
            "store" => ResponseType::StoreSuccess,
            "key not found" => ResponseType::LoadNotFound,
            _ => ResponseType::Error,
        }
    }
}

impl Protocol for ResponseType {
    fn send(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut request = Response::from(self);
        request.send(stream)
    }

    fn load(stream: &mut TcpStream) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(ResponseType::from(Response::load(stream)?))
    }
}