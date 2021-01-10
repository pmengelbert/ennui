TARGET = x86_64-unknown-linux-gnu
MAPFILE = sample.yaml
OUTFILE = data/map.cbor

target/release/ennui: data/map.cbor
	cargo build --release

.PHONY: ennui
ennui: target/release/ennui

.PHONY: clean
clean:
	rm target/release/convert || true
	rm target/release/ennui || true
	rm target/release/server || true
	rm -rf data || true

target/release/convert:
	if ! test -f data/map.cbor; then touch data/map.cbor; fi
	cargo build --release --bin convert

data/map.cbor: data target/release/convert
	target/release/convert $(MAPFILE) $(OUTFILE)

data:
	mkdir data || true

target/release/server: data/map.cbor
	cargo build --release --bin server

.PHONY: pi
pi: data/map.cbor
	cargo build --release --target armv7-unknown-linux-gnueabihf

.PHONY: wasmserver
wasmserver: node_modules data/map.cbor
	npm run build
	cp index.html dist/index.html

node_modules:
	npm install
