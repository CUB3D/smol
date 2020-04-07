FROM rust:latest

WORKDIR /home/code

ADD ./src/ ./src/
COPY ./static ./static/
ADD ./templates ./templates/
ADD Cargo.toml .

RUN cargo build --release

CMD ["cargo", "run", "--release"]
