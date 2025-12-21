use std::io;

pub mod fetch;

use fetch::fetch;

fn main() -> io::Result<()> {
    fetch()?;
    Ok(())
}
