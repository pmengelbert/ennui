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
        let len = buf.len();

        let mut output = format!("pub const MAP: [u8; {}] = [\n\t", len);

        output.push_str(
            &buf.as_slice()
                .chunks(19)
                .map(|a| {
                    a.iter()
                        .map(|i| i.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                })
                .collect::<Vec<_>>()
                .join(",\n\t"),
        );
        output.push_str("\n];");

        std::fs::write(dstfile, output)?;
    } else {
        println!("usage: convert <srcfile> <dstfile>");
    }

    Ok(())
}
