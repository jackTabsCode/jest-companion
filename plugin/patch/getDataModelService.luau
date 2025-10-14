-- jest-companion modified this file

-- jest-companion patch start:
local FileSystemService = require((script.Parent :: any).FileSystemService)
local CoreScriptSyncService = require((script.Parent :: any).CoreScriptSyncService)
-- jest-companion patch end:

-- checks that the service exists and is accessible before returning it, otherwise returns nil
return function(service: string)
	-- jest-companion patch start:
	if service == "FileSystemService" then
		return FileSystemService
	elseif service == "CoreScriptSyncService" then
		return CoreScriptSyncService
	end
	-- jest-companion patch end:

	local success, result = pcall(function()
		local service = game:GetService(service)
		local _ = service.Name
		return service
	end)

	return success and result or nil
end
