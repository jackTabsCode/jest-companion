-- jest-companion inserted this file

local getRelativePath = require((script.Parent :: any).getRelativePath)

local CoreScriptSyncService = {}

function CoreScriptSyncService:GetScriptFilePath(path)
	print(`CoreScriptSyncService:GetScriptFilePath({path})`)
	return getRelativePath(path)
end

return CoreScriptSyncService
