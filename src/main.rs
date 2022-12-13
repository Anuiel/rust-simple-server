//! Implementation of a simple Contend Delivery Network. [Read more](https://github.com/Anuiel/rust-simple-server)
//! # Protocol of communication
//! All communication between the client and the server takes place through messages in json format.
//! ## store
//! Clients request:
//! ```
//! {
//!   "request_type": "store",
//!   "key": "some_key",
//!   "hash": "some_hash"
//! }
//! ```
//! Server response:
//! ```
//! {
//!   "response_status": "success"
//! }
//! ```
//!
//! ## load
//! Clients request:
//! ```
//! {
//!   "request_type": "load",
//!   "key": "some_key"
//! }
//! ```
//! Server response if there are ```hash``` under requested ```key```:
//! ```
//! {
//!   "response_status": "success",
//!   "requested_key": "some_key",
//!   "requested_hash": "0b672dd94fd3da6a8d404b66ee3f0c83",
//! }
//! ```
//! Otherwise
//! ```
//! {
//!   "response_status": "key not found",
//! }
//! ```

use clap::Parser;
use std::net::IpAddr;

mod utils;
use utils::run::run;

/// Structure to run server from command line much easier.
#[derive(Debug, Parser)]
struct Opts {
    #[clap(short, long)]
    ip: IpAddr,

    /// It is recommended to set `port` in range [1024, 49151]
    #[clap(short, long, default_value = "0")]
    port: u16,
}

fn main() {
    let opts = Opts::parse();
    run(opts.ip, opts.port);
}
