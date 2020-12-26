TARGET = x86_64-unknown-linux-gnu
MAPFILE = sample.yaml
OUTFILE = data/map.cbor

target/release/ennui: data/map.cbor
	cargo build --release --target $(TARGET)

.PHONY ennui: target/release/ennui

.PHONY clean:
	rm target/release/convert || true
	rm target/release/ennui || true
	rm target/release/server || true
	rm -rf data || true

target/release/convert: datafile
	cargo build --release --bin convert

data/map.cbor: convert
	target/release/convert $(MAPFILE) $(OUTFILE)

data:
	mkdir data || true

.PHONY datafile: data
	touch data/map.cbor

target/release/server:
	cargo build --release --bin server

.PHONY convert: target/release/convert

.PHONY server: target/release/server
