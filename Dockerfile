FROM rust AS builder
RUN mkdir /app
COPY . /app
WORKDIR /app
RUN cargo build --release


FROM ubuntu:jammy
RUN apt-get update
COPY --from=builder /app/target/release/cardgamesbot /
RUN mkdir /app

WORKDIR /app

CMD /cardgamesbot
