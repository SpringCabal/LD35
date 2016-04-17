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
		bodypart = "eyes",
	},
	objectName 		    = "eye.dae",
}
local Ears = BaseEffect:New {
	name                = "Ears",
	customParams        = {
		bodypart = "ears",
	},
	objectName 		    = "ear.dae",
}
local Nose = BaseEffect:New {
	name                = "Nose",
	customParams        = {
		bodypart = "nose",
	},
	objectName 		    = "nose.dae",
}
local Mouth = BaseEffect:New {
	name                = "Mouth",
	customParams        = {
		bodypart = "mouth",
	},
	objectName 		    = "mouth.dae",
}
local Arms = BaseEffect:New {
	name                = "Arms",
	customParams        = {
		bodypart = "arms",
	},
	objectName 		    = "hand.dae",
}
local Legs = BaseEffect:New {
	name                = "Legs",
	customParams        = {
		bodypart = "legs",
	},
}

local Plate = BaseEffect:New {
	name                = "Plate",
	objectName 			= "plate.dae",
	onoffable           = true,
	script              = "plate.lua",
}

local Lever = BaseEffect:New {
	name                = "Lever",
	objectName 			= "lever.dae",
	onoffable           = true,
	script              = "lever.lua",
}

local Laser = BaseEffect:New {
	name                = "Laser ",
	objectName 			= "laser.dae",
}

local Target = BaseEffect:New {
	name                = "Target",
	objectName 			= "target.dae",
}

local Door = BaseEffect:New {
	name                = "Door",
	objectName 			= "door.dae",
}


return {
	Eyes  = Eyes,
	Ears  = Ears,
	Nose  = Nose,
	Mouth = Mouth,
	Arms  = Arms,
	Legs  = Legs,
	
	Plate = Plate,
	Lever = Lever,
	Laser = Laser,
	Target = Target,
	Door = Door,
}
