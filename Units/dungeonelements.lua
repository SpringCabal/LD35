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
	levelGround       		= false,
	customParams = {
		wall = true,
		dungeonElement = true, -- Make sure all non interactable items have this as true
	}
}

local gateSmoth = Element:New {
	name                = "gateSmoth",
	objectName 			= "gateSmoth.dae",
	footprintX				= 3,
	footprintZ				= 3,
	script					= "gate.lua",
	onoffable				= true,
	customParams = {
		wall = false,
		dungeonElement = true,
		triggerable = true,
	},
	collisionVolumeType		= "box",
	collisionVolumeScales	= "116 115 271",
	collisionVolumeOffsets	= "-6 60 0",
	
	canMove				= true,
    maxVelocity         = 0.00001,
	pushResistant       = true,
	
	movementClass		= "Wisp", 
}

local wall1 = Element:New {
	name                = "wall1",
	objectName 			= "wall1.dae",
	footprintX				= 3,
	footprintZ				= 3,
}

local wall2 = Element:New {
	name                = "wall2",
	objectName 			= "wall2.dae",
	footprintX				= 3,
	footprintZ				= 3,
}
local wall3 = Element:New {
	name                = "wall3",
	objectName 			= "wall3.dae",
	footprintX				= 3,
	footprintZ				= 3,
}
local wall4 = Element:New {
	name                = "wall4",
	objectName 			= "wall4.dae",
	footprintX				= 3,
	footprintZ				= 3,
}
local wall4 = Element:New {
	name                = "wall4",
	objectName 			= "wall4.dae",
	footprintX				= 3,
	footprintZ				= 3,
}
local wall5 = Element:New {
	name                = "wall5",
	objectName 			= "wall5.dae",
	footprintX				= 3,
	footprintZ				= 3,
}
local wall6 = Element:New {
	name                = "wall6",
	objectName 			= "wall6.dae",
	footprintX				= 3,
	footprintZ				= 3,
}
local wall7 = Element:New {
	name                = "wall7",
	objectName 			= "wall7.dae",
	footprintX				= 3,
	footprintZ				= 3,
}
local wall8 = Element:New {
	name                = "wall8",
	objectName 			= "wall8.dae",
	footprintX				= 3,
	footprintZ				= 3,
}

local corner1 = Element:New {
	name                = "corner1",
	objectName 			= "corner1.dae",
	footprintX				=  3,
	footprintZ				=  3,
}
local corner2 = Element:New {
	name                = "corner2",
	objectName 			= "corner2.dae",
	footprintX				=  3,
	footprintZ				=  3,
}

local corner3 = Element:New {
	name                = "corner3",
	objectName 			= "corner3.dae",
	footprintX				=  3,
	footprintZ				=  3,
}

local corner4 = Element:New {
	name                = "corner4",
	objectName 			= "corner4.dae",
	footprintX				=  3,
	footprintZ				=  3,
}

local corner5 = Element:New {
	name                = "corner5",
	objectName 			= "corner5.dae",
	footprintX				=  3,
	footprintZ				=  3,
}

local corner6 = Element:New {
	name                = "corner6",
	objectName 			= "corner6.dae",
	footprintX				=  3,
	footprintZ				=  3,
}

local floor1 = Element:New {
	name                = "floor",
	objectName 			= "floor.dae",
	footprintX				= 11,
	footprintZ				= 11,
}

local vertical_wall1		= Element:New {
	name                = "vertical_wall1",
	objectName 			= "vertical_wall1.dae",
	footprintX				= 4,
	footprintZ				= 11,
}

local vertical_wall2		= Element:New {
	name                = "vertical_wall2",
	objectName 			= "vertical_wall2.dae",
	footprintX				= 4,
	footprintZ				= 11,
}

local vertical_wall3		= Element:New {
	name                = "vertical_wall3",
	objectName 			= "vertical_wall3.dae",
	footprintX				= 4,
	footprintZ				= 11,
}

local vertical_wall4		= Element:New {
	name                = "vertical_wall4",
	objectName 			= "vertical_wall4.dae",
	footprintX				= 4,
	footprintZ				= 11,
}

local vertical_wall5		= Element:New {
	name                = "vertical_wall5",
	objectName 			= "vertical_wall5.dae",
	footprintX				= 4,
	footprintZ				= 11,
}

local horizontal_wall1		= Element:New {
	name                = "horizontal_wall1",
	objectName 			= "horizontal_wall1.dae",
	footprintX				= 11,
	footprintZ				= 4,
}
local horizontal_wall2		= Element:New {
	name                = "horizontal_wall2",
	objectName 			= "horizontal_wall2.dae",
	footprintX				= 11,
	footprintZ				= 4,
}
local horizontal_wall3		= Element:New {
	name                = "horizontal_wall3",
	objectName 			= "horizontal_wall3.dae",
	footprintX				= 11,
	footprintZ				= 4,
}
local horizontal_wall4		= Element:New {
	name                = "horizontal_wall4",
	objectName 			= "horizontal_wall4.dae",
	footprintX				= 11,
	footprintZ				= 4,
}

local horizontal_wall5		= Element:New {
	name                = "horizontal_wall5",
	objectName 			= "horizontal_wall5.dae",
	footprintX				= 11,
	footprintZ				= 4,
}

local tank		= Element:New {
	name                = "tank",
	objectName 			= "tank.dae",
	footprintX				= 1,
	footprintZ				= 1,
}

local stool		= Element:New {
	name                = "stool",
	objectName 			= "stool.dae",
	footprintX				= 1,
	footprintZ				= 1,
}

local atable		= Element:New {
	name                = "a table",
	objectName 			= "atable.dae",
	footprintX				= 1,
	footprintZ				= 1,
}

local pipes		= Element:New {
	name                = "pipes",
	objectName 			= "pipes.dae",
	footprintX				= 1,
	footprintZ				= 1,
}

local Door = Element:New {
	name                = "Door",
	objectName 			= "door.dae",
	customParams = {
		triggerable = true,
	},
}

return {
	gateSmoth = gateSmoth,
	pipes = pipes,
	tank = tank,
	stool = stool,
	atable = atable,
	wall1 = wall1,
	wall2 = wall2,
	wall3 = wall3,
	wall4 = wall4,
	wall5 = wall5,
	wall6 = wall6,
	wall7 = wall7,
	wall8 = wall8,
	wall = wall,
	floor1 = floor1,
	corner1 = corner1,
	corner2 = corner2,
	corner3 = corner3,
	corner4 = corner4,
	corner5 = corner5,
	corner6 = corner6,
	vertical_wall1 = vertical_wall1,
	vertical_wall2 = vertical_wall2,
	vertical_wall3 = vertical_wall3,
	vertical_wall4 = vertical_wall4,
	vertical_wall5 = vertical_wall5,
	horizontal_wall1 = horizontal_wall1,
	horizontal_wall2 = horizontal_wall2,
	horizontal_wall3 = horizontal_wall3,
	horizontal_wall4 = horizontal_wall4,
	horizontal_wall5 = horizontal_wall5,
}

