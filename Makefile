target/release/ennui: clean src/mapdata.rs
	cargo build --release --bin ennui

.PHONY clean:
	rm target/release/convert
	rm target/release/ennui
	rm src/mapdata.rs
	echo "pub const MAP: [u8; 0] = [];" > src/mapdata.rs

target/release/convert:
	cargo build --release --bin convert

src/mapdata.rs: target/release/convert
	target/release/convert sample.yaml src/mapdata.rs

