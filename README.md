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
- Configurable search execution: background worker search for normal play or
  deterministic per-suggestion budgets for simulation and optimization.
- Search statistics emitted through SBP `info` messages.
- Optional JSON configuration for search policy, behavior selection, and
  behavior-specific evaluator settings.

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

Frostetra uses an embedded default config from `src/config/default.json`. User
config files are applied over the embedded defaults, so small override files can
change only the fields they care about.

The config is split into three top-level sections:

- `behavior`: selects the active bot behavior.
- `search`: controls search randomness and search budgeting.
- `behaviors`: contains behavior-specific settings such as freestyle evaluator
  weights.

A minimal override can name only the starting behavior:

```json
{
  "behavior": {
    "initial": "freestyle"
  }
}
```

To override freestyle settings, provide the nested `behaviors.freestyle` object
shown in the default config. You can provide a replacement JSON file with:

```bash
cargo run --release -- --config path/to/config.json
```

Or, when launching a compiled binary:

```bash
target/release/frostetra --config path/to/config.json
```

### Search Policy And Determinism

By default, Frostetra uses background search:

```json
{
  "search": {
    "rng": {
      "mode": "entropy"
    },
    "budget": {
      "mode": "background",
      "node_limit": 3000000
    }
  }
}
```

In background mode, Frostetra searches continuously on a worker thread until the
configured node cap is reached. `suggest` returns the current best move from the
tree at the time the runner asks. This is responsive for real play, but the exact
amount of completed search can vary with timing.

For simulations, evaluator tuning, and ML optimization, use a seeded RNG and a
per-suggestion budget:

```json
{
  "search": {
    "rng": {
      "mode": "seeded",
      "seed": 12345
    },
    "budget": {
      "mode": "iterations_per_suggest",
      "iterations": 10000
    }
  }
}
```

In `iterations_per_suggest` mode, Frostetra does not start the background worker.
Each `suggest` request runs exactly the configured number of search iterations
synchronously, then returns the suggestion. `nodes_per_suggest` is also available
when you prefer a node-count budget:

```json
{
  "search": {
    "rng": {
      "mode": "seeded",
      "seed": 12345
    },
    "budget": {
      "mode": "nodes_per_suggest",
      "nodes": 50000
    }
  }
}
```

This makes move decisions reproducible for the same config, rules, start state,
piece stream, and SBP message sequence. Determinism is useful for optimizers
because it keeps evaluator weight comparisons from being polluted by timing or
ambient RNG differences.

### Multi-Threaded Search

Background mode can use multiple worker threads to search in parallel. Add a
`threads` field to the `search` section:

```json
{
  "search": {
    "threads": 4
  }
}
```

`threads` only affects `background` mode (the per-suggestion budgets are
single-threaded by design). The embedded default is 4 background workers.
Seeded background workers use independent, repeatable random streams. Because
workers update one shared DAG concurrently, the exact search path and resulting
move can still vary with scheduling; use a single worker when exact replay is
required.

Background workers publish their accumulated search statistics after each search
step by default, so `accounting_batch` defaults to 1. If you want to trade
gameplay responsiveness for higher raw background node generation, set
`accounting_batch` to a larger value:

```json
{
  "search": {
    "accounting_batch": 4
  }
}
```

Larger batches reduce bookkeeping overhead but hold the shared search read lock
for longer, so runner updates such as `play`, `board`, and `new_piece` may wait
longer.

## Protocol Capabilities

On startup, Frostetra sends an SBP `register` message with these capabilities:

- `randomizers`: `seven_bag`
- `kicksets`: `srs`, `srs_plus`
- `rot180`: supported
- `sonic_drop`: `only`, `allow`
- `spin_detection`: `none`, `t-spins`, `t-spins+`, `all`, `all+`, `all-mini`,
  `all-mini+`, `mini-only`
- `back_to_back_sources`: `quad`, `t-spin`, `t-spin-mini`, `allspin`,
  `allspin-mini`, `perfect-clear`
