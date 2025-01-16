FROM rust:1 AS build-env

RUN apt install -y libpq-dev

WORKDIR /app
COPY . /app

RUN cargo build --release

#FROM gcr.io/distroless/cc-debian12
FROM debian:12-slim
COPY --from=build-env /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
COPY --from=build-env /app/target/release/rustforum /
COPY --from=build-env /app/templates /templates
CMD ["./rustforum"]
