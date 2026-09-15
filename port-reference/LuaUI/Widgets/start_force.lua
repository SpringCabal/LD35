

function widget:GetInfo()
	return {
		name    = 'Start Force',
		desc    = 'sends forcestart command',
		author  = 'Googlefrog',
		date    = '',
		license = 'GNU GPL v2',
        layer = 0,
		enabled = true,
	}
end

function widget:Initialize()
	Spring.SendCommands('forcestart')
end
