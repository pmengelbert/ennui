

convert:
	cargo build --release --bin convert

build-map: convert
	target/release/convert sample.yaml src/mapdata.rs

all: build-map
	cargo build --release --bin ennui
