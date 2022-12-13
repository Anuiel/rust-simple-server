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
        std::thread::Builder::new()
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
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{TcpListener, TcpStream},
        sync::{atomic::AtomicUsize, Arc},
        thread::{self, JoinHandle},
    };

    use crate::utils::{protocol::Protocol, request::RequestType, response::ResponseType};

    #[test]
    fn multiple_users() {
        let addr = "localhost:7775";
        let thread_count = 100;
        let listener = TcpListener::bind(addr).unwrap();
        let users_threads: Vec<JoinHandle<()>> = (0..thread_count)
            .into_iter()
            .map(|id| {
                std::thread::Builder::new()
                    .name(id.to_string())
                    .spawn(move || {
                        let mut stream = TcpStream::connect(addr).unwrap();
                        RequestType::Store {
                            key: format!("key {id}"),
                            value: id.to_string(),
                        }
                        .send(&mut stream)
                        .unwrap();
                        assert_eq!(
                            ResponseType::load(&mut stream).unwrap(),
                            ResponseType::SuccessStore
                        );
                    })
                    .unwrap()
            })
            .collect();

        let server_thread = thread::spawn(move || {
            let counter = Arc::new(AtomicUsize::new(0));
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let cnt = Arc::clone(&counter);
                thread::spawn(move || {
                    let request = RequestType::load(&mut stream).unwrap();
                    assert!(matches!(request, RequestType::Store { key: _, value: _ }));
                    ResponseType::SuccessStore.send(&mut stream).unwrap();
                    cnt.fetch_add(1, std::sync::atomic::Ordering::Release);
                })
                .join()
                .unwrap();
                let cnt = counter.load(std::sync::atomic::Ordering::Relaxed);
                if cnt >= thread_count {
                    break;
                }
            }
        });

        for user_thread in users_threads {
            user_thread.join().unwrap();
        }
        server_thread.join().unwrap();
    }
}
