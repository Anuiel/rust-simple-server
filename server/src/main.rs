use clap::Parser;
use rand::Rng;
use std::{net::IpAddr, str::FromStr};

use server::run;

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
