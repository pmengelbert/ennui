.PHONY: server ennui clean pi wasmserver serve rebuild-map convert build-and-push docker-build up

TARGET = x86_64-unknown-linux-gnu
MAPFILE = sample.yaml
NPCFILE = npcs.yaml
OUTFILE = data/map.cbor

target/release/ennui: data/map.cbor data/npc.cbor
	cargo +nightly build --release

ennui: target/release/ennui

clean:
	rm target/release/convert || true
	rm target/release/ennui || true
	rm target/release/server || true
	rm -rf data || true

convert: target/release/convert

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

server: target/release/server

target/release/server: data/map.cbor data/npc.cbor
	cargo +nightly build --release --bin server

pi: data/map.cbor
	cargo +nightly build --release --target armv7-unknown-linux-gnueabihf

wasmserver: web/node_modules data/map.cbor
	cd web && npm run build
	cd web && cp index.html dist/index.html

web/node_modules:
	npm install

rebuild-map:
	rm data/map.cbor || true
	rm target/release/server || true
	make target/release/server

build-and-push: docker-build
	docker push bundle.bar/u/pmengelbert/ennui:latest

docker-build:
	docker build -t bundle.bar/u/pmengelbert/ennui:latest .

up: down
	docker-compose pull ennui
	scripts/up.sh

down:
	docker-compose down
