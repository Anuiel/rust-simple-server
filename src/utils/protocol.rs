#![forbid(unsafe_code)]

use std::{net::TcpStream, io::{BufReader, Write, BufRead}};
use serde::{de::DeserializeOwned, Serialize};

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
            return Result::Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "EOF",
            ));
        }
        Ok(serde_json::from_slice(&buf)?)
    }
}
