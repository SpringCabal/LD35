
if not gadgetHandler:IsSyncedCode() then
	return
end
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name    = "Attributes Generic",
		desc    = "Handles modification of unit attributes such as speed and reload time.",
		author  = "GoogleFrog",
		date    = "27 March 2016",
		license = "GNU GPL, v2 or later",
		layer   = -1,
		enabled = true, 
	}
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- GG.Attributes.UpdateLineOfSightMult(unitID, multiplier, [key])
-- GG.Attributes.UpdateBuildSpeedMult(unitID, multiplier, [key])
-- GG.Attributes.UpdateWeaponReloadMult(unitID, multiplier, [key])
-- GG.Attributes.UpdateMoveSpeedMult(unitID, multiplier, [key])
-- 
-- The optional key is used to combine multipliers from multiple sources.
--

local RELOAD_UPDATE_PERIOD = 3

local floor = math.floor

local spValidUnitID         = Spring.ValidUnitID
local spGetUnitDefID        = Spring.GetUnitDefID
local spGetGameFrame        = Spring.GetGameFrame

local spSetUnitBuildSpeed   = Spring.SetUnitBuildSpeed
local spSetUnitWeaponState  = Spring.SetUnitWeaponState
local spGetUnitWeaponState  = Spring.GetUnitWeaponState
local spSetUnitSensorRadius = Spring.SetUnitSensorRadius

local spGetUnitMoveTypeData    = Spring.GetUnitMoveTypeData
local spMoveCtrlGetTag         = Spring.MoveCtrl.GetTag
local spSetAirMoveTypeData     = Spring.MoveCtrl.SetAirMoveTypeData
local spSetGunshipMoveTypeData = Spring.MoveCtrl.SetGunshipMoveTypeData
local spSetGroundMoveTypeData  = Spring.MoveCtrl.SetGroundMoveTypeData

local GetMovetype = Spring.Utilities.GetMovetype

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Sensor Handling

local origUnitDefLineOfSight = {}

local function UpdateLineOfSight(unitID, sightFactor)
	local unitDefID = spGetUnitDefID(unitID)
	if not origUnitDefLineOfSight[unitDefID] then
		local ud = UnitDefs[unitDefID]
		origUnitDefLineOfSight[unitDefID] = {
			los = ud.losRadius or 0,
			airLos = ud.airLosRadius or 0,
		}
	end
	
	local state = origUnitDefLineOfSight[unitDefID]
	
	spSetUnitSensorRadius(unitID, "los",    state.los    * sightFactor)
	spSetUnitSensorRadius(unitID, "airLos", state.airLos * sightFactor)
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Build Speed Handling

local origUnitDefBuildSpeed = {}

local function UpdateBuildSpeed(unitID, speedFactor)
	local unitDefID = spGetUnitDefID(unitID)
	local ud = UnitDefs[unitDefID]
	if ud.buildSpeed == 0 then
		return
	end
	
	local unitDefID = ud.id
	if not origUnitDefBuildSpeed[unitDefID] then
		origUnitDefBuildSpeed[unitDefID] = {
			buildSpeed = ud.buildSpeed or 0,
			repairSpeed = ud.repairSpeed or 0,
			reclaimSpeed = ud.reclaimSpeed or 0,
			resurrectSpeed = ud.resurrectSpeed or 0,
			captureSpeed = ud.captureSpeed or 0,
			terraformSpeed = ud.terraformSpeed or 0,
		}
	end
	
	local state = origUnitDefBuildSpeed[unitDefID]
	
	spSetUnitBuildSpeed(unitID, 
		state.buildSpeed     * speedFactor,
		state.repairSpeed    * speedFactor,
		state.reclaimSpeed   * speedFactor,
		state.resurrectSpeed * speedFactor,
		state.captureSpeed   * speedFactor,
		state.terraformSpeed * speedFactor
	)
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Reload Time Handling

local origUnitDefReload = {}
local unitReloadPaused = {}

local function UpdatePausedReload(unitID, unitDefID, gameFrame)
	local state = origUnitDefReload[unitDefID]
	
	for i = 1, state.weaponCount do
		local w = state.weapon[i]
		local reloadState = spGetUnitWeaponState(unitID, i , 'reloadState')
		if reloadState then
			local reloadTime  = spGetUnitWeaponState(unitID, i , 'reloadTime')
			 -- set a high reload time so healthbars don't judder. NOTE: math.huge is TOO LARGE
			local newReload = 100000
			if reloadState < 0 then -- unit is already reloaded, so set unit to almost reloaded
				spSetUnitWeaponState(unitID, i, {
					reloadTime = newReload, 
					reloadState = gameFrame + RELOAD_UPDATE_PERIOD + 1
				})
			else
				local nextReload = gameFrame + (reloadState - gameFrame) * newReload/reloadTime
				spSetUnitWeaponState(unitID, i, {
					reloadTime = newReload, 
					reloadState = nextReload+RELOAD_UPDATE_PERIOD
				})
			end
		end
	end
