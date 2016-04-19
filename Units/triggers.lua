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
	objectName 		    = "leg.dae",
}

local Plate = BaseEffect:New {
	name                = "Plate",
	objectName 			= "plate.dae",
	onoffable           = true,
	script              = "plate.lua",
	customParams = {
		trigger = true,
		stand_trigger = true,
	},
}

local Cage = BaseEffect:New {
	name                = "Cage",
	objectName 			= "cage.dae",
	onoffable           = true,
	script              = "cage.lua",
	customParams = {
		triggerable = true,
		effect = false,
	},
	blocking = true,
}

local Lever = BaseEffect:New {
	name                = "Lever",
	objectName 			= "lever.dae",
	onoffable           = true,
	script              = "lever.lua",
	customParams = {
		trigger = true,
	},
}

-- FIXME: cleanse
-- Probably don't need, who knows!
-- return {
--   areas = {},
--   features = {},
--   units = {
--     [18189] = {
--       defName = "cage",
--       paralyze = 0,
--       team = 0,
--       blocking = {
--         blockEnemyPushing = true,
--         blockHeightChanges = false,
--         crushable = false,
--         isBlocking = true,
--         isProjectileCollidable = true,
--         isRaySegmentCollidable = true,
--         isSolidObjectCollidable = true,
--       },
--       collision = {
--         axis = 1,
--         disabled = false,
--         offsetX = 0,
--         offsetY = 23,
--         offsetZ = 0,
--         scaleX = 64.5954666,
--         scaleY = 85.595459,
--         scaleZ = 64.5954666,
--         testType = 1,
--         vType = 1,
--       },
--       midAimPos = {
--         aim = {
--           x = 0.00024414,
--           y = 0,
--           z = 31.4523926,
--         },
--         mid = {
--           x = -0.9998779,
--           y = 21,
--           z = -6.5610352,
--         },
--       },
--       pos = {
--         x = 1039.70911,
--         y = 411.499023,
--         z = 2508.77148,
--       },
--       radiusHeight = {
--         height = 43.1672592,
--         radius = 53.7977295,
--       },
--       rot = {
--         x = 0,
--         y = 0.01340772,
--         z = 0,
--       },
--     },
--   },
-- }
local Light = BaseEffect:New {
	name                = "Light",
	objectName 		    = "eye.dae",
}


return {
	Eyes  = Eyes,
	Ears  = Ears,
	Nose  = Nose,
	Mouth = Mouth,
	Arms  = Arms,
	Legs  = Legs,
	
	Plate = Plate,
	Cage  = Cage,
	Lever = Lever,
	
	Light = Light,
}
