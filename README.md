Vatu
====

Dumb chess engine written in Rust for fun.



Features
--------

- [UCI][cpw-uci]
- Barely optimized board, position and moves representations
- Simple [negamax][cpw-negamax] for node evaluation
- [Alpha-beta][cpw-ab] search tree pruning for speeding searches

[cpw-uci]: https://www.chessprogramming.org/UCI
[cpw-negamax]: https://www.chessprogramming.org/Negamax
[cpw-ab]: https://www.chessprogramming.org/Alpha-Beta

Last time I checked it ran approximately at 10000 nps.

Thanks to UCI the bot can run with most compatible software; [Cutechess][cc] has
been used for testing.

[cc]: https://github.com/cutechess/cutechess



Usage
-----

### Build

With Cargo:

```bash
cargo build --release
```

With Docker, to avoid setting up a Rust toolchain:

```bash
docker build -f res/docker/Dockerfile -t vatu-builder --target builder .
docker create vatu-builder  # Returns a container ID.
docker cp <id>:/src/target/release/vatu .
docker rm <id>
docker rmi vatu-builder
```

### Run

If you built it with Cargo, the binary is in `target/release`.

```bash
./vatu
```

To run your own instance of the bot on Lichess (why would you do that?), create
a bot account and get an OAuth token. Then using the full Docker image:

```bash
# Fetch the lichess-bot submodule.
git submodule update --init --recursive
# Copy the config.yml template to your own copy and modify it.
cp external/lichess-bot/config.yml.example /tmp/vatu-config/config.yml
# Build the image. Make sure config.yml is not there to not embed it!
docker build -f res/docker/Dockerfile -t vatu .
# Run with the config folder mounted at /config.
docker run -v /tmp/vatu-config:/config -ti vatu
```



TODO
----

- Support time constraints
- Proper unmake mechanism instead of allocating boards like there is no tomorrow
- Precompute some pieces moves, maybe
- Transposition table that does not actually slows search down
- Check Zobrist hashes for previous point
- Actual bitboard
- Multithreading (never)
