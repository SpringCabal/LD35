--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
   return {
      name      = "Game modes",
      desc      = "Loads game modes from modoptions and sets the appropriate gamerules",
      author    = "gajop",
      date      = "15.04.2016.",
      license   = "Public Domain",
      layer     = 0,
      enabled   = true
   }
end

local modOptions

function LoadGameMode(modeName)
	local modeValue = Spring.GetGameRulesParam(modeName)
	if modeValue == nil then
		modeValue = modOptions[modeName] or 0
		Spring.SetGameRulesParam(modeName, modeValue)
	end
	return modeValue
end

function SafeSetGameMode(gameMode)
	-- three valid game modes:
	-- develop -> camera isn't locked and input isn't grabbed, units aren't spawned initially
	-- test    -> camera is locked and input is grabbed, units are spawned, BUT it's possible to use the console and switch back to develop mode
	-- play    -> all the same as test except it's not possible to switch to develop
	if gameMode ~= "develop" and gameMode ~= "test" and gameMode ~= "play" then
		gameMode = "develop"
	end
	if Spring.GetGameRulesParam("gameMode") ~= gameMode then
		Spring.SetGameRulesParam("gameMode", gameMode)
	end
end

function gadget:Initialize()
	if Spring.GetGameRulesParam("gameMode") == nil then
		modOptions = Spring.GetModOptions()
		local gameMode = modOptions.gamemode
		SafeSetGameMode(gameMode)
	end
end

function gadget:GameFrame()
	local gameMode = Spring.GetGameRulesParam("gameMode")
	SafeSetGameMode(gameMode)
end
