--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function gadget:GetInfo()
	return {
		name 	= "Wisp draw",
		desc	= "Wisp draw gadget.",
		author	= "gajop",
		date	= "16 April 2016",
		license	= "GNU GPL, v2 or later",
		layer	= 0,
		enabled = true
	}
end


-------------------------------------------------------------------
-- UNSYNCED
-------------------------------------------------------------------
if not gadgetHandler:IsSyncedCode() then
	
local TEXTURE     = 'LuaUI/Images/wisp1.png'

-------------------------------------------------------------------
-------------------------------------------------------------------
	
local wispDefID = UnitDefNames["wisp"].id
local wispID = nil
local npcWispDefID = UnitDefNames["npcwisp"].id

-- BEGIN Custom eye stuff
-- local shader
local shaderTimeLoc = nil
local shaderSpiritAmountLoc = nil
local shaderUnitIDLoc = nil

-- changing form manually
local CHANGE_DURATION = 15 -- in frames
local oldSpiritMode
local changeTime
local changeSpiritMode = 0

-- END


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

function gadget:Initialize()
	CreateShader()
	for _, unitID in ipairs(Spring.GetAllUnits()) do
		local unitDefID = Spring.GetUnitDefID(unitID)
		gadget:UnitCreated(unitID, unitDefID)
	end
end

function FreeResources()
  if (gl.DeleteShader) then
    gl.DeleteShader(shader)
  end
end

function DrawWisp()
    gl.DepthTest(true)
    gl.Blending(GL.SRC_ALPHA, GL.ONE)

	local width, height = 40, 40

	local x, y, z = Spring.GetUnitViewPosition(wispID)
	gl.PushMatrix()
	gl.Translate(x,y,z)
	gl.Billboard()
	gl.TexRect(-width/2, 0, width/2, height)
	gl.PopMatrix()
	
	for _, npcWispID in pairs(Spring.GetAllUnits()) do
		if Spring.GetUnitDefID(npcWispID) == npcWispDefID then
			x, y, z = Spring.GetUnitViewPosition(npcWispID)
			gl.Uniform(shaderUnitIDLoc,  npcWispID)
			gl.PushMatrix()
			gl.Translate(x,y,z)
			gl.Billboard()
			gl.TexRect(-width/2, 0, width/2, height)
			gl.PopMatrix()
		end
	end

    gl.Texture(false)
    gl.Blending(GL.SRC_ALPHA, GL.ONE_MINUS_SRC_ALPHA)
    gl.DepthTest(false)
end

function CreateShader()
  shaderNeedLocs = true

  shader = gl.CreateShader({
	fragment = [[
	  uniform float time;
	  uniform float spiritAmount;
	  uniform float unitID;

	  float rand(vec2 n) { 
		return fract(sin(dot(n, vec2(12.9898, 4.1414))) * 43758.5453);
	}

      void main(void)
      {
		float dx = 0.5 - gl_TexCoord[0].x;
		float dy = 0.5 - gl_TexCoord[0].y;
		float dist = 1 - 2.5 * (1.5 * dx * dx + dy * dy);
		
		float f = 0;
		float2 coord = gl_TexCoord[0].xy;
		for (int i = -3; i <= 3; i++) {
			for (int j = -3; j <= 3; j++) {
				coord = round(gl_TexCoord[0].xy * 50 + vec2(i, j));
				f += rand(coord * time * unitID) * dist;
			}
		}
		gl_FragColor.rgba = f / 49;
		gl_FragColor.r *= rand(vec2(cos(unitID), sin(unitID)));
		gl_FragColor.g *= rand(vec2(sin(unitID), cos(unitID)));
		gl_FragColor.b *= rand(vec2(-cos(unitID), -sin(unitID)));
		
		gl_FragColor.rgba += dist * dist * dist / 1.2;
		gl_FragColor.rgba *= spiritAmount;
      }
    ]],
  })

  if (shader == nil) then
    print(gl.GetShaderLog())
    return false
  end
  return true
end

function GetShaderLocations()
  shaderTimeLoc           = gl.GetUniformLocation(shader, 'time')
  shaderSpiritAmountLoc   = gl.GetUniformLocation(shader, 'spiritAmount')
  shaderUnitIDLoc         = gl.GetUniformLocation(shader, 'unitID')
end

function gadget:DrawScreenEffects()
  if not wispID then
	return
  end
  if not shader then return end

  	if oldSpiritMode == nil then
		oldSpiritMode = Spring.GetGameRulesParam("spiritMode")
	end
	local newSpiritMode = Spring.GetGameRulesParam("spiritMode")
	if oldSpiritMode ~= newSpiritMode then
		changeSpiritMode = newSpiritMode - oldSpiritMode
		changeTime = Spring.GetGameFrame()
		oldSpiritMode = newSpiritMode
	end

	local opts = {}
	if changeSpiritMode ~= 0 then
		local deltaTime = Spring.GetGameFrame() - changeTime
		local progress = math.min(1, deltaTime / CHANGE_DURATION)
		if changeSpiritMode == 1 then
			opts = { spiritAmount = 1 * progress}
		else
			opts = { spiritAmount = math.max(0, 1 - 5 * progress)}
		end

		if deltaTime > CHANGE_DURATION then
			changeSpiritMode = 0
		end
	elseif Spring.GetGameRulesParam("spiritMode") == 1 then
		opts = { spiritAmount = 1 }
	else
		opts = { spiritAmount = 0 }
	end

  gl.UseShader(shader)
  if (shaderNeedLocs) then
    GetShaderLocations()
    shaderNeedLocs = false
  end

  local gameFrame = Spring.GetGameFrame()

  local timeOffset = Spring.GetFrameTimeOffset() / 30
  local time = Spring.GetGameSeconds() + timeOffset
  time = math.floor(10 * time) / 10
  gl.Uniform(shaderTimeLoc,   time)
  gl.Uniform(shaderSpiritAmountLoc,  opts.spiritAmount)
  gl.Uniform(shaderUnitIDLoc,  wispID)

  gl.MatrixMode(GL.PROJECTION); gl.PushMatrix(); gl.LoadMatrix("camprj")
  gl.MatrixMode(GL.MODELVIEW);  gl.PushMatrix(); gl.LoadMatrix("camera")

  DrawWisp(opts)

  gl.MatrixMode(GL.PROJECTION); gl.PopMatrix()
  gl.MatrixMode(GL.MODELVIEW);  gl.PopMatrix()

  gl.UseShader(0)
end

end

