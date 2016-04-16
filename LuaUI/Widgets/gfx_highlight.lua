-- $Id: gfx_outline.lua 3171 2008-11-06 09:06:29Z det $
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
--
--  file:    gfx_outline.lua
--  brief:   Displays a nice cartoon like outline around units
--  author:  jK
--
--  Copyright (C) 2007.
--  Licensed under the terms of the GNU GPL, v2 or later.
--
--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
  return {
    name      = "Outline",
    desc      = "Displays a nice cartoon like outline around units.",
    author    = "jK",
    date      = "Dec 06, 2007",
    license   = "GNU GPL, v2 or later",
    layer     = -10,
    enabled   = true  --  loaded by default?
  }
end

local thickness = 1
local forceLowQuality = false

local function OnchangeFunc()
  thickness = options.thickness.value
end
local function QualityChangeCheckFunc()
  if forceLowQuality then
    options.lowQualityOutlines.OnChange = nil
    options.lowQualityOutlines.value = true
    options.lowQualityOutlines.OnChange = QualityChangeCheckFunc
  end
end

options_path = 'Settings/Graphics/Unit Visibility/Outline'
options = {
	thickness = {
		name = 'Outline Thickness',
		desc = 'How thick the outline appears around objects',
		type = 'number',
		min = 0.4, max = 1, step = 0.01,
		value = 1,
    OnChange = OnchangeFunc,
	},
  lowQualityOutlines = {
    name = 'Low Quality Outlines',
    desc = 'Reduces outline accuracy to improve perfomance, only recommended for low-end machines',
    type = 'bool',
    value = false,
    advanced = true,
    OnChange = QualityChangeCheckFunc,
  },
}

OnchangeFunc()

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--//textures
local offscreentex
local depthtex
local blurtex

--//shader
local depthShader
local blurShader_h
local blurShader_v
local uniformUseEqualityTest, uniformScreenXY, uniformScreenX, uniformScreenY

--// geometric
local vsx, vsy = 0,0
local resChanged = false

--// display lists
local enter2d,leave2d

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local GL_DEPTH_BITS = 0x0D56

local GL_DEPTH_COMPONENT   = 0x1902
local GL_DEPTH_COMPONENT16 = 0x81A5
local GL_DEPTH_COMPONENT24 = 0x81A6
local GL_DEPTH_COMPONENT32 = 0x81A7

--// speed ups
local ALL_UNITS       = Spring.ALL_UNITS
local GetUnitHealth   = Spring.GetUnitHealth
local GetVisibleUnits = Spring.GetVisibleUnits

local GL_MODELVIEW  = GL.MODELVIEW
local GL_PROJECTION = GL.PROJECTION
local GL_COLOR_BUFFER_BIT = GL.COLOR_BUFFER_BIT

local glUnit            = gl.Unit
local glCopyToTexture   = gl.CopyToTexture
local glRenderToTexture = gl.RenderToTexture
local glCallList        = gl.CallList

local glUseShader  = gl.UseShader
local glUniform    = gl.Uniform
local glUniformInt = gl.UniformInt

local glClear    = gl.Clear
local glTexRect  = gl.TexRect
local glColor    = gl.Color
local glTexture  = gl.Texture

local glResetMatrices = gl.ResetMatrices
local glMatrixMode    = gl.MatrixMode
local glPushMatrix    = gl.PushMatrix
local glLoadIdentity  = gl.LoadIdentity
local glPopMatrix     = gl.PopMatrix

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

--tables
local unbuiltUnits = {}

--------------------------------------------------------------------------------
--------------------------------------------------------------------------------


