all:
	cargo build

release:
	cargo build --release

pine:
	cargo build --release --target aarch64-unknown-linux-musl

push-pine: pine
	cd ./target/aarch64-unknown-linux-musl/release/ && oras push bundle.bar/u/pmengelbert/rust/ennui:$(TAG) ./ennui
