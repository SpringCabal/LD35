-- $Id: BloomShader.lua 3171 2008-11-06 09:06:29Z det $
function widget:GetInfo()
	return {
		name      = "Blood/spirit drawing shader",
		desc      = "Draws blood and spirit on ground",
		author    = "gajop",
		date      = "April 2016",
		license   = "",
		layer     = 9,
		enabled   = true
	}
end

-- gl.Utilities

if (not gl) then
    return
end

gl.Utilities = gl.Utilities or {}

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local min    = math.min
local max    = math.max
local sin    = math.sin
local cos    = math.cos
local floor  = math.floor
local TWO_PI = math.pi * 2

local glVertex = gl.Vertex

GL.KEEP      = 0x1E00
GL.INCR_WRAP = 0x8507
GL.DECR_WRAP = 0x8508
GL.INCR      = 0x1E02
GL.DECR      = 0x1E03
GL.INVERT    = 0x150A

local stencilBit1 = 0x01
local stencilBit2 = 0x10

function gl.Utilities.DrawVolume(vol_dlist)
  gl.DepthMask(false)
  if (gl.DepthClamp) then gl.DepthClamp(true) end
  gl.StencilTest(true)

  gl.Culling(false)
  gl.DepthTest(true)
  gl.ColorMask(false, false, false, false)
  gl.StencilOp(GL.KEEP, GL.INCR, GL.KEEP)
  --gl.StencilOp(GL.KEEP, GL.INVERT, GL.KEEP)
  gl.StencilMask(0x11)
  gl.StencilFunc(GL.ALWAYS, 0, 0)

  gl.CallList(vol_dlist)

  gl.Culling(GL.FRONT)
  gl.DepthTest(false)
  gl.ColorMask(true, true, true, true)
  gl.StencilOp(GL.ZERO, GL.ZERO, GL.ZERO)
  gl.StencilMask(0x11)
  gl.StencilFunc(GL.NOTEQUAL, 0, 0+1)

  gl.CallList(vol_dlist)

  if (gl.DepthClamp) then gl.DepthClamp(false) end
  gl.StencilTest(false)
  gl.DepthTest(true)
  gl.Culling(false)
end

--


local shaderObj, dlist
local scale = 1
local size = 100
local rotation = 0

local groundDetailMapping = {
	blood = {
		[4] = "b_ok",
		[5] = "b_ok",
		[6] = "b_ok",
		[7] = "b_ok",
		
		[8] = "b_bad",
		
		[14] = "b_bad",
		
		[19] = "b_ok",
	},
	spirit = {
		[9] = "s_ok",
		[10] = "s_bad",
		[11] = "s_ok",
		[12] = "s_ok",
		
		[13] = "s_ok",
		
		[15] = "s_bad",
		[16] = "s_bad",
		[17] = "s_bad",
		[18] = "s_bad",
	},
}

local areas = {}

function InitShader()
    local shaderFragStr = [[
        uniform sampler2D brushTex;
        void main(void)
        {
            vec4 brushColor = texture2D(brushTex, gl_TexCoord[0].st);

            gl_FragColor = gl_Color * brushColor.rgba;
            //gl_FragColor = gl_Color;
            //gl_FragColor = vec4(gl_TexCoord[0].st, 0, 1);
        }
    ]]

    local shaderTemplate = {
        fragment = shaderFragStr,
        uniformInt = {
            brushTex = 0,
        },
    }

    local shader = gl.CreateShader(shaderTemplate)
    local errors = gl.GetShaderLog(shader)
	if shader ~= nil then
		shaderObj = {
            shader = shader,
        }
	end
    if errors ~= "" and errors ~= nil then
        Spring.Log("Scened", "notice", "Shader infolog: " .. tostring(errors))
    end
end

