use std::io;

pub mod mnea_parser;

use mnea_parser::parse_nmea;

fn main() -> io::Result<()> {
    parse_nmea()?;
    Ok(())
}
