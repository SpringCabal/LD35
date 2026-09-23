# Parts Needed Core-WASM port

Run these commands from this directory:

```sh
just fmt     # format Rust sources
just clippy  # release WASM clippy, warnings denied
just build   # format, lint, and build both modules
just copy    # build and install both WASM artifacts
```

The unit scripts are Rust too (`rules-synced/src/unit_scripts.rs`, run through
Core-CUS). The original Lua game is `PartsNeeded.sdd`, beside this archive.
