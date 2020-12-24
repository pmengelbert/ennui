MAPFILE = sample.yaml

target/release/ennui: src/mapdata.rs
	cargo build --release --bin ennui

.PHONY clean:
	rm target/release/convert || true
	rm target/release/ennui || true
	rm src/mapdata.rs || true
	echo "pub const MAP: [u8; 0] = [];" > src/mapdata.rs

target/release/convert:
	cargo build --release --bin convert

src/mapdata.rs: target/release/convert
	target/release/convert $(MAPFILE) src/mapdata.rs

