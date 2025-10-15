-- jest-companion inserted this file

local getRelativePath = require((script.Parent :: any).getRelativePath)

local CoreScriptSyncService = {}

function CoreScriptSyncService:GetScriptFilePath(path)
	return getRelativePath(path, game:FindFirstChild("jest-companion", true))
end

return CoreScriptSyncService
