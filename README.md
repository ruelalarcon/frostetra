# Frostetra

> A Modern Tetris bot descended from
> [Cold Clear 2](https://github.com/MinusKelvin/cold-clear-2) by MinusKelvin. It
> speaks the
> [Stacker Bot Protocol](https://github.com/ruelalarcon/stacker_bot_protocol)
> over stdin/stdout, and is designed to be launched via an SBP bot runner
> such as [Minorail](https://github.com/ruelalarcon/minorail).

The bot maintains its own Tetris model, generates legal placements with kicksets
such as SRS or SRS+, searches a transposition-aware game tree, and returns placement
suggestions to the runner. The current default behavior is a configurable
freestyle evaluator, currently running the same weights as Cold Clear 2.

## Features

- SBP JSON-lines protocol over standard input and output.
- Seven-bag piece stream tracking and speculative search once bag state is
  known.
- SRS and SRS+ kick support.
- 180 rotation support.
- Sonic drop rule support for `only` and `allow`.
- Configurable spawn position.
- Column-major bitboard board representation.
- Worker-thread search loop with search statistics emitted through SBP `info`
  messages.
- Optional JSON configuration for the current freestyle behavior.

## Requirements

- Rust stable toolchain.
- An SBP-compatible runner or frontend.

For a local runner, you can use
[Minorail](https://github.com/ruelalarcon/minorail), a Tetris bot runner and
visualizer that can launch SBP bots. It also provides a websocket API for
retrieving suggestions, allowing you to connect real games to SBP bots.

## Build

```bash
cargo build --release
```

The release binary is written to:

```text
target/release/frostetra
```

On Windows, the binary is:

```text
target/release/frostetra.exe
```

For development builds:

```bash
cargo build
```

## Run With Minorail

Build Frostetra first, then pass the compiled binary path to [Minorail](https://github.com/ruelalarcon/minorail).

From a Minorail checkout on Windows:

```powershell
python run.py "path\to\frostetra\target\release\frostetra.exe"
```

From a Unix-like shell:

```bash
python run.py "path/to/frostetra/target/release/frostetra"
```

Useful Minorail modes:

```bash
python run.py "path/to/frostetra/target/release/frostetra" --web
python run.py "path/to/frostetra/target/release/frostetra" --headless
python run.py "path/to/frostetra/target/release/frostetra" --games 100 --headless
```

Frostetra itself reads SBP messages from stdin and writes SBP messages to
stdout. It does not draw a board, own a game loop, or provide a human-facing UI;
those responsibilities belong to the runner.

## Configuration

Frostetra uses an embedded default bot config from `src/bot/default_config.json`.
The config selects the initial behavior and scopes behavior-specific settings
under that behavior name. A minimal override can name only the starting
behavior:

```json
{
  "initial_behavior": "freestyle"
}
```

User config files are applied over the embedded defaults, so a config containing
only `"initial_behavior"` still uses the default freestyle settings. To override
freestyle settings, provide the full nested `freestyle` object shown in the
default config. You can provide a replacement JSON file with:

```bash
cargo run --release -- --config path/to/config.json
```

Or, when launching a compiled binary:

```bash
target/release/frostetra --config path/to/config.json
```

## Protocol Capabilities

On startup, Frostetra sends an SBP `register` message with these capabilities:

- `randomizers`: `seven_bag`
- `kicksets`: `srs`, `srs_plus`
- `rot180`: supported
- `sonic_drop`: `only`, `allow`
- `piece_stream`: supported
- `spawn_position`: supported

The normal SBP flow is:

```text
runner -> rules
bot    -> ready
runner -> start
runner -> suggest
bot    -> suggestion
runner -> play
runner -> new_piece
```

Frostetra may also emit SBP `info` messages. Search statistics are sent under
the `search` topic, and runtime notes are sent under the `log` topic.

## Architecture

Frostetra is a single Rust crate with both a library target and a binary target.

```text
Frontend / runner
  -> SBP JSON over stdin/stdout
  -> src/main.rs
  -> runtime::dispatcher
  -> runtime::worker_pool
  -> bot::Bot
  -> bot::behavior::freestyle
  -> search::dag
  -> tetris::movegen
  -> tetris::model
```

Main modules:

- `protocol/` contains SBP serde message types.
- `runtime/` owns process orchestration, message dispatch, worker threads, and
  SBP-to-bot adaptation.
- `bot/` owns bot state, behavior selection, configuration, and statistics.
- `bot/behavior/freestyle/` contains the default evaluator and feature scoring.
- `search/` contains the transposition-aware game tree.
- `tetris/model/` contains board, piece, placement, rotation, spin, and rule
  types.
- `tetris/movegen/` contains legal placement generation and kick handling.
- `tetris/randomizer/` contains piece stream and seven-bag inference.

## Development

Run tests:

```bash
cargo test
```

Run the move generation benchmark:

```bash
cargo bench movegen
```

Run all benchmarks:

```bash
cargo bench
```

Frostetra can expose a Puffin profiler server when built with the optional
`puffin_http` feature:

```bash
cargo run --features puffin_http -- --profile
```

By default, the profiler listens on Puffin's default HTTP port.

## License

Frostetra is descended from Cold Clear 2 by Mark Carlson (MinusKelvin). It is licensed under
the [MIT License](LICENSE.txt).
