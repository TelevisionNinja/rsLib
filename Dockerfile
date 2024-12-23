FROM rust:latest
WORKDIR /rsLib
COPY ./ ./
WORKDIR /rsLib/rs_lib
CMD ["cargo", "test"]
