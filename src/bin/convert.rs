use ennui::map::Room;
use std::env::args;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    if let [srcfile, dstfile, ..] = args().skip(1).take(2).collect::<Vec<String>>().as_slice() {
        let x = fs::read(srcfile)?;
        let r: Vec<Room> = serde_yaml::from_slice(&x[..]).unwrap_or_default();
        println!("{:#?}", r);
        let buf = serde_cbor::to_vec(&r).unwrap_or_default();
        std::fs::File::create(dstfile)?;
        std::fs::write(dstfile, buf)?;
    } else {
        println!("usage: convert <srcfile> <dstfile>");
    }

    Ok(())
}
