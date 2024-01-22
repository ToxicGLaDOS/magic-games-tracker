FROM rust as builder

ADD . /ormos/

WORKDIR /ormos

RUN echo [target.armv7-unknown-linux-gnueabihf] >> /usr/local/cargo/config.toml && \
  echo linker = \"arm-linux-gnueabihf-gcc\" >> /usr/local/cargo/config.toml && \
  apt-get update && \
  apt-get install -y gcc-arm-linux-gnueabihf && \
  rustup target add wasm32-unknown-unknown && \
  rustup target add armv7-unknown-linux-gnueabihf && \
  cargo install --locked trunk && \
  cargo build --bin server --release --target armv7-unknown-linux-gnueabihf && \
  trunk build --release

FROM arm32v7/ubuntu

COPY --from=builder /ormos/target /ormos
COPY --from=builder /ormos/dist /ormos/dist

CMD ["/ormos/armv7-unknown-linux-gnueabihf/release/server", "--port", "80", "--addr", "0.0.0.0", "--static-dir", "/ormos/dist/"]
