local ArmL = piece('ArmL');
local ArmR = piece('ArmR');
local CrusL = piece('CrusL');
local CrusR = piece('CrusR');
local FootL = piece('FootL');
local FootR = piece('FootR');
local ForeArmL = piece('ForeArmL');
local ForeArmR = piece('ForeArmR');
local Hair1 = piece('Hair1');
local Hair2 = piece('Hair2');
local Hair3 = piece('Hair3');
local Hair4 = piece('Hair4');
local HandR = piece('HandR');
local Head = piece('Head');
local Eyes = piece('Eyes');
local Ears = piece('Ears');
local HandL = piece('HandL');
local Body = piece('Body');
local Mouth = piece('Mouth');
local Nose = piece('Nose');
local ThighL = piece('ThighL');
local ThighR = piece('ThighR');

local SIG_WALK =  tonumber("00001",2);
local SIG_IDLE =  tonumber("00010",2);

local floating = false;

local pieceNames = {
	body = Body,
	head = Head,
	eyes = Eyes,
	ears = Ears,
	arms = { ArmL, ArmR, ForeArmL, ForeArmR, HandL, HandR},
	legs = { ThighL, ThighR, CrusL, CrusR, FootL, FootR},
	nose = Nose,
	mouth = Mouth,
}

local scriptEnv = {	ArmL = ArmL,
	ArmR = ArmR,
	CrusL = CrusL,
	CrusR = CrusR,
	FootL = FootL,
	FootR = FootR,
	ForeArmL = ForeArmL,
	ForeArmR = ForeArmR,
	Hair1 = Hair1,
	Hair2 = Hair2,
	Hair3 = Hair3,
	Hair4 = Hair4,
	HandR = HandR,
	Head = Head,
	Eyes = Eyes,
	Ears = Ears,
	HandL = HandL,
	Body = Body,
	Mouth = Mouth,
	Nose = Nose,
	ThighL = ThighL,
	ThighR = ThighR,
	x_axis = x_axis,
	y_axis = y_axis,
	z_axis = z_axis,
}

-- you can include externally saved animations like this:
-- Animations['importedAnimation'] = VFS.Include("Scripts/animations/animationscript.lua", scriptEnv)
local Animations = {};
Animations['walk_legs'] = VFS.Include("Scripts/animations/wisp_walk_legs.lua", scriptEnv)
Animations['walk_arms'] = VFS.Include("Scripts/animations/wisp_walk_arms.lua", scriptEnv)
Animations['idle'] = VFS.Include("Scripts/animations/wisp_idle.lua", scriptEnv)
Animations['stop'] = VFS.Include("Scripts/animations/wisp_reset_all.lua", scriptEnv)


-- blender2lus infrastructure (should really be turned into an include one day)

function constructSkeleton(unit, piece, offset)
    if (offset == nil) then
        offset = {0,0,0};
    end

    local bones = {};
    local info = Spring.GetUnitPieceInfo(unit,piece);

    for i=1,3 do
        info.offset[i] = offset[i]+info.offset[i];
    end 

    bones[piece] = info.offset;
    local map = Spring.GetUnitPieceMap(unit);
    local children = info.children;

    if (children) then
        for i, childName in pairs(children) do
            local childId = map[childName];
            local childBones = constructSkeleton(unit, childId, info.offset);
            for cid, cinfo in pairs(childBones) do
                bones[cid] = cinfo;
            end
        end
    end        
    return bones;
end

local animCmd = {['turn']=Turn,['move']=Move};
function PlayAnimation(animname)
    local anim = Animations[animname];
    for i = 1, #anim do
        local commands = anim[i].commands;
        for j = 1,#commands do
            local cmd = commands[j];
            animCmd[cmd.c](cmd.p,cmd.a,cmd.t,cmd.s);
        end
        if(i < #anim) then
            local t = anim[i+1]['time'] - anim[i]['time'];
            Sleep(t*33); -- sleep works on milliseconds
        end
    end
end

-- utility stuff

function SetPieceVisible(name, visible)
	local pieces = pieceNames[name]
    if true then return end
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
	--StartThread(SetPieceVisible, name, visible)
end

function SetAllPiecesInvisibleNoThread()
	for _, pieceID in pairs(Spring.GetUnitPieceMap(unitID)) do
		--StartThread(function() Hide(pieceID) end)
	end
end

local function Walk()
	Signal(SIG_WALK)
	SetSignalMask(SIG_WALK)
	PlayAnimation("walk_legs", true);
	while true do
		PlayAnimation("walk_legs", false);
	end
end

local function Wave_Arms()
	SetSignalMask(SIG_WALK)
	PlayAnimation("walk_arms", true);
	while true do
		PlayAnimation("walk_arms", false);
	end
end

local function Stop()
	Signal(SIG_WALK)
	SetSignalMask(SIG_WALK)
	PlayAnimation("idle",true)
	--PlayAnimation("stop",false)
end

-- call-ins

function script.Create()
    local map = Spring.GetUnitPieceMap(unitID);
    local offsets = constructSkeleton(unitID,map.Scene, {0,0,0});
    
    for a,anim in pairs(Animations) do
        for i,keyframe in pairs(anim) do
            local commands = keyframe.commands;
            for k,command in pairs(commands) do
                -- commands are described in (c)ommand,(p)iece,(a)xis,(t)arget,(s)peed format
                -- the t attribute needs to be adjusted for move commands from blender's absolute values
                if (command.c == "move") then
                    local adjusted =  command.t - (offsets[command.p][command.a]);
                    Animations[a][i]['commands'][k].t = command.t - (offsets[command.p][command.a]);
                end
            end
        end
    end    
    PlayAnimation('idle');
end

function script.StartMoving()
	Signal(SIG_WALK);
	StartThread(Walk);
	StartThread(Wave_Arms);
end

function script.StopMoving()
	Signal(SIG_WALK);
	StartThread(Stop);
end


function script.Killed(recentDamage, maxHealth)
	return 0
end
        
            
