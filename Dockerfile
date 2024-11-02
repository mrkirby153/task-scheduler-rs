FROM rust:1.81 AS base

RUN mkdir /app
WORKDIR /app

RUN cargo install cargo-chef --locked

FROM base AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder

RUN apt-get update && apt-get install -y protobuf-compiler

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release


FROM debian:bookworm-slim AS runner

RUN apt-get update && apt-get install -y curl && apt-get clean
RUN curl -fsSL https://github.com/amacneil/dbmate/releases/download/v2.21.0/dbmate-linux-amd64 -o /usr/local/bin/dbmate && chmod +x /usr/local/bin/dbmate

RUN mkdir /app
WORKDIR /app

COPY --from=builder /app/target/release/task-scheduler-rs /app/task-scheduler-rs
COPY db/ ./db
COPY ./entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh


CMD ["/app/entrypoint.sh"]

EXPOSE 50051
EXPOSE 8081