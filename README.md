# Parts Needed WASM

Parts Needed (LD35) with Rust game code compiled to Core-WASM. The archive contains no Lua gameplay, unit scripts or widgets.

Use the [SpringCabal development repository](https://github.com/gajop/springcabal-dev) for engine setup, launching, builds and verification.

Differences from the Lua version:

- The spirit trail uses small puffs instead of Lups ribbons.
- The end screen uses drawn text, with click or R to restart and Ctrl-Q to return to the menu.
- Form changes are predicted immediately and undone after 30 frames if refused by the rules.
- Conflicting cage triggers use their level-table order; Lua's unit-table order can vary.
- Wisps arriving during the ending fade after six seconds so they can reach the player; Lua fades them after 1.5 seconds.

[![Join the chat at https://gitter.im/SpringCabal/LD35](https://badges.gitter.im/SpringCabal/LD35.svg)](https://gitter.im/SpringCabal/LD35?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

## Scoped SDK port

This worktree builds the Wasm game with `spring-sdk` and the shared addon/CUS
host. Helpers receive `&Spring`; queued commands and their continuations get
fresh engine access. Unit-script tasks use `UnitCtx::with_spring` for each
operation. UI sound variation retains the former module-local PCG stream.
This port does not build native game modules.

Verification on the SDK test engine:

- Linux and Wine pass all 57 main assertions, four ending assertions, four
  restart assertions, five same-pass assertions and the UI handshake.
- Strict Clippy passes with default/all features; four shared Rust tests pass.
  The headless-only UI build is checked by the actual integration run.
- Unit-script source comparison passes. Live Lua/Wasm comparison passes for
  3,498 transform samples: largest delta 0.00003384, below the existing 0.0001
  limit. Walking, hovering, idle animation and assembled body parts are kept.
- The six-shot visual scenario passes the non-black and settled-camera
  assertions on NVIDIA. The parent repository stores reviewed NVIDIA goldens.
- NPC sprites track creation, destruction and LOS events instead of polling
  every unit, including during the ending.

Use the parent development repository's `just verify partsneeded` and
`just verify unit-scripts partsneeded` commands, selecting the engine with
`SPRING_ENGINE_DIR`.
