#![forbid(unsafe_code)]

// use log::info;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, TcpListener},
    sync::{Arc, Mutex},
};

use chrono::prelude::*;

use super::{protocol::Protocol, request::RequestType, response::ResponseType};

/// Enum for simplier log output
enum LogInfo<'a> {
    Request(&'a RequestType),
    ConnectionEstablished,
}

/// Prints more readable log output
fn print_log(ip: IpAddr, log: LogInfo, storage_size: usize) {
    print!("{} [{}] ", ip, Utc::now().format("%d/%b%Y:%T %z"),);
    match log {
        LogInfo::ConnectionEstablished => {
            print!("Connection established. ");
        }
        LogInfo::Request(request) => match request {
            RequestType::Store { key, value } => {
                print!("Received request to write new value {value} by key {key}. ")
            }
            RequestType::Load { key } => {
                print!("Received request to get value by key {key}. ")
            }
        },
    }
    println!("Storage size: {storage_size}.")
}

/// Main function that wll lauch server on ```ip```:```port```
///
/// Communication protocol implemented throw json files. More about them c
pub fn run(ip: IpAddr, port: u16) {
    let addr = SocketAddr::from((ip, port));
    let listener = TcpListener::bind(addr).unwrap();

    let data = HashMap::<String, String>::new();
    let data_ref = Arc::new(Mutex::new(data));
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let map = Arc::clone(&data_ref);
        print_log(
            ip,
            LogInfo::ConnectionEstablished,
            map.lock().unwrap().len(),
        );
        let _ = std::thread::Builder::new()
            .name(stream.peer_addr().unwrap().to_string())
            .spawn(move || loop {
                let request = match RequestType::load(&mut stream) {
                    Ok(request) => request,
                    Err(err) if err.kind() == std::io::ErrorKind::Other => continue,
                    Err(_err) => {
                        let _ = ResponseType::Error.send(&mut stream);
                        break;
                    }
                };
                print_log(ip, LogInfo::Request(&request), map.lock().unwrap().len());
                let mut response = match request {
                    RequestType::Store { key, value } => {
                        map.lock().unwrap().insert(key, value);
                        ResponseType::SuccessStore
                    }
                    RequestType::Load { key } => match map.lock().unwrap().get(&key) {
                        Some(value) => ResponseType::SuccessLoad {
                            key,
                            value: value.to_string(),
                        },
                        None => ResponseType::KeyNotFound,
                    },
                };
                response.send(&mut stream).unwrap();
            });
    }
}
