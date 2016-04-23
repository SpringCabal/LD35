
function gadget:GetInfo()
  return {
    name    = "Wall Terrain Maker",
    desc    = "Sets terrain height below walls",
    author  = "ashdnazg",
    date    = "",
    license = "Public Domain",
    layer   = 0,
    enabled = true,
  }
end

if (gadgetHandler:IsSyncedCode()) then 

local function RoundToHeightScale(x)
    local diff = x % Game.squareSize
    if diff > Game.squareSize / 2 then
        return x + Game.squareSize - diff
    else
        return x - diff
    end
end
    
local function AdjustWallTerrain(unitID, height)
    
    local wx, wy, wz = Spring.GetUnitPosition(unitID)
    local scaleX, scaleY, scaleZ, offsetX, offsetY, offsetZ = Spring.GetUnitCollisionVolumeData(unitID)
    
    -- hack for gate opening:
    local dx, dy, dz = Spring.GetUnitDirection(unitID)
    local x1 = wx - ((scaleX * dz / 2) + scaleZ * dx / 2)
    local z1 = wz + scaleX * dx / 2 -- ((scaleZ * dz / 2) - scaleX * dx / 2)
    local x2 = wx + ((scaleX * dz / 2) + scaleZ * dx / 2)
    local z2 = wz - scaleX * dx / 2 -- ((scaleZ * dz / 2) - scaleX * dx / 2)
    
	Spring.Echo( x1 - x2, z1 - z2)
--     Spring.Echo(x1, z1, x2, z2, x1 - x2, z1 - z2, scaleZ)
    
    local x, z = x1, z1
    local xdiff = dz * 8
    local zdiff = -(dx * 8)
    local num
    if math.abs(xdiff) > math.abs(zdiff) then
        num = math.ceil(math.abs((x1 - x2) / xdiff)) + 1
    else
        num = math.ceil(math.abs((z1 - z2) / zdiff)) + 1
    end
    --local rows = math.floor(math.abs(z1 - z2) / zdiff)
    --Spring.Echo(num, xdiff, zdiff)
    -- local minx, maxx, minz, maxz = math.min(x1, x2), math.max(x1, x2), math.min(z1, z2), math.max(z1, z2)2
    for i = 1, num do
        Spring.LevelHeightMap(RoundToHeightScale(x) - 2, RoundToHeightScale(z) - 2, RoundToHeightScale(x) + 2, RoundToHeightScale(z) + 2,  wy + height * scaleY)
        x = x + xdiff
        z = z + zdiff
    end
end

local areas = {}
function AddAreaWall(unitID, area)
	areas[unitID] = area
	AdjustAreaTerrain(unitID, 1)
end

function AdjustAreaTerrain(unitID, adjust)
	local area = areas[unitID]
	local value = 499
	if adjust == 0 then
		value = 411.5
	end
	if area then
		Spring.LevelHeightMap(area[1], area[2], area[3], area[4], value)
		GG.Delay.DelayCall(function()
			for _, uid in pairs(Spring.GetUnitsInRectangle(area[1]-100, area[2]-100, area[3]+100, area[4]+100)) do
				if UnitDefs[Spring.GetUnitDefID(uid)].name ~= "wisp" then
					local ux, uy, uz = Spring.GetUnitPosition(uid)
					Spring.MoveCtrl.Enable(uid)
					Spring.SetUnitPosition(uid, ux, 411.5, uz)
				end
			end
		end, {})
	end
end

function gadget:UnitCreated(unitID, unitDefID, unitTeam, builderID)
    GG.Delay.DelayCall(function()
        local unitDef = UnitDefs[unitDefID]
        if unitDef.name == "gatesmoth"  then
			AdjustAreaTerrain(unitID, 1)
        end
    end, {})
    
--     
--     if not unitDef.customParams.wall then return end
--     --local dx, dy, dz = Spring.SetUnitRotation(unitID, 0, , 0)
--     GG.Delay.DelayCall(AdjustWallTerrain, {unitID, 1})
end

function gadget:UnitDestroyed(unitID, unitDefID, unitTeam, builderID)
    local unitDef = UnitDefs[unitDefID]
    if not unitDef.customParams.wall then return end
    if Spring.GetUnitRulesParam(unitID, "invulnerable") == 0 or unitDef.name == "gate" then
        --GG.Delay.DelayCall(AdjustWallTerrain, {unitID, 0})
    end
end

GG.AdjustWallTerrain = AdjustAreaTerrain
GG.AddAreaWall = AddAreaWall

end
