use std::io;

pub mod nmea_parser;

use nmea_parser::{parse_nmea, run};

fn main() -> io::Result<()> {
    parse_nmea()?;
    run()?;
    Ok(())
}
