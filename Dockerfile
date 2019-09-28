FROM rust:1.37

WORKDIR /usr/src/
COPY . .

RUN cargo install --path .

CMD ["crud"]
