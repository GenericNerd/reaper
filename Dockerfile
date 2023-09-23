FROM lukemathwalker/cargo-chef:latest-rust-1.72.0 AS chef
WORKDIR /usr/src/reaper

FROM chef AS prepare
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=prepare /usr/src/reaper/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM rust AS runtime
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=build /usr/src/reaper/target/release/reaper .
CMD ["./reaper"]