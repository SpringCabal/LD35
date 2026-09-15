function gadget:GetInfo()
	return {
		name = "Plates, levers, cages and triggerables",
		desc = "Would you a like trigger?",
		author = "gajop",
		date = "April 2016",
		license = "GNU GPL, v2 or later",
		layer = 1,
		enabled = true
	}
end

local LOG_SECTION = "trigger-triggerable"
local LOG_LEVEL = LOG.NOTICE

if (gadgetHandler:IsSyncedCode()) then

local triggers = {}
local bitmaskLinks = {}
local linkChecksEnabled = false
local reportedError = false
local UPDATE_RATE = 5
local PLATE_ACTIVATION_RANGE = 60

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam, builderID)
	local unitDef = UnitDefs[unitDefID]
    if unitDef.customParams.trigger then
        triggers[unitID] = { pressed = false }
		if unitDef.name == "lever" then
			Spring.SetUnitBlocking(unitID, false, false, false, true, false, false, false)
		else
			Spring.SetUnitCollisionVolumeData(unitID, 
			0, 0, 0, 
			0, 0, 0, 
			0, 0, 0);
			Spring.SetUnitBlocking(unitID, false, false, false, false, false, false, false)
		end
    end
--     -- EXAMPLE:
--     if UnitDefs[unitDefID].customParams.triggerable then
--         for triggerID, _ in pairs(triggers) do
--             SimpleLink(triggerID, unitID)
--         end
--     end
end

function gadget:UnitDestroyed(unitID, unitDefID, ...)
    if triggers[unitID] then
        triggers[unitID] = nil
    end
    if bitmaskLinks[unitID] then
        bitmaskLinks[unitID] = nil
    end
end

function SimpleLink(triggerID, triggerableID, opts)
	opts = opts or {}
    if triggers[triggerID] == nil then
        Spring.Log(LOG_SECTION, "error", "SimpleLink: No such trigger with ID: ", triggerID)
        return
    end
    if not UnitDefs[Spring.GetUnitDefID(triggerableID)].customParams.triggerable then
        Spring.Log(LOG_SECTION, "error", "SimpleLink: Trying to link trigger with non-triggerable: ", triggerableID)
        return
    end
    Spring.Log(LOG_SECTION, LOG_LEVEL, "Linking trigger " .. tostring(triggerID) .. " with triggerable: " .. tostring(triggerableID))
	if triggers[triggerID].triggerables == nil then
		triggers[triggerID].triggerables = {}
	end
    table.insert(triggers[triggerID].triggerables, {
		triggerableID = triggerableID,
		opts = opts
	})
end

function BitmaskLink(triggerMask, triggerableID)
    for _, triggerObj in pairs(triggerMask) do
        local triggerID = triggerObj[1]
        if triggers[triggerID] == nil then
            Spring.Log(LOG_SECTION, "error", "BitwiseLink: No such trigger with ID: ", triggerID)
            return
        end
    end
    if not UnitDefs[Spring.GetUnitDefID(triggerableID)].customParams.triggerable then
        Spring.Log(LOG_SECTION, "error", "BitwiseLink: Trying to link trigger with non-triggerable: ", triggerableID)
        return
    end
    for _, triggerObj in pairs(triggerMask) do
        local triggerID = triggerObj[1]
        triggers[triggerID].bitmaskLink = true
    end
    bitmaskLinks[triggerableID] = triggerMask
end

function EnableLinkChecks()
    linkChecksEnabled = true
    reportedError = false
end

function DisableLinkChecks()
    linkChecksEnabled = false
end

local function SetUnitState(unitID, state)
    if triggers[unitID] then 
        triggers[unitID].state = state
    end
    local active = Spring.GetUnitStates(unitID).active
    if active ~= state then
        if state then
            Spring.GiveOrderToUnit(unitID, CMD.ONOFF, { 1 }, {})
        else
--             Spring.GiveOrderToUnit(unitID, CMD.ONOFF, { 0 }, {})
        end
    end
end

function gadget:GameFrame()
    if not linkChecksEnabled and false then
        return
    end

    if Spring.GetGameFrame() % UPDATE_RATE == 0 then
        -- check if triggers are toggled or not
        for triggerID, trigger in pairs(triggers) do
            if trigger.triggerables or trigger.bitmaskLink then
                local x, y, z = Spring.GetUnitPosition(triggerID)
                local units = Spring.GetUnitsInCylinder(x, z, PLATE_ACTIVATION_RANGE)
                local newState = false
				if UnitDefs[Spring.GetUnitDefID(triggerID)].customParams.stand_trigger then
					for _, unitID in pairs(units) do
						if Spring.GetGameRulesParam("spiritMode") ~= 0 or Spring.GetGameRulesParam("has_legs") ~= 1 then
							break
						end
						if UnitDefs[Spring.GetUnitDefID(unitID)].customParams.trigger_toggler then
							newState = true
							break
						end
					end
					SetUnitState(triggerID, newState)
				end
            elseif not reportedError then
                Spring.Log(LOG_SECTION, LOG_LEVEL, "Plate has no triggerable: " .. tostring(triggerID))
            end
        end
        reportedError = true

        -- issue simple links
        for triggerID, trigger in pairs(triggers) do
            if trigger.triggerables then
				for _, triggerable in pairs(trigger.triggerables) do
					local triggerableID = triggerable.triggerableID
					local opts = triggerable.opts
					local active = Spring.GetUnitStates(triggerID).active
					if active then
						for key, value in pairs(opts) do
							Spring.SetUnitRulesParam(triggerableID, key, value)
						end
					end
					SetUnitState(triggerableID, active)
				end
            end
        end
        -- issue bitmask links
        for triggerableID, bitmaskLink in pairs(bitmaskLinks) do
            local totalState = 1
            for _, triggerObj in pairs(bitmaskLink) do
                if triggerObj[2] ~= Spring.GetUnitStates(triggerObj[1]).active then
                    totalState = false
                    break
                end
            end
            SetUnitState(triggerableID, totalState)
        end
    end
end

GG.Plate = {
    BitmaskLink = BitmaskLink,
    SimpleLink = SimpleLink,
    EnableLinkChecks = EnableLinkChecks,
    DisableLinkChecks = DisableLinkChecks,
}

end
