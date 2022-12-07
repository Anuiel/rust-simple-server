#![forbid(unsafe_code)]

// use log::info;
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, TcpListener},
    sync::{Arc, Mutex},
};

use chrono::prelude::*;

use super::{request::RequestType, response::ResponseType, protocol::Protocol};

enum LogInfo<'a> {
    Request(&'a RequestType),
    ConnectionEstablished
}

fn print_log(ip: IpAddr, log: LogInfo, storage_size: usize) {
    print!("{} [{}] ", ip, Utc::now().format("%d/%b%Y:%T %z"), );
    match log {
        LogInfo::ConnectionEstablished => {
            print!("Connection established. ");
        },
        LogInfo::Request(request) => {
            match request {
                RequestType::Store { key, value } => {
                    print!("Received request to write new value {value} by key {key}. ")
                },
                RequestType::Load { key } => {
                    print!("Received request to get value by key {key}. ")
                },
                RequestType::Error => {
                    print!("Error while received request. ")
                },
            }
        }
    }
    println!("Storage size: {storage_size}.")
}

pub fn run(ip: IpAddr, port: u16) {
    let addr = SocketAddr::from((ip, port));
    let listener = TcpListener::bind(addr).unwrap();

    let data = HashMap::<String, String>::new();
    let data_ref = Arc::new(Mutex::new(data));
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let map = Arc::clone(&data_ref);
        print_log(ip, LogInfo::ConnectionEstablished, map.lock().unwrap().len());
        let _ = std::thread::Builder::new()
            .name(stream.peer_addr().unwrap().to_string())
            .spawn(move || loop {
                let request = match RequestType::load(&mut stream) {
                    Ok(request) => request,
                    Err(err) if err.kind() == std::io::ErrorKind::Other => continue,
                    Err(err) if err.kind() == std::io::ErrorKind::ConnectionAborted => break,
                    Err(_err) => {
                        print!("{}", _err.kind());
                        RequestType::Error
                    }
                };
                println!("{:?}", request);
                print_log(ip, LogInfo::Request(&request), map.lock().unwrap().len());
                let mut response = match request {
                    RequestType::Store { key, value } => {
                        map.lock().unwrap().insert(key, value);
                        ResponseType::StoreSuccess
                    }
                    RequestType::Load { key } => match map.lock().unwrap().get(&key) {
                        Some(value) => ResponseType::LoadSuccess {
                            key,
                            value: value.to_string(),
                        },
                        None => ResponseType::LoadNotFound,
                    },
                    RequestType::Error => ResponseType::Error,
                };
                response.send(&mut stream).unwrap();
            });
    }
}
