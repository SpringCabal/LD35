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
  (`src/unit_scripts/`, run through Core-CUS). The gadgets attach the
  scripts and call them directly through `with_cus_module`.
- `ui/` builds `LuaUI/wasm/partsneeded-ui.wasm`: the widgets (`src/widgets/`)
  for input, camera, HUD, sound and the screen effects.
- `shared/` holds what both agree on: body parts and the two messages
  between them, `Status` (rules to UI) and `Command` (UI to rules).

The `headless-integration`, `e2e` and `unit-script-probe` features add the
test gadgets and widgets the verification suites use; they are never part of
the shipped modules.

`src/restart.rs` in the UI embeds the start script the in-game restart
reloads, a copy of `launch/games/partsneeded/wasm.txt` in springcabal-games
(the archive has to be self-contained).
