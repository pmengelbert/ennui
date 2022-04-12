FROM messense/rust-musl-cross:x86_64-musl as builder

ARG CARGO_VERSION=nightly
RUN apt update && apt install -y make tree
RUN rustup install "$CARGO_VERSION"
RUN rustup target add --toolchain "$CARGO_VERSION" x86_64-unknown-linux-musl

COPY src/ /home/rust/src/src
WORKDIR /home/rust/src
RUN mkdir /home/rust/src/data
COPY sample.yaml /home/rust/src/sample.yaml
COPY npcs.yaml /home/rust/src/npcs.yaml
COPY Cargo.toml /home/rust/src/Cargo.toml
COPY Cargo.lock /home/rust/src/Cargo.lock
COPY Makefile /home/rust/src/Makefile
RUN sed -Ei 's/release\//x86_64-unknown-linux-musl\/release\//g' Makefile

RUN make server

FROM scratch
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/server /server
ENTRYPOINT ["/server"]
EXPOSE 8089
