local body   = piece('Body')
local eyes   = piece('Eyes')
local ears   = piece('Ears')
local arml   = piece('ArmL')
local armr   = piece('ArmR')
local thighl = piece('ThighL')
local thighr = piece('ThighR')
local nose   = piece('Nose')
local mouth  = piece('Mouth')

local pieceNames = {
	body = body,
	eyes = eyes,
	ears = ears,
	arms = { arml, armr },
	legs = { thighl, thighr },
	nose = nose,
	mouth = mouth,
}

function script.Killed(recentDamage, maxHealth)
	return 0
end

function SetPieceVisible(name, visible)
	local pieces = pieceNames[name]
	if type(pieces) ~= "table" then
		if visible then
			Show(pieces)
		else
			Hide(pieces)
		end
	else
		for _, piece in pairs(pieces) do
			if visible then
				Show(piece)
			else
				Hide(piece)
			end
		end
	end
end

function SetPieceVisibleNoThread(name, visible)
	StartThread(SetPieceVisible, name, visible)
end

function SetAllPiecesInvisibleNoThread()
	for _, pieceID in pairs(Spring.GetUnitPieceMap(unitID)) do
		StartThread(function() Hide(pieceID) end)
	end
end