-- minX,minY,minZ, maxX,maxY,maxZ
-- 0,   -0.5,  0,     1, 0.5,   1
function DrawRectangle()
    gl.BeginEnd(GL.QUADS, function()
    --                 gl.MultiTexCoord(0, mCoord[1], mCoord[2])
    --                 gl.MultiTexCoord(1, tCoord[1], tCoord[2] )
        gl.MultiTexCoord(0, 0, 0 )
        gl.Vertex(0, 0, 0)

    --                 gl.MultiTexCoord(0, mCoord[3], mCoord[4])
    --                 gl.MultiTexCoord(1, tCoord[3], tCoord[4] )
        gl.MultiTexCoord(0, 1, 0 )
        gl.Vertex(1, 0, 0)

    --                 gl.MultiTexCoord(0, mCoord[5], mCoord[6])
    --                 gl.MultiTexCoord(1, tCoord[5], tCoord[6] )
        gl.MultiTexCoord(0, 1, 1 )
        gl.Vertex(1, 0, 1)

    --                 gl.MultiTexCoord(0, mCoord[7], mCoord[8])
    --                 gl.MultiTexCoord(1, tCoord[7], tCoord[8] )
        gl.MultiTexCoord(0, 0, 1 )
        gl.Vertex(0, 0, 1)
    end)
end

-- local heightMargin = 2000
-- local averageGroundHeight = (minheight + maxheight) / 2
-- local shapeHeight = heightMargin + (maxheight - minheight) + heightMargin

function DrawTexturedGroundRectangle(x1,z1,x2,z2, rot, dlist)
  if (type(x1) == "table") then
    local rect = x1
    x1,z1,x2,z2 = rect[1],rect[2],rect[3],rect[4]
  end
  gl.PushMatrix()
  local sizeX, sizeZ = x2 - x1, z2 - z1
  local y = Spring.GetGroundHeight((x1+x2)/2, (z1+z2)/2) - 1
--   gl.Rotate(rot, 0, 1, 0)
  gl.Translate(x1, y, z1)
  gl.Translate(sizeX/2, 0, sizeZ/2)
  gl.Rotate(rot, 0, 1, 0)
  gl.Translate(-sizeX/2, 0, -sizeZ/2)
  gl.Scale(x2-x1, 1, z2-z1)
  gl.Utilities.DrawVolume(dlist)
  gl.PopMatrix()
end

function DrawShape(shape, x1, z1, x2, z2)
    gl.PushMatrix()
    local scale = 1/2 * math.sqrt(2)
--     local rotRad = math.rad(self.rotation) + math.pi/2

    if not shaderObj then
        InitShader()
        dlist = gl.CreateList(DrawRectangle)
    end
    gl.Texture(0, shape)
    gl.UseShader(shaderObj.shader)
    gl.Blending("alpha_add")
--     gl.Color(0, 1, 0, 0.3)
--         gl.Utilities.DrawGroundRectangle(x-self.size, z-self.size, x+self.size, z+self.size)
    DrawTexturedGroundRectangle(x1, z1, x2, z2, rotation, dlist)
    gl.UseShader(0)
    gl.Texture(0, false)
--     gl.Color(0, 1, 1, 0.5)
--     gl.Utilities.DrawGroundHollowCircle(x+self.size * math.sin(rotRad), z+self.size * math.cos(rotRad), self.size / 10, self.size / 12)
    gl.PopMatrix()
end


function DrawStuff()
	gl.DepthTest(true)
    gl.Blending(GL.SRC_ALPHA, GL.ONE)
	local spiritMode = Spring.GetGameRulesParam("spiritMode")
-- 	x, y = Spring.GetMouseState()
-- 	local result, coords = Spring.TraceScreenRay(x, y, true)
-- 	if result == "ground" then
-- 		local x, z = coords[1], coords[3]
-- 		
-- 		DrawShape(shape, x, z)
-- 	end	
	for name, groundTypes in pairs(groundDetailMapping) do
		if (name == "blood" and spiritMode == 0) or
		   (name == "spirit" and spiritMode == 1) then
			for id, fname in pairs(groundTypes) do
				local shape = "LuaUI/images/ground/" .. fname .. ".png"
				local area = areas[id]
				DrawShape(shape, area[1], area[2], area[3], area[4])
			end
		end
	end
	
	gl.Texture(false)
	gl.Blending(GL.SRC_ALPHA, GL.ONE_MINUS_SRC_ALPHA)
    gl.DepthTest(false)
end

function widget:DrawWorldPreUnit()
	DrawStuff()
end

local customElements = "LuaRules/Configs/map_customs.lua"
function widget:Initialize()
	local elements = VFS.LoadFile(customElements)
	elements = loadstring(elements)()
	areas = elements.areas
end
