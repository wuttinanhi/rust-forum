FROM rust:1.88.0-bookworm AS build-env

RUN apt install -y libpq-dev

WORKDIR /app
COPY . /app

RUN cargo build --release --bins

# FROM gcr.io/distroless/cc-debian12
FROM debian:12-slim

RUN apt update && apt install -y ca-certificates

WORKDIR /app

# COPY --from=build-env /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu

COPY --from=build-env /usr/lib/x86_64-linux-gnu/lib* /usr/lib/x86_64-linux-gnu/
COPY --from=build-env /app/target/release/rustforum /app/rustforum
COPY --from=build-env /app/target/release/reset_db /app/reset_db
COPY --from=build-env /app/templates /app/templates
COPY --from=build-env /app/migrations /app/migrations

# VOLUME [ "/app/static" ]

CMD ["/app/rustforum"]
