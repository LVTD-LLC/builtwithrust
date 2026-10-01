FROM rust:1.98.1-bookworm AS builder
WORKDIR /app
ENV SQLX_OFFLINE=true
COPY Cargo.toml Cargo.lock ./
COPY .sqlx .sqlx
COPY src src
COPY migrations migrations
COPY assets assets
ARG BUILD_REVISION=development
ENV BUILD_REVISION=${BUILD_REVISION}
RUN cargo build --locked --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app \
    && mkdir /data && chown app:app /data
WORKDIR /app
COPY --from=builder /app/target/release/builtwithrust /usr/local/bin/builtwithrust
COPY seed/projects.json seed/projects.json
ENV BIND_ADDR=0.0.0.0:3000 \
    DATABASE_URL=sqlite:///data/builtwithrust.db?mode=rwc
USER app
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl --fail --silent http://127.0.0.1:3000/healthz || exit 1
ENTRYPOINT ["builtwithrust"]