end

local function UpdateReloadSpeed(unitID, speedFactor)
	local unitDefID = spGetUnitDefID(unitID)
	local gameFrame = spGetGameFrame()
	
	if not origUnitDefReload[unitDefID] then
		local ud = UnitDefs[unitDefID]
		origUnitDefReload[unitDefID] = {
			weapon = {},
			weaponCount = #ud.weapons,
		}
		local state = origUnitDefReload[unitDefID]
		
		for i = 1, state.weaponCount do
			local wd = WeaponDefs[ud.weapons[i].weaponDef]
			local reload = wd.reload
			state.weapon[i] = {
				reload = reload,
				burstRate = wd.salvoDelay,
				oldReloadFrames = floor(reload*30),
			}
			if wd.type == "BeamLaser" then
				state.weapon[i].burstRate = false -- beamlasers go screwy if you mess with their burst length
			end
		end
	end
	
	local state = origUnitDefReload[unitDefID]

	for i = 1, state.weaponCount do
		local w = state.weapon[i]
		local reloadState = spGetUnitWeaponState(unitID, i , 'reloadState')
		local reloadTime  = spGetUnitWeaponState(unitID, i , 'reloadTime')
		if speedFactor <= 0 then
			if not unitReloadPaused[unitID] then
				local newReload = 100000 -- set a high reload time so healthbars don't judder. NOTE: math.huge is TOO LARGE
				unitReloadPaused[unitID] = unitDefID
				if reloadState < gameFrame then -- unit is already reloaded, so set unit to almost reloaded
					spSetUnitWeaponState(unitID, i, {reloadTime = newReload, reloadState = gameFrame+RELOAD_UPDATE_PERIOD+1})
				else
					local nextReload = gameFrame+(reloadState-gameFrame)*newReload/reloadTime
					spSetUnitWeaponState(unitID, i, {reloadTime = newReload, reloadState = nextReload+RELOAD_UPDATE_PERIOD})
				end
				-- add RELOAD_UPDATE_PERIOD so that the reload time never advances past what it is now
			end
		else
			if unitReloadPaused[unitID] then
				unitReloadPaused[unitID] = nil
			end
			local newReload = w.reload/speedFactor
			local nextReload = gameFrame+(reloadState-gameFrame)*newReload/reloadTime
			if w.burstRate then
				spSetUnitWeaponState(unitID, i, {
					reloadTime = newReload, 
					reloadState = nextReload, 
					burstRate = w.burstRate/speedFactor
				})
			else
				spSetUnitWeaponState(unitID, i, {
					reloadTime = newReload, 
					reloadState = nextReload
				})
			end
		end
	end
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Movement Speed Handling

local origUnitDefSpeed = {}

