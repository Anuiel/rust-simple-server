#![forbid(unsafe_code)]
//! Trait that implement server-client communication as json files

use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

const DELIMETER: u8 = b'}';

/// Trait for types that implement json-like protocol
pub trait Protocol {
    /// Send ```Self``` into ```TcpStream``` as json.
    fn send(&mut self, stream: &mut TcpStream) -> std::io::Result<()>;
    /// Load ```Self``` from ```TcpStrem```.
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
        Ok(())
    }

    fn load(stream: &mut TcpStream) -> std::io::Result<Self> {
        let mut reader = BufReader::new(stream);
        let mut buf = Vec::<u8>::new();
        let bytes_read = reader.read_until(DELIMETER, &mut buf)?;
        if bytes_read == 0 {
            return Result::Err(std::io::Error::new(std::io::ErrorKind::Other, "EOF"));
        }
        Ok(serde_json::from_slice(&buf)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{request::RequestType, response::ResponseType};
    use std::{
        io::{BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        thread,
    };

    use super::Protocol;

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

    fn read_from_stream(stream: &mut TcpStream) -> String {
        let mut buf: Vec<u8> = vec![];
        let mut reader = BufReader::new(stream);
        reader.read_to_end(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    fn protocol_testing(
        user_request: String,
        user_expected_response: String,
        server_expected_request: RequestType,
        mut server_response: ResponseType,
        port: usize,
    ) {
        let addr = format!("localhost:{port}");
        let listener = TcpListener::bind(addr.clone()).unwrap();

        let user_thread = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            stream.write_all(user_request.as_bytes()).unwrap();

            assert_eq!(read_from_stream(&mut stream), user_expected_response);
        });

        let server_thread = thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let request = RequestType::load(&mut stream).unwrap();
                assert_eq!(request, server_expected_request);
                server_response.send(&mut stream).unwrap();
                break;
            }
        });

        user_thread.join().unwrap();
        server_thread.join().unwrap();
    }

    #[test]
    fn protocol_works() {
        protocol_testing(
            r#"{
            "request_type": "store",
            "key": "key",
            "hash": "hash"
          }"#
            .into(),
            r#"{
            "response_status": "success"
          }"#
            .to_string()
            .clear_whitespace(),
            RequestType::Store {
                key: "key".into(),
                value: "hash".into(),
            },
            ResponseType::SuccessStore,
            7777,
        );

        protocol_testing(
            r#"{
            "request_type": "load",
            "key": "key"
          }"#
            .into(),
            r#"{
            "response_status": "success",
            "requested_key": "key",
            "requested_hash": "hash"
          }"#
            .to_string()
            .clear_whitespace(),
            RequestType::Load { key: "key".into() },
            ResponseType::SuccessLoad {
                key: "key".into(),
                value: "hash".into(),
            },
            7778,
        );

        protocol_testing(
            r#"{
            "request_type": "load",
            "key": "key228"
          }"#
            .into(),
            r#"{
            "response_status": "key-not-found"
          }"#
            .to_string()
            .clear_whitespace()
            .replace("-", " "),
            RequestType::Load {
                key: "key228".into(),
            },
            ResponseType::KeyNotFound,
            7779,
        );
    }

    #[test]
    #[should_panic]
    fn protocol_error() {
        protocol_testing(
            r#"{
            "request_type": "rm -rf /"
          }"#
            .into(),
            "".into(),
            RequestType::Load {
                key: "key228".into(),
            },
            ResponseType::KeyNotFound,
            7776,
        );
    }
}
