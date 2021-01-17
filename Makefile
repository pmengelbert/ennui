TARGET = x86_64-unknown-linux-gnu
MAPFILE = sample.yaml
NPCFILE = npcs.yaml
OUTFILE = data/map.cbor

target/release/ennui: data/map.cbor data/npc.cbor
	cargo +nightly build --release

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
	if ! test -f data/npc.cbor; then touch data/npc.cbor; fi
	cargo +nightly build --release --bin convert

data/map.cbor: data target/release/convert
	target/release/convert map $(MAPFILE) $(OUTFILE)

data/npc.cbor: data target/release/convert
	target/release/convert npc $(NPCFILE) data/npc.cbor

data:
	mkdir data || true

target/release/server: data/map.cbor
	cargo +nightly build --release --bin server

.PHONY: pi
pi: data/map.cbor
	cargo +nightly build --release --target armv7-unknown-linux-gnueabihf

.PHONY: wasmserver
wasmserver: web/node_modules data/map.cbor
	cd web && npm run build
	cd web && cp index.html dist/index.html

.PHONY: serve
serve: wasmserver
	python3 -m http.server --directory web/dist --bind 100.106.254.52 8000

web/node_modules:
	npm install

.PHONY: rebuild-map
rebuild-map:
	rm data/map.cbor
	rm target/release/server
	make target/release/server
