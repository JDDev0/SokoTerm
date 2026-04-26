FROM buildpack-deps:jammy AS build
LABEL authors="jddev0"

# Setup build environment

# Rust install taken from https://github.com/rust-lang/docker-rust/blob/dd106de2954f52f336c3d2c1326ae778c51830f3/stable/bookworm/Dockerfile (MIT License & Apache 2.0 License)
# Modification: Remove all arch cases apart from "amd64"

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.95.0

RUN set -eux; \
    \
    arch="$(dpkg --print-architecture)"; \
    case "$arch" in \
        'amd64') \
            rustArch='x86_64-unknown-linux-gnu'; \
            rustupSha256='4acc9acc76d5079515b46346a485974457b5a79893cfb01112423c89aeb5aa10'; \
            ;; \
        *) \
            echo >&2 "unsupported architecture (Only amd64 is supported): $arch"; \
            exit 1; \
            ;; \
    esac; \
    \
    url="https://static.rust-lang.org/rustup/archive/1.29.0/${rustArch}/rustup-init"; \
    wget --progress=dot:giga "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN set -eux; \
    apt update;

# Linux 64 bit
RUN set -eux; \
    apt install -y cmake libasound2-dev libudev-dev;

# Windows 64 bit
RUN set -eux; \
    apt install -y mingw-w64; \
    rustup target add x86_64-pc-windows-gnu;

# Linux 32 bit
RUN set -eux; \
    dpkg --add-architecture i386; \
    apt update; \
    apt install -y gcc-multilib g++-multilib libncurses5-dev:i386 libasound2-dev:i386 libudev-dev:i386; \
    rustup target add i686-unknown-linux-gnu;

# Windows 32 bit
RUN set -eux; \
    rustup target add i686-pc-windows-gnu;

# Start build
ARG CARGO_FEATURE_FLAGS=""
ARG BUILD_DATE=""
ENV BINARY_NAME="SokoTerm"

WORKDIR /app

RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=assets,target=assets \
    --mount=type=bind,source=build,target=build \
    --mount=type=bind,source=resources,target=resources \
    --mount=type=bind,source=.cargo,target=.cargo \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=build.rs,target=build.rs \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -eux

mkdir /app/release/

# Linux 64 bit
cargo build --features ${CARGO_FEATURE_FLAGS} -r
mkdir /app/release/linux_64/
cp -p ./target/release/${BINARY_NAME} /app/release/linux_64/${BINARY_NAME}
find ./target/release/ -iname 'libsteam_api*.so' -exec cp {} /app/release/linux_64/ \;

# Windows 64 bit
cargo build --features ${CARGO_FEATURE_FLAGS} --target x86_64-pc-windows-gnu -r
mkdir /app/release/windows_64/
cp -p ./target/x86_64-pc-windows-gnu/release/${BINARY_NAME}.exe /app/release/windows_64/${BINARY_NAME}.exe
find ./target/x86_64-pc-windows-gnu/release/ -iname 'steam_api*.dll' -exec cp {} /app/release/windows_64/ \;

# Linux 32 bit
PKG_CONFIG_SYSROOT_DIR=/usr/x86_64-w64-mingw32/ cargo build --features ${CARGO_FEATURE_FLAGS} --target i686-unknown-linux-gnu -r
mkdir /app/release/linux_32/
cp -p ./target/i686-unknown-linux-gnu/release/${BINARY_NAME} /app/release/linux_32/${BINARY_NAME}
find ./target/i686-unknown-linux-gnu/release/ -iname 'libsteam_api*.so' -exec cp {} /app/release/linux_32/ \;

# Windows 32 bit
cargo build --features ${CARGO_FEATURE_FLAGS} --target i686-pc-windows-gnu -r
mkdir /app/release/windows_32/
cp -p ./target/i686-pc-windows-gnu/release/${BINARY_NAME}.exe /app/release/windows_32/${BINARY_NAME}.exe
find ./target/i686-pc-windows-gnu/release/ -iname 'steam_api*.dll' -exec cp {} /app/release/windows_32/ \;
EOF

# Prepare for export
FROM scratch AS buildexport
COPY --from=build /app /
