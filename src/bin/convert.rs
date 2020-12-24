use ennui::item::Item;
use ennui::map::Room;
use std::env::args;
use std::fs;
use std::io;
use std::io::Write;

fn main() -> io::Result<()> {
    if let [srcfile, dstfile, ..] = args().skip(1).take(2).collect::<Vec<String>>().as_slice() {
        let x = fs::read(srcfile)?;
        let r: Vec<Room> = serde_yaml::from_slice(&x[..]).unwrap_or_default();
        println!("{:#?}", r);
        let mut f = fs::File::create(dstfile)?;
        serde_cbor::to_writer(&f, &r);
        println!("cbor written to {}", dstfile);
        f.flush()?;
    } else {
        println!("usage: convert <srcfile> <dstfile>");
    }

    Ok(())
}
