FROM rust:1.36.0-slim-stretch as builder
RUN mkdir /src
WORKDIR /src
COPY . .
RUN cargo build --release

FROM debian:stretch-slim
EXPOSE 3000
ENTRYPOINT [ "/app/server" ]
RUN mkdir /app
WORKDIR /app
COPY --from=builder /src/target/release/server /app/server