function widget:Initialize()
  vsx, vsy = widgetHandler:GetViewSizes()

  self:ViewResize(widgetHandler:GetViewSizes())

  if gl.CreateShader == nil then --For old Intel chips
    Spring.Log(widget:GetInfo().name, LOG.ERROR, "Outline widget: cannot create shaders. forcing shader-less fallback.")
    forceLowQuality = true
    options.lowQualityOutlines.value = true
    return true
  end

  --For cards that can use shaders
  enter2d = gl.CreateList(function()
    glUseShader(0)
    glMatrixMode(GL_PROJECTION); glPushMatrix(); glLoadIdentity()
    glMatrixMode(GL_MODELVIEW);  glPushMatrix(); glLoadIdentity()
    gl.Blending("alpha")
  end)
  leave2d = gl.CreateList(function()
    glMatrixMode(GL_PROJECTION); glPopMatrix()
    glMatrixMode(GL_MODELVIEW);  glPopMatrix()
    glTexture(false)
    glUseShader(0)
    gl.Blending(false)
  end)

  depthShader = gl.CreateShader({
    fragment = [[
      uniform sampler2D tex0;
      uniform int useEqualityTest;
      uniform vec2 screenXY;

      void main(void)
      {
        vec2 texCoord = vec2( gl_FragCoord.x/screenXY.x , gl_FragCoord.y/screenXY.y );
        float depth  = texture2D(tex0, texCoord ).z;

        if (depth < gl_FragCoord.z) {
          discard;
        }
        gl_FragColor = gl_Color;
      }
    ]],
    uniformInt = {
      tex0 = 0,
      useEqualityTest = 1,
    },
    uniform = {
      screenXY = {vsx,vsy},
    },
  })

  blurShader_h = gl.CreateShader({
    fragment = [[
      uniform sampler2D tex0;
      uniform int screenX;

      const vec2 kernel = vec2(0.6,0.7);

      void main(void) {
        vec2 texCoord  = vec2(gl_TextureMatrix[0] * gl_TexCoord[0]);
        gl_FragColor = vec4(0.0);

        float pixelsize = 1.0/float(screenX);
        gl_FragColor += kernel[0] * texture2D(tex0, vec2(texCoord.s + 2.0*pixelsize,texCoord.t) );
        gl_FragColor += kernel[1] * texture2D(tex0, vec2(texCoord.s + pixelsize,texCoord.t) );

        gl_FragColor += texture2D(tex0, texCoord );

        gl_FragColor += kernel[1] * texture2D(tex0, vec2(texCoord.s + -1.0*pixelsize,texCoord.t) );
        gl_FragColor += kernel[0] * texture2D(tex0, vec2(texCoord.s + -2.0*pixelsize,texCoord.t) );
      }
    ]],
    uniformInt = {
      tex0 = 0,
      screenX = vsx,
    },
  })


  blurShader_v = gl.CreateShader({
    fragment = [[      uniform sampler2D tex0;
      uniform int screenY;

      const vec2 kernel = vec2(0.6,0.7);

      void main(void) {
        vec2 texCoord  = vec2(gl_TextureMatrix[0] * gl_TexCoord[0]);
        gl_FragColor = vec4(0.0);

        float pixelsize = 1.0/float(screenY);
        gl_FragColor += kernel[0] * texture2D(tex0, vec2(texCoord.s,texCoord.t + 2.0*pixelsize) );
        gl_FragColor += kernel[1] * texture2D(tex0, vec2(texCoord.s,texCoord.t + pixelsize) );

        gl_FragColor += texture2D(tex0, texCoord );

        gl_FragColor += kernel[1] * texture2D(tex0, vec2(texCoord.s,texCoord.t + -1.0*pixelsize) );
        gl_FragColor += kernel[0] * texture2D(tex0, vec2(texCoord.s,texCoord.t + -2.0*pixelsize) );
      }
    ]],
    uniformInt = {
      tex0 = 0,
      screenY = vsy,
    },
  })

  if (depthShader == nil) then
    Spring.Log(widget:GetInfo().name, LOG.ERROR, "Outline widget: depthcheck shader error, forcing shader-less fallback: "..gl.GetShaderLog())
    -- widgetHandler:RemoveWidget()
    -- return false
    forceLowQuality = true
    options.lowQualityOutlines.value = true
    return true
  end
  if (blurShader_h == nil) then
    Spring.Log(widget:GetInfo().name, LOG.ERROR, "Outline widget: hblur shader error, forcing shader-less fallback: "..gl.GetShaderLog())
    -- widgetHandler:RemoveWidget()
    -- return false
    forceLowQuality = true
    options.lowQualityOutlines.value = true
    return true
  end
  if (blurShader_v == nil) then
    Spring.Log(widget:GetInfo().name, LOG.ERROR, "Outline widget: vblur shader error, forcing shader-less fallback: "..gl.GetShaderLog())
    -- widgetHandler:RemoveWidget()
    -- return false
    forceLowQuality = true
    options.lowQualityOutlines.value = true
    return true
  end

  uniformScreenXY        = gl.GetUniformLocation(depthShader,  'screenXY')
  uniformScreenX         = gl.GetUniformLocation(blurShader_h, 'screenX')
  uniformScreenY         = gl.GetUniformLocation(blurShader_v, 'screenY')
end

function widget:ViewResize(viewSizeX, viewSizeY)
  vsx = viewSizeX
  vsy = viewSizeY

  gl.DeleteTexture(depthtex or 0)
  gl.DeleteTextureFBO(offscreentex or 0)
  gl.DeleteTextureFBO(blurtex or 0)

  if not forceLowQuality then
    depthtex = gl.CreateTexture(vsx,vsy, {
      border = false,
      format = GL_DEPTH_COMPONENT24,
      min_filter = GL.NEAREST,
      mag_filter = GL.NEAREST,
    })

    offscreentex = gl.CreateTexture(vsx,vsy, {
      border = false,
      min_filter = GL.LINEAR,
      mag_filter = GL.LINEAR,
      wrap_s = GL.CLAMP,
      wrap_t = GL.CLAMP,
      fbo = true,
      fboDepth = true,
    })

    blurtex = gl.CreateTexture(vsx,vsy, {
      border = false,
      min_filter = GL.LINEAR,
      mag_filter = GL.LINEAR,
      wrap_s = GL.CLAMP,
      wrap_t = GL.CLAMP,
      fbo = true,
    })
  end

  resChanged = true
