FROM rust:1.69.0

WORKDIR /app
COPY . .

RUN cargo install --path .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8035

EXPOSE 8035

CMD ["pixel-rate-nft-backend"]
