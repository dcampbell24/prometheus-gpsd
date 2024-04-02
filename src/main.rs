use std::net::{Ipv4Addr, TcpStream};
use std::sync::mpsc::channel; //, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};
use std::{io, net::SocketAddrV4};

use gpsd_proto::{get_data, ResponseData};
use metrics::gauge;
use metrics_exporter_prometheus::PrometheusBuilder;

fn main() {
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9002))
        .install()
        .expect("failed to install recorder/exporter");

    let _satellites = gauge!("satellites");

    let (tx, rx) = channel();

    thread::spawn(move || {
        if let Ok(stream) = TcpStream::connect("127.0.0.1:2947") {
            let mut reader = io::BufReader::new(&stream);
            let mut _writer = io::BufWriter::new(&stream);
            let mut count = 0;

            match get_data(&mut reader) {
                Ok(data) => println!("{data:?}"),
                Err(err) => println!("{err}"),
            }

            loop {
                let msg = get_data(&mut reader).unwrap();
                if let ResponseData::Sky(sky) = msg {
                    count = sky.satellites.map_or_else(
                        || 0,
                        |sats| sats.iter().filter(|sat| sat.used).map(|_| 1).sum(),
                    );
                    println!("{count}");
                    tx.send(count).expect("Unable to send on channel");
                } else {
                    println!("... {count}")
                }
            }
        } else {
            panic!("Couldn't connect to gpsd...");
        }
    });

    loop {
        let timeout = Duration::from_millis(10_000);
        let t0 = Instant::now();
        let mut count = 0;

        while t0.elapsed() < timeout {
            match rx.recv_timeout(timeout - t0.elapsed()) {
                Ok(count_) => count = count_,
                Err(_) => {
                    // println!("...");
                    break;
                }
            }
        }

        // println!("{count}")
    }
}
