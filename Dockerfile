# Stage 1
FROM rust:1.69

WORKDIR /app

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release

# Stage 2
FROM rust:1.69-slim-buster

WORKDIR /app

COPY ./.env ./.env
COPY --from=0 /app/target/release/pixel-rate-nft-backend .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8035

EXPOSE 8035

CMD ["./pixel-rate-nft-backend"]
