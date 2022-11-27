#![forbid(unsafe_code)]

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    str,
    sync::{Arc, Mutex},
};
use log::{error, info};
use serde::{
    de::{value, DeserializeOwned},
    Deserialize, Serialize,
};

const DELIMETER: u8 = 1u8;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request {
    request_type: String,
    key: String,
    hash: Option<String>,
}

impl Request {
    pub fn new_store(key: &String, value: &String) -> Self {
        Request {
            request_type: "store".to_string(),
            key: key.to_string(),
            hash: Some(value.to_string()),
        }
    }

    pub fn new_load(key: &String) -> Self {
        Request {
            request_type: "load".to_string(),
            key: key.to_string(),
            hash: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Response {
    pub response_status: String,
    pub requested_key: Option<String>,
    pub requested_hash: Option<String>,
}

impl Response {
    pub fn is_ok(&self) -> bool {
        self.response_status == "success".to_string()
    }

    pub fn not_found(&self) -> bool {
        self.response_status == "key not found".to_string()
    }
}

pub trait Protocol {
    fn send(&mut self, stream: &mut TcpStream) -> std::io::Result<()>;
    fn load(stream: &mut TcpStream) -> std::io::Result<Self>
    where
        Self: Sized;
}

impl<T> Protocol for T
where
    T: Serialize + DeserializeOwned + Sized,
{
    fn send(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_all(serde_json::to_string(self)?.as_bytes())?;
        stream.write_all(&[DELIMETER])?;
        Ok(())
    }

    fn load(stream: &mut TcpStream) -> std::io::Result<Self> {
        let mut reader = BufReader::new(stream);
        let mut buf = Vec::<u8>::new();
        let _ = reader.read_until(DELIMETER, &mut buf)?;
        buf.pop();
        Ok(serde_json::from_str(str::from_utf8(&buf).unwrap())?)
    }
}

pub fn run(ip: IpAddr, port: u16) {
    let addr = SocketAddr::from((ip, port));
    let listener = TcpListener::bind(addr).unwrap();

    let data = HashMap::<String, String>::new();
    let data_ref = Arc::new(Mutex::new(data));

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let map = Arc::clone(&data_ref);
        let _ = std::thread::Builder::new()
            .name(stream.peer_addr().unwrap().to_string())
            .spawn(move || loop {
                let request = match Request::load(&mut stream) {
                    Ok(request) => request,
                    Err(e) => break,
                };
                error!("{request:?}");
                let mut response = match request.request_type.as_str() {
                    "store" => {
                        map.lock()
                            .unwrap()
                            .insert(request.key, request.hash.unwrap());
                        Response {
                            response_status: "success".to_string(),
                            requested_key: None,
                            requested_hash: None,
                        }
                    }
                    "load" => match map.lock().unwrap().get(&request.key) {
                        Some(hash) => Response {
                            response_status: "success".to_string(),
                            requested_key: Some(request.key),
                            requested_hash: Some(hash.to_owned()),
                        },
                        None => Response {
                            response_status: "key not found".to_string(),
                            requested_key: None,
                            requested_hash: None,
                        },
                    },
                    _ => Response {
                        response_status: "error".to_string(),
                        requested_key: None,
                        requested_hash: None,
                    },
                };
                response.send(&mut stream).unwrap();
            });
    }
}
