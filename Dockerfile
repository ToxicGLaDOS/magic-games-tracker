FROM rust as builder

ADD . /magic-games-tracker/

RUN cd magic-games-tracker && \
  rustup target add wasm32-unknown-unknown && \
  cargo install --locked trunk && \
  cargo build --bin server --release && \
  trunk build --release

ENV SQLX_OFFLINE=true

CMD ["/magic-games-tracker/target/release/server", "--port", "80", "--addr", "0.0.0.0", "--static-dir", "/magic-games-tracker/dist/"]
