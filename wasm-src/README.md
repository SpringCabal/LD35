# Parts Needed Core-WASM modules

Run these commands from this directory:

```sh
just fmt     # format Rust sources
just clippy  # release WASM clippy, warnings denied
just build   # format, lint, and build both modules
just copy    # build and install both WASM artifacts
```

- `rules-synced/` builds `LuaRules/wasm/partsneeded-rules.wasm`: the synced
  gadgets (`src/gadgets/`) and the unit scripts of every unit
  (`src/unit_scripts/`, hosted by the engine's `CusHost`). The gadgets
  attach the scripts and call them directly through
  `with_cus_module_or_defer`.
- `ui/` builds `LuaUI/wasm/partsneeded-ui.wasm`: the widgets (`src/widgets/`)
  for input, camera, HUD, sound and the screen effects.
- `shared/` holds what both agree on: body parts, the endings and the two
  messages between them, `Status` (rules to UI) and `Command` (UI to rules).
  Its tests run on the host: `cargo test -p partsneeded-shared --target
  x86_64-unknown-linux-gnu`.

The parts every SpringCabal game shares come from the `springcabal` crate
(`gajop/springcabal-rs`): the message codec, the test suite tally, the looping
music, the screenshot schedule, the movement keys and Ctrl+Q.

The `headless-integration`, `e2e` and `unit-script-probe` features add the
test gadgets and widgets the verification suites use; they are never part of
the shipped modules.
