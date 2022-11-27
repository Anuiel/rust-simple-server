use core::time;
use rand::{distributions::Alphanumeric, Rng};
use std::{
    collections::HashMap,
    fs::File,
    hash,
    io::{BufReader, Write},
    net::{IpAddr, SocketAddr, TcpListener, TcpStream},
    process::{Child, Command},
    str::FromStr,
    thread, sync::{Mutex, Arc}, time::Duration,
};

use server::{Protocol, Request, Response};

const BINARY_PATH: &str = env!("CARGO_BIN_EXE_server");

const DELIMETER: u8 = 1u8;

enum IpVersion {
    V4,
    V6,
}

struct ServerWrapper {
    proc: Option<Child>,
    addr: SocketAddr,
}

fn take_a_nap() {
    thread::sleep(Duration::from_millis(100));
}

impl ServerWrapper {
    fn start(ip_version: IpVersion) -> Self {
        let mut rng = rand::thread_rng();
        let port = rng.gen_range(40000..49151);
        let ip = match ip_version {
            IpVersion::V4 => IpAddr::from_str("127.0.0.1").unwrap(),
            IpVersion::V6 => IpAddr::from_str("::1").unwrap(),
        };

        eprintln!("binary path {}", BINARY_PATH);
        let proc = Command::new(BINARY_PATH)
            .arg("--ip")
            .arg(ip.to_string())
            .arg("--port")
            .arg(port.to_string())
            .spawn()
            .unwrap();
        thread::sleep(time::Duration::from_millis(1000));
        Self {
            proc: Some(proc),
            addr: SocketAddr::new(ip, port),
        }
    }

    fn is_alive(&mut self) -> bool {
        self.proc
            .as_mut()
            .map_or(false, |proc| proc.try_wait().unwrap().is_none())
    }

    fn stop(&mut self) -> std::io::Result<()> {
        self.proc.take().map_or(Ok(()), |mut proc| proc.kill())
    }
}

impl Drop for ServerWrapper {
    fn drop(&mut self) {
        let _ = self.stop().unwrap();
    }
}

fn random_string(string_size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(string_size)
        .map(char::from)
        .collect()
}

fn generate_dict(sample_size: usize, string_size: usize) -> HashMap<String, String> {
    let mut map = HashMap::<String, String>::new();
    for _ in 0..sample_size {
        map.insert(random_string(string_size), random_string(string_size));
    }
    map
}

fn check_response(conn: &mut TcpStream, should_be: Option<String>) -> bool {
    let response = Response::load(conn).unwrap();
    match should_be {
        None => response.is_ok(),
        Some(string) => {
            match response.requested_hash {
                None => false,
                Some(hash) => hash == string,
            }
        }
    }
}

#[test]
fn regular_map() {
    let server = ServerWrapper::start(IpVersion::V4);
    let mut conn = TcpStream::connect(server.addr).unwrap();

    let data = generate_dict(100, 20);

    for (key, hash) in &data {
        Request::new_store(key, hash).send(&mut conn).unwrap();
        assert!(check_response(&mut conn, None))
    }
    for (key, hash) in &data {
        Request::new_load(key).send(&mut conn).unwrap();
        assert!(check_response( &mut conn, Some(hash.to_string())));
    }
}

#[test]
fn response_test() {
    let server = ServerWrapper::start(IpVersion::V4);
    let mut conn = TcpStream::connect(server.addr).unwrap();

    Request::new_load(&random_string(20))
        .send(&mut conn)
        .unwrap();
    assert!(!check_response(&mut conn, None));

    let (key, hash1, hash2) = (random_string(20), random_string(20), random_string(20));
    Request::new_store(&key, &hash1)
        .send(&mut conn)
        .unwrap();
    assert!(check_response(&mut conn, None));
    Request::new_store(&key, &hash2)
        .send(&mut conn)
        .unwrap();
    assert!(check_response(&mut conn, None));

    Request::new_load(&key)
        .send(&mut conn)
        .unwrap();
    
    assert!(check_response(&mut conn, Some(hash2)));
}

#[test]
fn multithread() {
    let server = ServerWrapper::start(IpVersion::V4);

    let flat_map: Vec<(String, String)> = generate_dict(1000, 20).clone().into_iter().collect();
    let flat_map_ref = Arc::new(flat_map);

    let mut stream = TcpStream::connect(server.addr).unwrap();

    for i in 0..1000 {
        Request::new_store(&flat_map_ref[i].0, &flat_map_ref[i].1).send(&mut stream).unwrap();
        assert!(check_response(&mut stream, None));
    }

    let mut threads = vec![];
    for _ in 0..8 {
        let mut conn = TcpStream::connect(server.addr).unwrap();
        let data = Arc::clone(&flat_map_ref);
        threads.push(
            thread::spawn(move || {
                for _ in 0..30 {
                    Request::new_load(&data[rand::random::<usize>() % 1000].0).send(&mut conn).unwrap();
                    assert!(check_response(&mut conn, None));
                    take_a_nap();
                }

                for _ in 0..30 {
                    Request::new_load(&random_string(21)).send(&mut conn).unwrap();
                    assert!(!check_response(&mut conn, None));
                }
            })
        );
    }

    for thread in threads {
        thread.join().unwrap();
    }
}