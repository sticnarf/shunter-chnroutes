extern crate clap;
extern crate ipnet;
extern crate iprange;
extern crate shunter;

use std::net::SocketAddr;
use std::io::{BufRead, BufReader};
use std::fs::File;
use clap::{App, Arg};
use iprange::IpRange;
use shunter::{Router, Shunter};
use shunter::redirect::*;

fn main() {
    let matches = App::new("shunter-chnroutes")
        .version("0.1.0")
        .author("Yilin Chen")
        .about("Example of shunter, route traffic through chnroutes")
        .arg(
            Arg::with_name("chnroutes")
                .short("c")
                .long("chnroutes")
                .value_name("FILE")
                .required(true)
                .help("The chnroutes file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("socks5")
                .short("s")
                .long("socks5")
                .value_name("<host:port>")
                .required(true)
                .help("SOCKS5 proxy on given host:port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("binding")
                .short("b")
                .long("binding")
                .value_name("IP")
                .default_value("127.0.0.1")
                .help("Bind shunter to the specific IP")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .default_value("1080")
                .help("Run shunter on the specific port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Enable verbose mode"),
        )
        .get_matches();

    let chnroutes = matches.value_of("chnroutes").unwrap();
    let chnroutes = File::open(chnroutes).expect("Fail to open chnroutes file");



    let socks5 = matches
        .value_of("socks5")
        .unwrap()
        .parse()
        .expect("Invalid SOCKS5 address");

    let bind = format!(
        "{}:{}",
        matches.value_of("binding").unwrap(),
        matches.value_of("port").unwrap()
    ).parse()
        .expect("Invalid binding IP or port");

    let router = ChnRouter::new(chnroutes, socks5);
    let shunter = Shunter::create(bind).expect("Unable to bind to given address");
    shunter.start(router);
}

struct ChnRouter {
    chnroutes: IpRange<ipnet::Ipv4Net>,
    socks5: SocketAddr,
}

impl ChnRouter {
    fn new(chnroutes: File, socks5: SocketAddr) -> ChnRouter {
        let reader = BufReader::new(chnroutes);
        let iprange = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter_map(|line| line.parse().ok())
            .collect();
        ChnRouter {
            chnroutes: iprange,
            socks5,
        }
    }
}

impl Router for ChnRouter {
    fn route(&self, _from: SocketAddr, to: SocketAddr) -> Box<Proxy> {
        match to {
            SocketAddr::V4(addr) => if self.chnroutes.contains(addr.ip()) {
                Box::new(Direct::new(to))
            } else {
                Box::new(Socks5::new(self.socks5, to))
            },
            SocketAddr::V6(_) => Box::new(Direct::new(to)),
        }
    }
}
