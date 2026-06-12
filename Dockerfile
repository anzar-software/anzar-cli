ARG RUST_VERSION=1.90.0
ARG ALPINE_VERSION=3.22
ARG APP_NAME=anzar
ARG BIN_NAME=api

# Build stage
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
ARG BIN_NAME
WORKDIR /app

RUN apk add --no-cache musl-dev perl make

# Stub cli so the workspace resolver doesn't complain
RUN mkdir -p crates/cli/src && echo "fn main() {}" > crates/cli/src/main.rs

RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=crates/api/Cargo.toml,target=crates/api/Cargo.toml \
    --mount=type=bind,source=crates/shared/Cargo.toml,target=crates/shared/Cargo.toml \
    --mount=type=bind,source=crates/cli/Cargo.toml,target=crates/cli/Cargo.toml \
    --mount=type=bind,source=crates/api/src,target=crates/api/src \
    --mount=type=bind,source=crates/shared/src,target=crates/shared/src \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release -p api && \
    strip ./target/release/$BIN_NAME && \
    cp ./target/release/$BIN_NAME /bin/$APP_NAME

RUN echo "appuser:x:10001:10001::/app:/sbin/nologin" > /etc/passwd.minimal && \
    echo "appuser:x:10001:" > /etc/group.minimal

# Execution stage
FROM scratch AS final
ARG APP_NAME=anzar
WORKDIR /app

COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=build /etc/passwd.minimal /etc/passwd
COPY --from=build /etc/group.minimal /etc/group

COPY --from=build /bin/$APP_NAME ./$APP_NAME
COPY --chown=10001:10001 app/configuration /app/configuration

USER 10001:10001

ENV ENV=prod
EXPOSE 3000
ENTRYPOINT ["./anzar"]

