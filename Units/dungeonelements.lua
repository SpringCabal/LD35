local Element = Unit:New {
	description				= "Scenery Element",
	collisionvolumetype		= "box",
	collisionvolumescales	= "110 50 172",
	collisionvolumeoffsets	= "0 0 0",		
	footprintX				= 11,
	footprintZ				= 11,	
	maxDamage				= 20000,
	maxSlope				= 3000,
	maxWaterDepth			= 0,
	name					= "Scenery Element",
	objectName				= "corner.dae",
	script			 		= "empty.lua",
	unitname           		= "Element",

}

local wall = Element:New {
	name                = "wall",
	objectName 			= "wall.dae",
	footprintX				= 3,
	footprintZ				= 3,
}

local corner = Element:New {
	name                = "corner",
	objectName 			= "corner.dae",
	footprintX				=  3,
	footprintZ				=  3,
}

local floor1 = Element:New {
	name                = "floor",
	objectName 			= "floor.dae",
	footprintX				= 11,
	footprintZ				= 11,
}

local vertical_doorway	= Element:New {
	name                = "vertical_doorway",
	objectName 			= "vertical_doorway.dae",
	footprintX				= 4,
	footprintZ				= 3,
}

local vertical_hall		= Element:New {
	name                = "vertical_hall",
	objectName 			= "vertical_hall.dae",
	footprintX				= 4,
	footprintZ				= 11,
}

local horzontal_doorway	= Element:New {
	name                = "horzontal_doorway",
	objectName 			= "horzontal_doorway.dae",
	footprintX				= 3,
	footprintZ				= 4,
}

local horzontal_hall		= Element:New {
	name                = "horzontal_hall",
	objectName 			= "horzontal_hall.dae",
	footprintX				= 11,
	footprintZ				= 4,
}

return {
	wall = wall,
	floor1 = floor1,
	corner = corner,
    vertical_doorway = vertical_doorway,
	vertical_hall = vertical_hall,
	horzontal_doorway = horzontal_doorway,
	horzontal_hall = horzontal_hall,
}

