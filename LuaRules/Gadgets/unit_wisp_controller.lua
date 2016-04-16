--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Wisp",
		desc	= "Wisp control gadget.",
		author	= "gajop",
		date	= "16 April 2016",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end


-------------------------------------------------------------------
-- SYNCED
-------------------------------------------------------------------
if gadgetHandler:IsSyncedCode() then
-------------------------------------------------------------------

local MAX_HEIGHT_CHANGE = 40

-- local wispDefID = UnitDefNames["wisp"].id
local wispID = nil

local heightmapChangedFrame
-------------------------------------------------------------------
-------------------------------------------------------------------

local function explode(div,str)
	if (div=='') then return 
		false 
	end
	local pos,arr = 0,{}
	-- for each divider found
	for st,sp in function() return string.find(str,div,pos,true) end do
		table.insert(arr,string.sub(str,pos,st-1)) -- Attach chars left of current divider
		pos = sp + 1 -- Jump past current divider
	end
	table.insert(arr,string.sub(str,pos)) -- Attach chars right of last divider
	return arr
end

-------------------------------------------------------------------
-- Handling unit
-------------------------------------------------------------------

function gadget:UnitCreated(unitID, unitDefID, unitTeam)
	if unitDefID == wispDefID then
		wispID = unitID
	end
end

function gadget:UnitDestroyed(unitID)
	if wispID == unitID then
		wispID = nil
	end
end

function gadget:GameStart()
	gameStarted = true
end

function gadget:Initialize()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

local kernels = {}
function _PrecalculateKernel(radius)
	if kernels[radius] then return kernels[radius] end
	local kernel = {}
	kernels[radius] = kernel
	for x = 0, 2*radius do
		local dx = radius - x
		for y = 0, 2*radius do
			local dy = radius - y
			kernel[1 + x + y * 2*radius] = 1000 / (dx * dx + dy * dy + 1000)
		end
	end
	return kernel
end

function _ChangeHeightmap(startX, startZ, delta, radius)
	local kernel = _PrecalculateKernel(radius)
	for x = 0, 2*radius, Game.squareSize do
		for z = 0, 2*radius, Game.squareSize do
			Spring.AddHeightMap(x + startX, z + startZ, delta * kernel[1 + x + z * radius])
		end
	end
end

function ChangeHeightmap(x, z, delta, radius)
	Spring.SetHeightMapFunc(_ChangeHeightmap, x - radius, z - radius, delta, radius)
end

-------------------------------------------------------------------
-- Handling messages
-------------------------------------------------------------------

function HandleLuaMessage(msg)
-- 	if not wispID then
-- 		return
-- 	end
	local msg_table = explode('|', msg)
	if msg_table[1] == 'inc_heightmap' then --LMB
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])	

		ChangeHeightmap(x, z, 0.6, 20)
	elseif msg_table[1] == 'dec_heightmap' then -- RMB
		local x = tonumber(msg_table[2])
		local y = tonumber(msg_table[3])
		local z = tonumber(msg_table[4])

		ChangeHeightmap(x, z, -0.6, 20)
	end
end

function gadget:RecvLuaMsg(msg)
	HandleLuaMessage(msg)
end



-------------------------------------------------------------------
-- UNSYNCED
-------------------------------------------------------------------
else
-------------------------------------------------------------------

  return

-------------------------------------------------------------------
end

