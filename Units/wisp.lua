local Wisp = Unit:New {
	acceleration        = 0.5,
	brakeRate           = 0.9,
    --buildCostMetal        = 65, -- used only for power XP calcs
    canMove             = true,
--     canGuard            = false,
--     canPatrol           = false,
--     canRepeat           = false,
    category            = "INFANTRY",

    --pushResistant       = true,
    collisionVolumeScales   = '37 40 37',
    collisionVolumeTest     = 1,
    collisionVolumeType     = 'CylY',
    footprintX          = 2,
    footprintZ          = 2,
    mass                = 50,
    minCollisionSpeed   = 1,
    movementClass       = "Wisp", -- TODO: --KBOT
    repairable          = false,
    sightDistance       = 800,


    stealth             = true,
    turnRate            = 5000,
    upright             = true,

	
    name                = "Wisp",
    activateWhenBuilt   = true,
    customParams = {
        player = true,
        radius = 20,
		plate_toggler = true,
		wisp   = true,
    },

    idletime = 120, --in simframes
    idleautoheal = 50,
    autoheal = 1,    

    maxDamage           = 1600,
    maxVelocity         = 5,
    onoffable           = true,
    fireState           = 0,
    moveState           = 0,
    script              = "wisp.lua",
	objectName 			= "wisp.dae",
}

NPCWisp = Wisp:New {
	name 				= "NPCWisp",
	customParams = {
        player = false,
		plate_toggler = false,
		wisp   = true,
    },
}


return {
    Wisp    = Wisp,
	NPCWisp = NPCWisp,
}
