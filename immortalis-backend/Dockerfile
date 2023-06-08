FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

FROM debian:stable-slim as immortalis-backend-api
COPY --from=builder app/target/release/immortalis-backend-api ./immortalis-backend-api
RUN apt update -y && \
    apt install curl -y && \
    apt install libpq5 -y
EXPOSE 8080
CMD ["./immortalis-backend-api"]

FROM debian:stable-slim as immortalis-backend-common
COPY --from=builder app/target/release/immortalis-backend-common ./immortalis-backend-common
RUN apt update -y && \
    apt install curl -y && \
    apt install libpq5 -y
EXPOSE 8080
CMD ["./immortalis-backend-common"]

FROM python:slim-bullseye as immortalis-Backend-archiver
COPY --from=builder app/target/release/immortalis-backend-archiver ./immortalis-backend-archiver
RUN apt update -y && \
    apt install curl -y && \
    apt install libpq5 -y && \
    curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp && \
    apt install ffmpeg -y
EXPOSE 8080
CMD ["./immortalis-backend-archiver"]

FROM python:slim-bullseye as immortalis-backend-tracker
COPY --from=builder app/target/release/immortalis-backend-tracker ./immortalis-backend-tracker
RUN apt update -y && \
    apt install curl -y && \
    apt install libpq5 -y && \
    curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp
EXPOSE 8080
CMD ["./immortalis-backend-tracker"]