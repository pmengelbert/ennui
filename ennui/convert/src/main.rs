use ennui::map::Room;
use ennui::player::npc::YamlPlayer;
use std::env::args;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    if let [kind, srcfile, dstfile, ..] = args().skip(1).take(3).collect::<Vec<String>>().as_slice()
    {
        match kind.as_str() {
            "npc" => {
                let x = fs::read(srcfile)?;
                let r: Vec<YamlPlayer> = match serde_yaml::from_slice(&x[..]) {
                    Ok(y) => y,
                    Err(err) => {
                        eprintln!("{}", err);
                        eprintln!("in file {} on line number {}", file!(), line!());

                        std::process::exit(1)
                    }
                };
                // eprintln!("{:#?}", r);
                eprintln!("in file {} on line number {}", file!(), line!());

                let buf = match serde_cbor::to_vec(&r) {
                    Ok(y) => y,
                    Err(err) => {
                        eprintln!("{}", err);
                        eprintln!("in file {} on line number {}", file!(), line!());

                        std::process::exit(1)
                    }
                };
                std::fs::File::create(dstfile)?;
                std::fs::write(dstfile, buf)?;
            }
            _ => {
                let x = fs::read(srcfile)?;
                let r: Vec<Room> = match serde_yaml::from_slice(&x[..]) {
                    Ok(y) => y,
                    Err(err) => {
                        eprintln!("{}", err);
                        eprintln!("in file {} on line number {}", file!(), line!());

                        std::process::exit(1)
                    }
                };
                // eprintln!("{:#?}", r);
                eprintln!("in file {} on line number {}", file!(), line!());

                let buf = match serde_cbor::to_vec(&r) {
                    Ok(y) => y,
                    Err(err) => {
                        eprintln!("{}", err);
                        eprintln!("in file {} on line number {}", file!(), line!());

                        std::process::exit(1)
                    }
                };
                std::fs::File::create(dstfile)?;
                std::fs::write(dstfile, buf)?;
            }
        }
    } else {
        eprintln!("usage: convert [map/npc] <srcfile> <dstfile>");
        eprintln!("in file {} on line number {}", file!(), line!());
    }

    Ok(())
}
