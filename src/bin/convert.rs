use std::env::args;
use std::fs;
use std::io;
use ennui::map::Room;

fn main() -> io::Result<()> {
    if let [srcfile, dstfile, ..] = args().skip(1).take(2).collect::<Vec<String>>().as_slice() {
        let x = fs::read(srcfile)?;
        let r: Vec<Room> = match serde_yaml::from_slice(&x[..]) {
            Ok(y) => y,
            Err(err) => {
                println!("{}", err);
                std::process::exit(1)
            }
        };
        // println!("{:#?}", r);
        let buf = match serde_cbor::to_vec(&r) {
            Ok(y) => y,
            Err(err) => {
                println!("{}", err);
                std::process::exit(1)
            }
        };
        std::fs::File::create(dstfile)?;
        std::fs::write(dstfile, buf)?;
    } else {
        println!("usage: convert <srcfile> <dstfile>");
    }

    Ok(())
}
