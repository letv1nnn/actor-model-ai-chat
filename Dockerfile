FROM rust:latest AS builder
WORKDIR /app

COPY backend ./backend
COPY frontend ./frontend

WORKDIR /app/backend
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/backend/target/release/chat /usr/local/bin/chat
COPY frontend /app/frontend

WORKDIR /app
CMD ["chat"]
