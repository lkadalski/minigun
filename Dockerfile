FROM rust:alpine3.14 as builder

WORKDIR /app

RUN apk upgrade --update-cache --available && \
    apk add --no-cache musl-dev openssl-dev pkgconfig  && \
        rm -rf /var/cache/apk/*
# create a new empty project
COPY ./src src
COPY ./vendor vendor
COPY ./.cargo .cargo
COPY Cargo.toml Cargo.lock ./
# build with x86_64-unknown-linux-musl to make it run with alpine.
RUN cargo install --path . --target=x86_64-unknown-linux-musl --profile release
COPY /usr/local/cargo/bin/* /usr/local/bin/
RUN cargo clean -p minigun
CMD ["minigun"]
