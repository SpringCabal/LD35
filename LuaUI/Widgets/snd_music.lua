function widget:GetInfo()
  return {
    name      = "Music for dummies",
    desc      = "",
    author    = "ashdnazg, gajop",
    date      = "yesterday",
    license   = "GPL-v2",
    layer     = 1001,
    enabled   = true,
  }
end

local VOLUME = 1
local BUFFER = 0.015

local playingTime = 0
local dtTime = 0
local trackTime
local startedPlaying = false

local musicFile = "sounds/music.ogg"
local musicFileMuffled = "sounds/muffled.ogg"

local oldHasEars

local function StartPlaying()
    playingTime = 0
    if not startedPlaying and Spring.GetGameRulesParam("gameMode") ~= "develop" then
		local file, volume
		if Spring.GetGameRulesParam("has_ears") == 1 then
			file = musicFile
			volume = 1
		else
			file = musicFileMuffled
			volume = 0.6
		end
		if Spring.GetGameRulesParam("gameMode") ~= "play" then
			Spring.Echo("Playing: " .. tostring(file))
		end
		Spring.PlaySoundStream(file, VOLUME)
        _, trackTime = Spring.GetSoundStreamTime()
		startedPlaying = true
    end
end

local function StopPlaying()
	startedPlaying = false
	Spring.StopSoundStream()
end

function widget:Initialize()
    if not musicFile then
        widgetHandler:RemoveWidget()
        return
    end
	oldHasEars = Spring.GetGameRulesParam("has_ears") or 0
end

function widget:GameStart()
    StartPlaying()
end

function widget:Update(dt)
	local newEars = Spring.GetGameRulesParam("has_ears") or 0
	if oldHasEars ~= newEars then
		oldHasEars = newEars
		StopPlaying()
		StartPlaying()
	end
	if startedPlaying and Spring.GetGameRulesParam("gameMode") == "develop" then
		StopPlaying()
	end
    if startedPlaying then
        playingTime = playingTime + dt
        --playingTime = Spring.GetSoundStreamTime()
        if playingTime > trackTime - BUFFER then
            StopPlaying()
            StartPlaying()
        end
	else
        StartPlaying()
    end
end 

function widget:Shutdown()
    Spring.StopSoundStream()
end
