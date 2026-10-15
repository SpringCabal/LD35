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
