FROM rust AS builder

#copy common project
COPY ./immortalis-backend-common /immortalis/immortalis-backend-common

# init empty project to generate a main.rs file
WORKDIR /immortalis
RUN cargo new --bin immortalis-backend-tracker
WORKDIR /immortalis/immortalis-backend-tracker

# copy cargo files and build to create a layer containing the installed dependencies. Remove generated .rs files after
COPY ./immortalis-backend-tracker/Cargo.lock ./Cargo.lock
COPY ./immortalis-backend-tracker/Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm ./src/*.rs 

# copy actual source and other necessities
COPY ./immortalis-backend-tracker/src ./src

# remove previously generated binary, not doing so may cause default code to be executed instead (cargo converts "-" to "_")
RUN ls ./target/release/deps &&rm ./target/release/deps/immortalis_backend_tracker*


# build with the actual code
RUN cargo build --release

FROM python:slim-bullseye

COPY --from=builder immortalis/immortalis-backend-tracker/target/release/immortalis-backend-tracker ./immortalis-backend-tracker

RUN apt update -y && \
    apt install curl -y && \
    apt install libpq5 -y && \
    curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp && \
    apt install ffmpeg -y

EXPOSE 8080
CMD ["./immortalis-backend-tracker"]