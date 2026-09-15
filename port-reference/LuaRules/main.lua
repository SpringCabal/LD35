-- Wiki: http://springrts.com/wiki/Gamedev:Glossary#springcontent.sdz

-- Include base content gadget handler to run synced gadgets
local SCRIPT_DIR = Script.GetName() .. '/'
VFS.Include(SCRIPT_DIR .. 'utilities.lua', nil, VFSMODE)

VFS.Include("luagadgets/gadgets.lua",nil, VFS.BASE)
