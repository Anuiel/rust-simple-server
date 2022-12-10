use clap::Parser;
use std::net::IpAddr;

mod utils;
use utils::run::run;

/// Structure to run server from command line much easier
#[derive(Debug, Parser)]
struct Opts {
    #[clap(short, long)]
    ip: IpAddr,

    #[clap(short, long, default_value = "0")]
    port: u16,
}

fn main() {
    let opts = Opts::parse();
    run(opts.ip, opts.port);
}