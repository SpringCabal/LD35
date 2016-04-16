function widget:GetInfo()
  return {
    name      = "Toggle button for testing/developing the game",
    desc      = "",
    author    = "gajop",
    date      = "in the future",
    license   = "GPL-v2",
    layer     = -10000,
    enabled   = true,
  }
end

local Chili, screen0
function widget:Initialize()
    gameMode = Spring.GetGameRulesParam("gameMode")
	if gameMode == "play" then
		return
	end

    if (not WG.Chili) then
		return
	end
	Chili = WG.Chili
	screen0 = Chili.Screen0

    MakeToggleButton()
end


local btnStartStop
local state

function MakeToggleButton()
    if (not WG.Chili) then
		return
	end

	btnStartStop = Chili.Button:New {
        caption='',
        bottom = 10,
        x = "45%",
		height = 50,
		width = 50,
		parent = screen0,
		padding = { 0,0,0,0 },
        OnClick = {
            function()
			    if not Spring.IsCheatingEnabled() then
					Spring.SendCommands("cheat")
				end
                if Spring.GetGameRulesParam("gameMode") == "develop" then
                   Spring.SendLuaRulesMsg('chonsole|gamerules|gameMode|test')
				elseif Spring.GetGameRulesParam("gameMode") == "test" then
                   Spring.SendLuaRulesMsg('chonsole|gamerules|gameMode|develop')
                end
            end
        }
    }
end

function widget:Update()
	if (not WG.Chili) then
		return
	end
    if state ~= "develop" and Spring.GetGameRulesParam("gameMode") == "develop" then
		state = "develop"
		btnStartStop:ClearChildren()
        btnStartStop.tooltip = "Start testing"
        btnStartStop:AddChild(
            Chili.Image:New {
                tooltip = "Start testing",
                file = "LuaUI/images/media-playback-start.png",
				x = 3,
				y = 3,
                height = 45,
				width = 45,
                margin = {0, 0, 0, 0},
            }
        )
    elseif state ~= "test" and Spring.GetGameRulesParam("gameMode") == "test" then
		btnStartStop:ClearChildren()
		state = "test"
        btnStartStop.tooltip = "Stop testing"
        btnStartStop:AddChild(
            Chili.Image:New {
                file = "LuaUI/images/media-playback-stop.png",
				x = 3,
				y = 3,
                height = 45,
				width = 45,
                margin = {0, 0, 0, 0},
            }
        )
    end
end
