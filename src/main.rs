use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::{Duration, Instant};
use std::io;

use gpsd_proto::{get_data, handshake, GpsdError, ResponseData};
use metrics::gauge;
use metrics_exporter_prometheus::PrometheusBuilder;

fn main() {
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(SocketAddr::from(([0, 0, 0, 0], 9002)))
        .install()
        .expect("failed to install recorder/exporter");

    let satellites = gauge!("satellites");

    let (tx, rx) = channel();
    if let Ok(stream) = TcpStream::connect("127.0.0.1:2947") {
        thread::spawn(move || {
            let mut reader = io::BufReader::new(&stream);
            let mut writer = io::BufWriter::new(&stream);
            loop_forever(tx, &mut reader, &mut writer).unwrap();
        });
    } else {
        panic!("Couldn't connect to gpsd...");
    }

    loop {
        let timeout = Duration::from_millis(10_000);
        let t0 = Instant::now();
        let mut count = 0;

        while let Ok(count_) = rx.recv_timeout(timeout - t0.elapsed()) {
            count = count_;
        }

        satellites.set(count);
    }
}

pub fn loop_forever<R>(
    tx: Sender<i32>,
    mut reader: &mut dyn io::BufRead,
    writer: &mut io::BufWriter<R>,
) -> Result<(), GpsdError>
where
    R: std::io::Write,
{
    handshake(reader, writer).unwrap();
    let mut _count = 0;
    loop {
        let msg = get_data(&mut reader).unwrap();
        if let ResponseData::Sky(sky) = msg {
            _count = sky.satellites.map_or_else(
                || 0,
                |sats| sats.iter().filter(|sat| sat.used).map(|_| 1).sum(),
            );
            tx.send(_count).expect("Unable to send on channel");
        }
    }
}
