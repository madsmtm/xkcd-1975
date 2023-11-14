use std::io;

use xkcd_1975::Data;

fn main() -> io::Result<()> {
    let _: Data = Data::load()?;
    // println!("{data:?}");
    Ok(())
}
