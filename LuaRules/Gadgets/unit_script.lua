-- Enables Lua unit scripts by including the gadget from springcontent.sdz.
--
-- Core-WASM owns the gameplay callins, but the models' animation and piece
-- visibility still live in Scripts/*.lua.  Without this gadget none of those
-- scripts run: script.Create() never builds the skeleton, so every unit keeps
-- its default model pose (the gate reads as a cross, the wisp T-poses) and
-- SetWasmPieceMask calls from the WASM module land nowhere.

UNITSCRIPT_DIR = "scripts/"

return include("luagadgets/gadgets/unit_script.lua")
