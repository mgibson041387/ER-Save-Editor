FROM rust:1-slim AS build
ARG BASE_PATH=/
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
RUN curl -sL https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar xz -C /usr/local/bin

COPY . .
RUN trunk build --release --public-url "${BASE_PATH}/"

FROM nginx:alpine
ARG BASE_PATH=/
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf.template /etc/nginx/templates/default.conf.template
ENV BASE_PATH=${BASE_PATH}
EXPOSE 80
