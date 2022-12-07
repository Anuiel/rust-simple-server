use std::net::TcpStream;

use serde::{Serialize, Deserialize};

use super::protocol::Protocol;

#[derive(Debug)]
pub enum RequestType {
    Store { key: String, value: String },
    Load { key: String },
    Error,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Request {
    request_type: String,
    key: String,
    #[serde(rename = "hash")]
    value: Option<String>,
}

impl From<Request> for RequestType {
    fn from(request: Request) -> Self {
        match request.request_type.as_str() {
            "store" => match request.value {
                Some(value) => RequestType::Store {
                    key: request.key,
                    value,
                },
                None => RequestType::Error,
            },
            "load" => RequestType::Load { key: request.key },
            _ => RequestType::Error,
        }
    }
}

impl From<&mut RequestType> for Request {
    fn from(request: &mut RequestType) -> Self {
        match request {
            RequestType::Store { key, value } => Request {
                request_type: "store".to_string(),
                key: key.to_string(),
                value: Some(value.to_string()),
            },
            RequestType::Load { key } => Request {
                request_type: "load".to_string(),
                key: key.to_string(),
                value: None,
            },
            RequestType::Error => todo!(),
        }
    }
}

impl Protocol for RequestType {
    fn send(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut request = Request::from(self);
        request.send(stream)
    }

    fn load(stream: &mut TcpStream) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        Ok(RequestType::from(Request::load(stream)?))
    }
}