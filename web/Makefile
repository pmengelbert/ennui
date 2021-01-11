TARGET = x86_64-unknown-linux-gnu
MAPFILE = sample.yaml
OUTFILE = data/map.cbor

.PHONY: wasmserver
wasmserver: node_modules data/map.cbor
	npm run build
	cp index.html dist/index.html

../target/release/convert:
	if ! test -f ../data/map.cbor; then touch ../data/map.cbor; fi
	cd .. && cargo build --release --bin convert

data/map.cbor: data ../target/release/convert
	../target/release/convert $(MAPFILE) $(OUTFILE)

data:
	mkdir data || true

node_modules:
	npm install
