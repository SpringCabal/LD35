Parts Needed (LD35) with its game code in Rust, compiled to Core-WASM modules.

The rules, the unit scripts and the UI are the two modules built from
`wasm-src/` (see `wasm-src/README.md`); the archive ships no Lua gameplay,
unit scripts or widgets. The original Lua game is `PartsNeeded.sdd`.

## Differences from the Lua version

- The spirit's trail is a stream of small puffs from the body, not the Lups
  ribbons on the four Hair pieces: Lups doesn't run in the WASM UI.
- The end screen is drawn text ("Click or press R to restart"), not a Chili
  window with a Restart button: the WASM UI has no Chili.
- The ending's arriving wisps fade out after 6 s instead of 1.5 s, so they
  can reach the player first: they spawn 100-400 elmos away on each axis.
- The UI predicts a form switch as soon as space is pressed instead of
  waiting for the rules, and undoes it after 30 frames if the rules refused.

[![Join the chat at https://gitter.im/SpringCabal/LD35](https://badges.gitter.im/SpringCabal/LD35.svg)](https://gitter.im/SpringCabal/LD35?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
