function widget:GetInfo()
  return {
    name      = "Ambient sound",
    desc      = "",
    author    = "gajop",
    date      = "tomorrow",
    license   = "GPL-v2",
    layer     = 1001,
    enabled   = true,
  }
end

local wispDefID = UnitDefNames["wisp"].id
local wispID = nil

local lastSoundTime
local cooldown = 3

local sounds = {
	moan = {
		AMBIENT_COOLDOWN_MIN = 5,
		AMBIENT_COOLDOWN_MAX = 50,
		MIN_VOLUME = 0.1,
		MAX_VOLUME = 1,
		sound = "sounds/moan.00.ogg",
	},
	wind = {
		AMBIENT_COOLDOWN_MIN = 15,
		AMBIENT_COOLDOWN_MAX = 60,
		MIN_VOLUME = 0.1,
		MAX_VOLUME = 1,
		sound = "sounds/wind.01.ogg",
	},
	drip = {
		AMBIENT_COOLDOWN_MIN = 3,
		AMBIENT_COOLDOWN_MAX = 45,
		MIN_VOLUME = 0.2,
		MAX_VOLUME = 1,
		sound = "sounds/drip.01.ogg",
	},
}

local MIN_COORD_DISTANCE = 100
local MAX_COORD_DISTANCE = 5000

function widget:Update()
	if Spring.GetGameRulesParam("gameMode") ~= "develop" and 
	Spring.GetGameRulesParam("has_ears") == 1 then
		local time = os.clock()
		for _, sound in pairs(sounds) do
			sound.lastSoundTime = sound.lastSoundTime or time
			sound.cooldown = sound.cooldown or sound.AMBIENT_COOLDOWN_MIN + math.random() * (sound.AMBIENT_COOLDOWN_MAX - sound.AMBIENT_COOLDOWN_MIN)
			if time - sound.lastSoundTime > sound.cooldown then
				sound.lastSoundTime = time
				sound.cooldown = math.random() * (sound.AMBIENT_COOLDOWN_MAX - sound.AMBIENT_COOLDOWN_MIN) + sound.AMBIENT_COOLDOWN_MIN
				
				local ux, uy, uz = Spring.GetUnitViewPosition(wispID)
				local dx = 2 * (0.5 - math.random()) * (MAX_COORD_DISTANCE - MIN_COORD_DISTANCE)
				if dx > 0 then
					dx = dx + MIN_COORD_DISTANCE
				else
					dx = dx - MIN_COORD_DISTANCE
				end
				local dz = 2 * (0.5 - math.random()) * (MAX_COORD_DISTANCE - MIN_COORD_DISTANCE)
				if dz > 0 then
					dz = dz + MIN_COORD_DISTANCE
				else
					dz = dz - MIN_COORD_DISTANCE
				end
				local volume = math.random() * (sound.MAX_VOLUME - sound.MIN_VOLUME) + sound.MIN_VOLUME
				local soundFile = sound.sound
				--Spring.Echo("Playing: ", soundFile, volume, ux + dx, uy, uz + dz)
				--Spring.PlaySoundFile(soundFile, volume, ux + dx, uy, uz + dz)
				Spring.PlaySoundFile(soundFile, volume)
			end
		end
	end
end 

function widget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == wispDefID then
		wispID = unitID
	end
end

function widget:UnitDestroyed(unitID)
	if wispID == unitID then
		wispID = nil
	end
end


function widget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		widget:UnitCreated(unitID, unitDefID)
	end
end
