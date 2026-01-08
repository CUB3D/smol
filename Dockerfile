FROM rust:latest

WORKDIR /home/code

HEALTHCHECK CMD curl --fail http://localhost:8080/ || exit 1

ADD ./migrations ./migrations
ADD ./src/ ./src/
ADD ./static ./static/
ADD ./templates ./templates/
ADD Cargo.toml .

RUN cargo build --release

CMD ["cargo", "run", "--release"]
