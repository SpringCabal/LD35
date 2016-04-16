local BaseEffect = Unit:New {
	customParams        = {
		effect = true,
		-- invulnerable means that most instances are invulnerable through normal damage and effects (could still be manually destroyed)
		invulnerable = 1,
	},
	script					= "trigger.lua",
	objectName				= "trigger.dae",
	category				= "EFFECT",
	footprintX				= 1,
	footprintZ				= 1,
	mass					= 50,
	maxDamage				= 10000,
	collisionVolumeScales   = '0 0 0',
	collisionVolumeType     = 'cylY',
	pushResistant			= true,
	blocking				= false,
	canMove					= false, --effects cannot be moved (even by gravity)
	canGuard				= false,
	canPatrol				= false,
	canRepeat				= false,
	stealth					= true,
	turnRate				= 0,
	upright					= true,
	sightDistance			= 0,
--     canCloak            = true,
--     initCloaked         = true,
--     decloakOnFire       = false,
--     minCloakDistance    = 0,
}

local Eyes = BaseEffect:New {
	name                = "Eyes",
	customParams        = {
		bodyPart = "eyes",
	}
}
local Ears = BaseEffect:New {
	name                = "Ears",
	customParams        = {
		bodyPart = "ears",
	}
}
local Nose = BaseEffect:New {
	name                = "Nose",
	customParams        = {
		bodyPart = "nose",
	}
}
local Mouth = BaseEffect:New {
	name                = "Mouth",
	customParams        = {
		bodyPart = "mouth",
	}
}
local Arms = BaseEffect:New {
	name                = "Arms",
	customParams        = {
		bodyPart = "arms",
	}
}
local Legs = BaseEffect:New {
	name                = "Legs",
	customParams        = {
		bodyPart = "legs",
	}
}

return {
	Eyes  = Eyes,
	Ears  = Ears,
	Nose  = Nose,
	Mouth = Mouth,
	Arms  = Arms,
	Legs  = Legs,
}