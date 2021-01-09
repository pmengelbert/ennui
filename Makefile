TARGET = x86_64-unknown-linux-gnu
MAPFILE = sample.yaml
OUTFILE = data/map.cbor

target/release/ennui: data data/map.cbor
	cargo build --release --target $(TARGET)

.PHONY: ennui
ennui: target/$(TARGET)/release/ennui

.PHONY: clean
clean:
	rm target/release/convert || true
	rm target/$(TARGET)/release/ennui || true
	rm target/$(TARGET)/release/server || true
	rm -rf data || true

target/release/convert: datafile
	cargo build --release --bin convert

data/map.cbor: convert
	target/release/convert $(MAPFILE) $(OUTFILE)

data:
	mkdir data || true

.PHONY: datafile
datafile: data
	touch data/map.cbor

target/release/server: data/map.cbor
	cargo build --release --bin server --target $(TARGET)

.PHONY: convert
convert: target/release/convert

.PHONY: server
server: target/$(TARGET)/release/server

.PHONY: pi
pi: data/map.cbor
	cargo build --release --target armv7-unknown-linux-gnueabihf

.PHONY: wasmserver
wasmserver:
	npm run build
	cp index.html dist/index.html
