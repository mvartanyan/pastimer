# syntax=docker/dockerfile:1
FROM rust:1.93.1-bookworm AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY public ./public
ARG CARGO_BUILD_JOBS=4
RUN --mount=type=secret,id=proxy_ca \
    if [ -f /run/secrets/proxy_ca ]; then \
      export CARGO_HTTP_CAINFO=/run/secrets/proxy_ca SSL_CERT_FILE=/run/secrets/proxy_ca; \
    fi; \
    cargo build --release --locked --bin pastimer --bin verify-audit

FROM debian:bookworm-slim
RUN mkdir -p /data /etc/ssl/certs && chown 10001:10001 /data
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /build/target/release/pastimer /build/target/release/verify-audit /usr/local/bin/
ENV HOST=0.0.0.0 PORT=3000 PASTIMER_ENV=production DATABASE_PATH=/data/pastimer.sqlite
USER 10001:10001
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/pastimer"]
