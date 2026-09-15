local modinfo = {
	name = "Parts Needed WASM",
	shortname = "PartsNeededWASM",
	game = "Parts Needed WASM",
	shortgame = "Parts Needed WASM",
	version = "0.1.0-wasm",
	description = "Ludum Dare 35 entry",
	modtype = "1",
	-- Single-player only: without this the engine binds a UDP socket and warns
	-- that nobody else can reach it. Nothing here is ever hosted or joined.
	onlyLocal = true,
	depend = {
		"cursors.sdz",
	}
}

return modinfo
