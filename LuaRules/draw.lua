-- The unsynced LuaRules entry point.
--
-- Without this file the engine never creates the unsynced LuaRules state, and
-- as a side effect the synced Lua gadgets stop receiving GameFrame.  That
-- silently disabled every unit script animation: LuaGadgets/Gadgets/unit_script
-- wakes its sleeping animation threads from GameFrame, so Walk() and
-- Wave_Arms() ran once and then slept forever.  ToTheLastDrop-wasm ships the
-- same one-line file for the same reason.
return true
