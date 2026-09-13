local modinfo = {
	name = "Parts Needed",
	shortname = "Parts Needed",
	game = "Parts Needed",
	shortgame = "Parts Needed",
	version = "$VERSION",
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
