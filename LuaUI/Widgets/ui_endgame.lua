--------------------------------------------------------------------------------
--------------------------------------------------------------------------------

function widget:GetInfo()
  return {
    name      = "Endgame UI",
    desc      = "Simple engame UI for LD35",
    author    = "gajop",
    date      = "April 2016",
    license   = "GNU GPL, v2 or later",
    layer     = 0,
    enabled   = true,
  }
end

local Chili
local backgroundWindow

function widget:Initialize()
	if (not WG.Chili) then
		widgetHandler:RemoveWidget()
		return
	end

	Chili = WG.Chili
end

function widget:Update()
	if backgroundWindow and Spring.GetGameRulesParam("game_over") ~= 1 then
		backgroundWindow:Dispose()
		backgroundWindow = nil
	end
end

function ShowEndgameUI()
	if backgroundWindow then
		return
	end

	local captionTxt = ""
	game_over_type = Spring.GetGameRulesParam("game_over_type") or 0
	if game_over_type == 0 then
		captionTxt = "Welcome back, Master."
	elseif game_over_type == 1 then
		captionTxt = "The souls have been freed."
	elseif game_over_type == 2 then
		captionTxt = "Their sacrifice is accepted."
	elseif game_over_type == 3 then
		captionTxt = "You have achieved nothing noteworthy."
	end
	
	captionTxt = captionTxt .. "      Ending: #" .. tostring(game_over_type+1)
	
	backgroundWindow = Chili.Control:New {
        parent = Chili.Screen0,
        x = 0,
        y = 0,
        bottom = 0,
		right = 0,
        minHeight = 25,
		padding  = {0, 0, 0, 0},
		children = {
			Chili.Label:New {
				y = "45%",
				x = "45%",
				caption = captionTxt,
			},
			Chili.Button:New {
				y = "30%",
				right = 400,
				height = 50,
				width = 80,
				caption = "Restart",
				OnClick = { function() 
					if not Spring.IsCheatingEnabled() then
						Spring.SendCommands({"cheat", "luarules reload"}) 
					else
						Spring.SendCommands({"luarules reload"}) 
					end
				end },
			},
-- 			Chili.Button:New {
-- 				y = 400,
-- 				right = 400,
-- 				height = 80,
-- 				width = 300,
-- 				caption = "Hard",
-- 				OnClick = { function() StartGame(2) end },
-- 			},
-- 			Chili.Button:New {
-- 				y = 500,
-- 				right = 400,
-- 				height = 80,
-- 				width = 300,
-- 				caption = "Extreme",
-- 				OnClick = { function() StartGame(3) end },
-- 			},
			Chili.Button:New {
				y = "50%",
				right = 400,
				height = 50,
				width = 80,
				caption = "Leave",
				OnClick = { function() Spring.SendCommands("quitforce") end },
			},
-- 			Chili.Image:New {
-- 				x = 0,
-- 				y = 0,
-- 				width = "100%",
-- 				height = "100%",
-- 				file = "LuaUI/images/end_background.png",
-- 				keepAspect = false,
-- 			},
		},
    }
end

WG.ShowEndgameUI = ShowEndgameUI
