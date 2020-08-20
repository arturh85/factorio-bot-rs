FROM rust:buster as test
# frontend needs nodejs to build, even for test/clippy
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash -
RUN apt-get install -y nodejs

COPY rust-toolchain .
RUN rustup component add clippy
COPY backend backend/
COPY frontend frontend/
COPY Cargo.* ./
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
COPY Cargo.* ./
RUN sudo chown rust.rust . -R; cargo build --release --target=x86_64-unknown-linux-musl

FROM scratch
WORKDIR /home/factoriobot/
COPY mod mod/
COPY Settings.toml .
COPY Rocket.toml .
COPY --from=build /home/rust/src/frontend/dist/ public/
COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/factorio-bot-backend .
ENV RUST_LOG "info"
ENV RUST_BACKTRACE="1"
# factorio
EXPOSE 34197
# web
EXPOSE 7123
# rcon
EXPOSE 1234
CMD ["./factorio-bot-backend", "start", "--clients", "0", "--seed", "1785882545"]
VOLUME "/home/factoriobot/workspace"
