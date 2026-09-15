function widget:GetInfo()
  return {
    name      = "Keybinds",
    desc      = "",
    author    = "Bluestone",
    date      = "in the future",
    license   = "GPL-v2",
    layer     = -10000,
    enabled   = true,
  }
end

local bindText, mouseText
local Chili, screen0
local children = {}
local x,y,h

local notPurple = "\255\202\202\101"
local white = "\255\255\255\255"

local function SetBindings()
    local binds = { --real keybinds
        
        "Any+pause  pause",
        
        --"Alt+b  debug",
        --"--Alt+v  debugcolvol",

		"ctrl+q quitforce",
		
		"f12 screenshot",
		
		"Any+up moveforward ",
		"Any+left moveleft ",
		"Any+right moveright",
		"Any+down moveback",
		
		"Any+w moveforward ",
		"Any+a moveleft ",
		"Any+d moveright",
		"Any+s moveback",
		
		"Any+pageup moveup",
		"Any+pagedown movedown",
    }

    for _,binding in pairs(binds) do
        Spring.SendCommands("bind ".. binding)
    end
end

local gameMode
local function SetGameMode(gameMode)
    if gameMode == "play" then
        Spring.SendCommands("unbindall") --muahahahaha
        Spring.SendCommands("unbindkeyset enter chat") --because because.
    else
        Spring.SendCommands("bindkeyset enter chat")
    end
    SetBindings()
end

local hasArms, hasEyes
local function SetBodyParts(hasEyes, hasArms)
	mouseText = {
        notPurple .. "Ctrl+Q : " .. white .. "Quit",
    }
	if hasEyes == 1 then
		mouseText[#mouseText+1] = notPurple .. "Space : " .. white .. "Switch form"
	end
	if hasArms == 1 then
		mouseText[#mouseText+1] = notPurple .. "Left click: " .. white .. "Interact"
	end
	MakeBindingText()
end

function widget:Initialize()
	bindText = { -- keybinds told to player
        --purple .. "Q : " .. white .. "swap pull / push",
        --purple .. "A : " .. white .. "stop shooting",
        --purple .. "W : " .. white .. "jump (+ left mouse)",
        --purple .. "S : " .. white .. "stop shooting & moving",
    }

    if (not WG.Chili) then
		return
	end
	Chili = WG.Chili
	screen0 = Chili.Screen0
	
    gameMode = Spring.GetGameRulesParam("gameMode")
    SetGameMode(gameMode)
	SetBodyParts(Spring.GetGameRulesParam("has_eyes"), Spring.GetGameRulesParam("has_arms"))
end


function MakeBindingText()
    if (not WG.Chili) then
		return
	end
    
    for _,child in pairs(children) do
        screen0:RemoveChild(child)
    end
    
    
    h = 20
    y = h*(#bindText + #mouseText)
    x = 10
    
    for _,text in ipairs(mouseText) do
        AddLine(text,x,y)
        y = y - h
    end    
    for _,text in ipairs(bindText) do
        AddLine(text,x,y)
        y = y - h
    end
end

function  AddLine(text,x,y,h)   
    children[#children+1] = Chili.Label:New{
        x = x,
        bottom = y,
        parent = screen0,
        caption = text,
    }
end

function widget:Update()
    local newGameMode = Spring.GetGameRulesParam("gameMode")
    if gameMode ~= newGameMode then
        gameMode = newGameMode
        SetGameMode(gameMode)
    end
	local newHasArms, newHasEyes = Spring.GetGameRulesParam("has_arms"), Spring.GetGameRulesParam("has_eyes")
	if hasArms ~= newHasArms or hasEyes ~= newHasEyes then
		hasArms = newHasArms
		hasEyes = newHasEyes
		SetBodyParts(hasEyes, hasArms)
	end
end
