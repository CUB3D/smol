FROM public.ecr.aws/docker/library/rust:latest AS build

WORKDIR /home/code

ADD ./migrations ./migrations
ADD ./src/ ./src/
ADD ./templates ./templates/
ADD Cargo.lock .
ADD Cargo.toml .

RUN cargo build --release


FROM public.ecr.aws/docker/library/rust:latest

RUN apt-get install -y curl
HEALTHCHECK --interval=30s --timeout=3s CMD curl -X HEAD -f http://localhost:8080/ || exit 1

WORKDIR /srv

COPY --from=build /home/code/target/release/smol /srv/smol
ADD ./static ./static/

CMD ["/srv/smol"]