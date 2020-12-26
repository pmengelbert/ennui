MAPFILE = sample.yaml
OUTFILE = data/map.cbor

target/release/ennui: data/map.cbor
	cargo build --release

.PHONY ennui: target/release/ennui

.PHONY clean:
	rm target/release/convert || true
	rm target/release/ennui || true
	rm target/release/server || true
	rm data/map.cbor || true

target/release/convert:
	cargo build --release --bin convert

data/map.cbor: target/release/convert
	target/release/convert $(MAPFILE) data/map.cbor

target/release/server:
	cargo build --release --bin server

.PHONY convert: target/release/convert

.PHONY server: target/release/server
