FROM rust:1.83-bullseye AS builder

RUN mkdir /opt/build

WORKDIR /opt/build

RUN mkdir ./src && mkdir ./out && apt update && apt install -y musl-tools musl-dev build-essential clang llvm && ln -s /usr/bin/musl-gcc /usr/bin/$(arch)-linux-musl-gcc && rustup target add $(arch)-unknown-linux-musl && cargo install cargo-sbom

COPY Cargo.toml ./
COPY src/ ./src

RUN echo "app:*:1000:1000:app:/:/bin/false" >> ./out/passwd && echo "app:*:1000:" >> ./out/group
RUN cargo build -r --target=$(arch)-unknown-linux-musl && cp ./target/$(arch)-unknown-linux-musl/release/extauthz-cfzt ./out
RUN cargo sbom >> ./out/extauthz-cfzt.spdx.json

FROM scratch AS app

COPY --from=builder /bin/false /bin/false
COPY --from=builder /opt/build/out/passwd /etc/passwd
COPY --from=builder /opt/build/out/group /etc/group
COPY --from=builder /opt/build/out/extauthz-cfzt /bin/extauthz-cfzt
COPY --from=builder /opt/build/out/extauthz-cfzt.spdx.json /usr/local/share/sbom/extauthz-cfzt.spdx.json

USER app

ENTRYPOINT [ "/bin/extauthz-cfzt" ]
