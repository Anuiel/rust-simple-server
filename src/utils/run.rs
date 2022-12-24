#![forbid(unsafe_code)]
//! Functon that lauch the server and regulates all logic

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr, TcpListener},
    sync::{Arc, Mutex, mpsc},
};

use super::{protocol::Protocol, request::Request, response::Response, logger::{Logger, Log}};

/// Main function that wll lauch server on ```ip```:```port```
///
/// Communication protocol implemented throw json files.
pub fn run(ip: IpAddr, port: u16) {
    let addr = SocketAddr::from((ip, port));
    let listener = TcpListener::bind(addr).unwrap();

    let (main_sender, receiver) = mpsc::channel();
    let _ = Logger::spawn(receiver);
    let data = HashMap::<String, String>::new();
    let data_ref = Arc::new(Mutex::new(data));
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let storage = Arc::clone(&data_ref);
        let ip = stream.peer_addr().unwrap();
        let sender = main_sender.clone();

        sender
        .send(Log::new(ip, storage.lock().unwrap().len(), None))
        .unwrap();

        std::thread::Builder::new()
            .name(stream.peer_addr().unwrap().to_string())
            .spawn(move || loop {
                let request = match Request::load(&mut stream) {
                    Ok(request) => request,
                    Err(err) if err.kind() == std::io::ErrorKind::Other => continue,
                    Err(_err) => {
                        let _ = Response::Error.send(&mut stream);
                        break;
                    }
                };

                sender
                .send(Log::new(
                    ip,
                    storage.lock().unwrap().len(),
                    Some(request.clone()),
                ))
                .unwrap();

                let mut response = match request {
                    Request::Store { key, value } => {
                        storage.lock().unwrap().insert(key, value);
                        Response::SuccessStore
                    }
                    Request::Load { key } => match storage.lock().unwrap().get(&key) {
                        Some(value) => Response::SuccessLoad {
                            key,
                            value: value.to_string(),
                        },
                        None => Response::KeyNotFound,
                    },
                };
                response.send(&mut stream).unwrap();
            }).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{TcpListener, TcpStream},
        sync::{atomic::AtomicUsize, Arc},
        thread::{self, JoinHandle},
    };

    use crate::utils::{protocol::Protocol, request::Request, response::Response};

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
                        Request::Store {
                            key: format!("key {id}"),
                            value: id.to_string(),
                        }
                        .send(&mut stream)
                        .unwrap();
                        assert_eq!(
                            Response::load(&mut stream).unwrap(),
                            Response::SuccessStore
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
                    let request = Request::load(&mut stream).unwrap();
                    assert!(matches!(request, Request::Store { key: _, value: _ }));
                    Response::SuccessStore.send(&mut stream).unwrap();
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
