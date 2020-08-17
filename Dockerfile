FROM rust:buster as test
# frontend needs nodejs to build, even for test/clippy
RUN curl -sL https://deb.nodesource.com/setup_14.x | sudo -E bash -
RUN sudo apt-get install -y nodejs

COPY rust-toolchain .
RUN rustup component add clippy
COPY backend backend/
COPY frontend frontend/
COPY Cargo.* .
RUN cd frontend; npm install; cd ..
RUN cargo clippy --all-features -- -D warnings
RUN bash -c "time cargo test"

FROM ekidd/rust-musl-builder:stable-openssl11 as build
# frontend needs nodejs to build
RUN curl -sL https://deb.nodesource.com/setup_14.x | sudo -E bash -
RUN sudo apt-get install -y nodejs

COPY rust-toolchain .
RUN rustup target add x86_64-unknown-linux-musl
ENV RUSTFLAGS="-Clinker=musl-gcc"
ENV RUST_BACKTRACE="1"
COPY backend backend/
COPY frontend frontend/
COPY Cargo.* .
RUN cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
WORKDIR /home/factoriobot/
COPY CHECKS .
COPY backend data/
COPY backend .
COPY --from=build /home/rust/src/frontend/build/ public/
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/factorio-bot-rs .
ENV PORT 8080
ENV RUST_LOG "info"
ENV RUST_BACKTRACE="1"
# factorio
EXPOSE 34197
# web
EXPOSE 8080
# rcon
EXPOSE 1234
CMD ["./factorio-bot-rs", "start", "--clients", "0", "--seed", "1785882545"]
VOLUME "/home/factoriobot/workspace"
