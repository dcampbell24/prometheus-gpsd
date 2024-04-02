use std::io;
use std::net::TcpStream;

use gpsd_proto::{get_data, handshake, GpsdError, ResponseData};

pub fn demo_forever<R>(
    reader: &mut dyn io::BufRead,
    writer: &mut io::BufWriter<R>,
) -> Result<(), GpsdError>
where
    R: std::io::Write,
{
    handshake(reader, writer)?;

    loop {
        let msg = get_data(reader)?;
        if let ResponseData::Sky(sky) = msg {
                let sats = sky.satellites.map_or_else(
                    || 0,
                    |sats| sats.iter().filter(|sat| sat.used).map(|_| 1).sum(),
                );
                println!("{sats}");
            }
        }
    }

fn main() {
    if let Ok(stream) = TcpStream::connect("127.0.0.1:2947") {
        let mut reader = io::BufReader::new(&stream);
        let mut writer = io::BufWriter::new(&stream);
        demo_forever(&mut reader, &mut writer).unwrap();
    } else {
        panic!("Couldn't connect to gpsd...");
    }
}
