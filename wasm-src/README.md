# Parts Needed Core-WASM port

Run these commands from this directory:

```sh
just fmt     # format Rust sources
just clippy  # release WASM clippy, warnings denied
just build   # format, lint, and build both modules
just copy    # build and install both WASM artifacts
```

The original Lua game is retained beside this port; Lua unit scripts remain the
content-side animation boundary.