- `piece_stream`: supported
- `spawn_position`: supported
- `board`: supported
- `board_size`: width `4..127`, height `1..64`

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
  -> runtime::bot_session
  -> bot::BotRunner
  -> bot::Bot
  -> bot::behavior::freestyle
  -> search::dag
  -> tetris::movegen
  -> tetris::model
```

Main modules:

- `config/` owns the global bot config, embedded defaults, search policy config,
  and behavior config layout.
- `protocol/` contains SBP serde message types.
- `runtime/` owns process orchestration, message dispatch, configured bot
  sessions, optional worker-thread search, and SBP-to-bot adaptation.
- `bot/` owns bot state, behavior selection, runner state, and statistics.
- `bot/behavior/freestyle/` contains the default evaluator and feature scoring.
- `search/` contains deterministic search context/RNG plumbing and the
  transposition-aware game tree.
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

### CPU Profiling With VTune

For sampling profiles, use Intel VTune Profiler with the opt-in fixed-work
search harness. The harness runs deterministic node-budgeted search without SBP
or Criterion overhead, then prints useful-work counters.

Install VTune:

- Download the standalone VTune installer from
  [Get Intel VTune Profiler](https://www.intel.com/content/www/us/en/developer/tools/oneapi/vtune-profiler-download.html).
- Follow Intel's
  [VTune installation guide](https://www.intel.com/content/www/us/en/docs/vtune-profiler/installation-guide/2026-0/overview.html)
  for Windows or Linux.
- If the `vtune` command is not on `PATH` after installation, open an Intel
  oneAPI command prompt on Windows or source the oneAPI environment script on
  Linux.

Build the harness:

```bash
cargo build --release --features profiling --bin profile
```

Run the harness directly:

```powershell
# Windows
target\release\profile.exe search --batches 10000 --nodes 32 --board tspin
```

```bash
# Linux
target/release/profile search --batches 10000 --nodes 32 --board tspin
```

Collect a VTune hotspots profile:

```powershell
# Windows
New-Item -ItemType Directory -Force target\profiles
vtune -collect hotspots -result-dir target\profiles\vtune-search -- target\release\profile.exe search --batches 10000 --nodes 32 --board tspin
```

```bash
# Linux
mkdir -p target/profiles
vtune -collect hotspots -result-dir target/profiles/vtune-search -- target/release/profile search --batches 10000 --nodes 32 --board tspin
```

Export text reports from the VTune result:

```powershell
# Windows
vtune -report hotspots -r target\profiles\vtune-search -format csv -report-output target\profiles\vtune-search-hotspots.csv
vtune -report hotspots -r target\profiles\vtune-search -group-by source-file-path,source-line -format csv -report-output target\profiles\vtune-search-lines.csv
vtune -report summary -r target\profiles\vtune-search -report-output target\profiles\vtune-search-summary.txt
```

```bash
# Linux
vtune -report hotspots -r target/profiles/vtune-search -format csv -report-output target/profiles/vtune-search-hotspots.csv
vtune -report hotspots -r target/profiles/vtune-search -group-by source-file-path,source-line -format csv -report-output target/profiles/vtune-search-lines.csv
vtune -report summary -r target/profiles/vtune-search -report-output target/profiles/vtune-search-summary.txt
```

The `hotspots` CSV, source-line CSV, and summary text reports are intended to be
checked into a temporary profiling notes workflow or compared locally; do not
commit generated `target/profiles` output.
On non-Intel CPUs, or without VTune's sampling driver enabled, deeper
microarchitecture insights may be unavailable; the basic hotspots report is
still useful for source-level CPU time.

Useful harness options:

```text
--batches N                   Number of repeated fixed-work batches.
--nodes N                     Search nodes per batch.
--board empty|tspin|terrible  Initial board shape.
--locked                      Use the background/locked DAG configuration.
--threads N                   Configure background worker shard density.
```

Release and bench builds include line-table debug info so sampled stacks can map
back to source locations without substantially changing optimized codegen.

## License

Frostetra is descended from Cold Clear 2 by Mark Carlson (MinusKelvin). It is licensed under
the [MIT License](LICENSE.txt).
