FROM rust:1.83-bullseye AS builder

RUN mkdir /opt/build

WORKDIR /opt/build

RUN mkdir ./src
RUN mkdir ./out
RUN apt update && apt install -y musl-tools musl-dev build-essential clang llvm && ln -s /usr/bin/musl-gcc /usr/bin/$(arch)-linux-musl-gcc && rustup target add $(arch)-unknown-linux-musl

COPY Cargo.toml ./
COPY src/ ./src

RUN cargo build -r --target=$(arch)-unknown-linux-musl && cp ./target/$(arch)-unknown-linux-musl/release/extauthz-cfzt ./out

FROM scratch AS app

COPY --from=builder /opt/build/out/extauthz-cfzt /

ENTRYPOINT [ "./extauthz-cfzt" ]