end


function widget:Shutdown()
  gl.DeleteTexture(depthtex or 0)
  if (gl.DeleteTextureFBO) then
    gl.DeleteTextureFBO(offscreentex or 0)
    gl.DeleteTextureFBO(blurtex or 0)
  end

  if (gl.DeleteShader) then
    gl.DeleteShader(depthShader or 0)
    gl.DeleteShader(blurShader_h or 0)
    gl.DeleteShader(blurShader_v or 0)
  end

  gl.DeleteList(enter2d or 0)
  gl.DeleteList(leave2d or 0)
end


--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

local function DrawVisibleUnits(overrideEngineDraw)
  if (Spring.GetGameFrame() % 15 == 0) then
        checknow = true
  end
 
  if Spring.GetGameRulesParam("has_arms") ~= 1 and Spring.GetGameRulesParam("spiritMode") ~= 0 then
	  return
  end

  local mx, my, lmb, mmb, rmb = Spring.GetMouseState()
  local traceType, unitID = Spring.TraceScreenRay(mx, my)
  if traceType == "unit" and UnitDefs[Spring.GetUnitDefID(unitID)].name == "lever" then
	local createdFrame = Spring.GetUnitRulesParam(unitID, "createdFrame") or 0
	local growTime = 33 * 0.5
	local duration = Spring.GetGameFrame() - createdFrame
	local grownPercentage = math.min(1, duration / growTime)
	local unitDefID = Spring.GetUnitDefID(unitID )
	local unitDef = UnitDefs[unitDefID]
	
    glUnit(unitID, overrideEngineDraw)
  end
end

local MyDrawVisibleUnits = function()
  glClear(GL_COLOR_BUFFER_BIT,0,0,0,0)
  glPushMatrix()
  glResetMatrices()
  glColor(0,0,0,thickness)
  DrawVisibleUnits(true)
  glColor(1,1,1,1)
  glPopMatrix()
end

local DrawVisibleUnitsLines = function(underwater) --This is expected to be a shader-less fallback for low-end machines, though it also works for refraction pass

  gl.DepthTest(GL.LESS)
  if underwater then
    gl.LineWidth(4.0 * thickness)
    gl.PolygonOffset(8.0, 4.0)
  else
    gl.LineWidth(4.0 * thickness)
  end
  gl.PolygonMode(GL.FRONT_AND_BACK, GL.LINE)
  gl.Culling(GL.FRONT)
  gl.DepthMask(false)
  glColor(0,0,0,1)

  glPushMatrix()
  glResetMatrices()
  DrawVisibleUnits(true)
  glPopMatrix()

  gl.LineWidth(1.0)
  glColor(1,1,1,1)
  gl.Culling(false)
  gl.PolygonMode(GL.FRONT_AND_BACK, GL.FILL)
  gl.DepthTest(GL.LESS)

  if underwater then
    gl.PolygonOffset(0.0, 0.0)
  end
end

local blur_h = function()
  glClear(GL_COLOR_BUFFER_BIT,0,0,0,0)
  glUseShader(blurShader_h)
  glTexRect(-1-0.5/vsx,1+0.5/vsy,1+0.5/vsx,-1-0.5/vsy)
end

local blur_v = function()
  glClear(GL_COLOR_BUFFER_BIT,0,0,0,0)
  glUseShader(blurShader_v)
  glTexRect(-1-0.5/vsx,1+0.5/vsy,1+0.5/vsx,-1-0.5/vsy)
end

function widget:DrawWorldPreUnit()
  if (options.lowQualityOutlines.value or forceLowQuality) then
    DrawVisibleUnitsLines(false)
  else
    glCopyToTexture(depthtex, 0, 0, 0, 0, vsx, vsy)
    glTexture(depthtex)

    if (resChanged) then
      resChanged = false
      if (vsx==1) or (vsy==1) then return end
      glUseShader(depthShader)
      glUniform(uniformScreenXY,   vsx,vsy )
       glUseShader(blurShader_h)
      glUniformInt(uniformScreenX, vsx )
       glUseShader(blurShader_v)
      glUniformInt(uniformScreenY, vsy )
    end

    glUseShader(depthShader)
    glRenderToTexture(offscreentex,MyDrawVisibleUnits)

    glTexture(offscreentex)
    glRenderToTexture(blurtex, blur_v)
    glTexture(blurtex)
    glRenderToTexture(offscreentex, blur_h)

    glCallList(enter2d)
    glTexture(offscreentex)
    glTexRect(-1-0.5/vsx,1+0.5/vsy,1+0.5/vsx,-1-0.5/vsy)
    glCallList(leave2d)
  end
end

function widget:DrawWorldRefraction()
  DrawVisibleUnitsLines(true)
end 

function widget:UnitCreated(unitID)
  unbuiltUnits[unitID] = true
end

function widget:UnitDestroyed(unitID)
  unbuiltUnits[unitID] = nil
end


--------------------------------------------------------------------------------
--------------------------------------------------------------------------------
