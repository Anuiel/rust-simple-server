//! Struct to output log messages in separeted thread

use std::{
    net::SocketAddr,
    sync::mpsc,
    thread::{self, JoinHandle}, fmt::{Display, Formatter, self},
};

use super::request::Request;


/// Log struct
pub struct Log {
    ip: SocketAddr,
    request: Option<Request>,
    storage_capacity: usize,
}

impl Log {
    pub fn new(ip: SocketAddr, storage_capacity: usize, request: Option<Request>) -> Self {
        Log {
            ip,
            request,
            storage_capacity,
        }
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let time = chrono::Utc::now().format("%d/%b%Y:%T %z");
        match &self.request {
            None => {
                write!(
                    f,
                    "{} [{}] Connection established. Storage size: {}",
                    self.ip, time, self.storage_capacity
                )
            }
            Some(request) => {
                match request {
                    Request::Store { key, value } => {
                        write!(f, "{} [{}] Received request to write new value \"{value}\" by key \"{key}\". Storage size: {}.", self.ip, time, self.storage_capacity)
                    }
                    Request::Load { key } => {
                        write!(f, "{} [{}] Received request to get value by key \"{key}\". Storage size: {}.", self.ip, time, self.storage_capacity)
                    }
                }
            }
        }
    }
}

pub struct Logger {}

impl Logger {
    pub fn spawn(receiver: mpsc::Receiver<Log>) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let log = receiver.recv().unwrap();
            println!("{log}");
        })
    }
}