local function UpdateMovementSpeed(unitID, speedFactor, accelerationFactor, turnFactor)
	local unitDefID = spGetUnitDefID(unitID)
	
	if not origUnitDefSpeed[unitDefID] then
		local ud = UnitDefs[unitDefID]
		local moveData = spGetUnitMoveTypeData(unitID)
	
		origUnitDefSpeed[unitDefID] = {
			origSpeed = ud.speed,
			origReverseSpeed = (moveData.name == "ground") and moveData.maxReverseSpeed or ud.speed,
			origTurnRate = ud.turnRate,
			origTurnAccel = ((moveData.name == "ground") and moveData.turnAccel) or ud.turnRate,
			origMaxAcc = ud.maxAcc,
			origMaxDec = ud.maxDec,
			movetype = -1,
		}
		
		local state = origUnitDefSpeed[unitDefID]
		state.movetype = GetMovetype(ud)
	end
	
	local state = origUnitDefSpeed[unitDefID]
	local decFactor = accelerationFactor
	local isSlowed = speedFactor < 1
	if isSlowed then
		-- increase brake rate to cause units to slow down to their new max speed correctly.
		decFactor = 1000
	end
	if speedFactor <= 0 then
		speedFactor = 0
		-- Set the units velocity to zero if it is attached to the ground.
		local x, y, z = Spring.GetUnitPosition(unitID)
		if x then
			local h = Spring.GetGroundHeight(x, z)
			if h and h >= y then
				Spring.SetUnitVelocity(unitID, 0,0,0)
				-- Perhaps attributes should do this:
				-- local env = Spring.UnitScript.GetScriptEnv(unitID)
				-- if env and env.script.StopMoving then
				--    Spring.UnitScript.CallAsUnit(unitID,env.script.StopMoving, hx, hy, hz)
				-- end
			end
		end
	end
	
	local turnAccelFactor = turnFactor
	if turnAccelFactor <= 0 then
		turnAccelFactor = 0
	end
	
	local turnFactor = turnAccelFactor
	if turnFactor <= 0.001 then
		turnFactor = 0.001
	end
	
	if accelerationFactor <= 0 then
		accelerationFactor = 0.001
	end
	
	if spMoveCtrlGetTag(unitID) == nil then
		if state.movetype == 0 then
			local attribute = {
				maxSpeed = state.origSpeed   * speedFactor,
				maxAcc   = state.origMaxAcc  * accelerationFactor,
			}
			spSetAirMoveTypeData(unitID, attribute)
			spSetAirMoveTypeData(unitID, attribute)
		elseif state.movetype == 1 then
			local attribute =  {
				maxSpeed        = state.origSpeed        * speedFactor,
				turnRate        = state.origTurnRate     * turnFactor,
				accRate         = state.origMaxAcc       * accelerationFactor,
				decRate         = state.origMaxDec       * accelerationFactor
			}
			spSetGunshipMoveTypeData (unitID, attribute)
		elseif state.movetype == 2 then
			local accRate = state.origMaxAcc*accelerationFactor 
			if isSlowed and accRate > speedFactor then
				-- Clamp acceleration to mitigate prevent brief speedup when executing new order
				-- 1 is here as an arbitary factor, there is no nice conversion which means that 1 is a good value.
				accRate = speedFactor 
			end 
			local attribute =  {
				maxSpeed        = state.origSpeed        * speedFactor,
				maxReverseSpeed = state.origReverseSpeed * speedFactor,
				turnRate        = state.origTurnRate     * turnFactor,
				accRate         = accRate,
				decRate         = state.origMaxDec       * decFactor,
				turnAccel       = state.origTurnAccel    * turnAccelFactor,
			}
			spSetGroundMoveTypeData(unitID, attribute)
		end
	end
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Interface Handling

local unitLos = {}
local unitBuildSpeed = {}
local unitReload = {}
local unitMoveSpeed = {}

local function UpdateMultTable(multTable, unitID, mult, key)
	if not spValidUnitID(unitID) then
		return false
	end
	if mult then
		return false
	end
	
	key = key or -1
	multTable[unitID] = multTable[unitID] or {}
	
	if mult == 1 then
		multTable[unitID][key] = nil
	else
		multTable[unitID][key] = mult
	end
	return true
end

local function UpdateParameter(unitID, mult, key, paramTable, paramFunction)
	if UpdateMultTable(paramTable, unitID, mult, key) then
		local totalFactor = 1
		for _, factor in pairs(paramTable[unitID]) do
			totalFactor = totalFactor * factor
		end
		paramFunction(unitID, totalFactor)
	end
end

local function Attribute_UpdateLineOfSightMult(unitID, mult, key)
	UpdateParameter(unitID, mult, key, unitLos, UpdateLineOfSight)
end

local function Attribute_UpdateBuildSpeedMult(unitID, mult, key)
	UpdateParameter(unitID, mult, key, unitBuildSpeed, UpdateBuildSpeed)
end

local function Attribute_UpdateWeaponReloadMult(unitID, mult, key)
	UpdateParameter(unitID, mult, key, unitReload, UpdateReloadSpeed)
end

local function Attribute_UpdateMoveSpeedMult(unitID, mult, key)
	if UpdateMultTable(unitMoveSpeed, unitID, mult, key) then
		local totalFactor = 1
		for _, factor in pairs(unitMoveSpeed[unitID]) do
			totalFactor = totalFactor * factor
		end
		-- In the future the interface could modify accel and turn rate independantly
		UpdateMovementSpeed(unitID, totalFactor, totalFactor, totalFactor)
	end
end

local function RemoveUnit(unitID)
	unitReloadPaused[unitID] = nil
	unitLos[unitID] = nil
	unitBuildSpeed[unitID] = nil
	unitReload[unitID] = nil
	unitMoveSpeed[unitID] = nil
end

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
-- Callin Handling

function gadget:Initialize()
	GG.Attributes = {
		UpdateLineOfSightMult  = Attribute_UpdateLineOfSightMult,
		UpdateBuildSpeedMult   = Attribute_UpdateBuildSpeedMult,
		UpdateWeaponReloadMult = Attribute_UpdateWeaponReloadMult,
		UpdateMoveSpeedMult    = Attribute_UpdateMoveSpeedMult,
	}
end

function gadget:GameFrame(f)
	if f % RELOAD_UPDATE_PERIOD == 1 then
		for unitID, unitDefID in pairs(unitReloadPaused) do
			UpdatePausedReload(unitID, unitDefID, f)
		end
	end
end

function gadget:UnitDestroyed(unitID)
	RemoveUnit(unitID)
end