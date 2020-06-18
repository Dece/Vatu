# Build engine.
FROM rust:1 AS builder
COPY . /src
WORKDIR /src
RUN cargo build --release

# Run on a Lichess account.
FROM python:3
COPY . /vatu
WORKDIR /vatu/external/lichess-bot
RUN ls -l
RUN pip install -r requirements.txt
COPY --from=builder /src/target/release/vatu engines/
ENTRYPOINT ["bash"]
