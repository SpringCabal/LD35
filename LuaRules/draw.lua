-- The unsynced LuaRules entry point.
--
-- Without this file the engine never creates the unsynced LuaRules state, and
-- as a side effect the synced Lua gadgets stop receiving GameFrame. The
-- gameplay and CUS unit scripts are still driven by the Core-WASM module.
return true
