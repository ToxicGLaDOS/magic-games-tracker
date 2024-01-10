FROM rust as builder

ADD . /magic-games-tracker/

RUN cd magic-games-tracker && \
  echo [target.armv7-unknown-linux-gnueabihf] >> /usr/local/cargo/config.toml && \
  echo linker = \"arm-linux-gnueabihf-gcc\" >> /usr/local/cargo/config.toml && \
  apt-get update && \
  apt-get install -y gcc-arm-linux-gnueabihf && \
  rustup target add wasm32-unknown-unknown && \
  rustup target add armv7-unknown-linux-gnueabihf && \
  cargo install --locked trunk && \
  cargo build --bin server --release --target armv7-unknown-linux-gnueabihf && \
  trunk build --release

FROM arm32v7/ubuntu

COPY --from=builder /magic-games-tracker/target /magic-games-tracker
COPY --from=builder /magic-games-tracker/dist /magic-games-tracker/dist

CMD ["/magic-games-tracker/armv7-unknown-linux-gnueabihf/release/server", "--port", "80", "--addr", "0.0.0.0", "--static-dir", "/magic-games-tracker/dist/"]
