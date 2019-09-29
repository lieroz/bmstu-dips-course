FROM rust:1.38

WORKDIR /usr/src/
COPY . .

RUN cargo install --path .

CMD ["crud"]
