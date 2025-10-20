--!strict

local ReplicatedStorage = game:GetService("ReplicatedStorage")
local HttpService = game:GetService("HttpService")

local hotReload = require(script.hotReload)
local pathToInstance = require(script.pathToInstance)

function getJest(): any
	local Jest = ReplicatedStorage:FindFirstChild("Jest", true)
	assert(Jest, "Couldn't find Jest!")
	assert(Jest:IsA("ModuleScript"), "Jest is not a ModuleScript!")

	patchJest(Jest)
	hotReload.flush()

	local mod = hotReload.require(Jest)
	return mod
end

function patchJest(Jest: ModuleScript)
	local parent = Jest.Parent
	assert(parent, "Jest has no parent")

	local RobloxShared = parent:FindFirstChild("RobloxShared")
	assert(RobloxShared, "Jest has no RobloxShared next to it!")

	local originalGetDataModelService = RobloxShared:FindFirstChild("getDataModelService")
	assert(originalGetDataModelService, "RobloxShared has no getDataModelService module under it")

	originalGetDataModelService:Destroy()

	local getDataModelService = script.patch.getDataModelService:Clone()
	getDataModelService.Parent = RobloxShared :: any

	local originalFileSystemService = RobloxShared:FindFirstChild("FileSystemService")
	if originalFileSystemService then
		originalFileSystemService:Destroy()
	end

	local FileSystemService = script.patch.FileSystemService:Clone()
	FileSystemService.Parent = RobloxShared :: any

	local originalCoreScriptSyncService = RobloxShared:FindFirstChild("CoreScriptSyncService")
	if originalCoreScriptSyncService then
		originalCoreScriptSyncService:Destroy()
	end

	local CoreScriptSyncService = script.patch.CoreScriptSyncService:Clone()
	CoreScriptSyncService.Parent = RobloxShared :: any
end

type TestBody = {
	projects: { string },
	options: {
		[string]: unknown?,
	},
}

type TestOutput = {}

function runTests(body: TestBody): {}
	local projects: { Instance } = {}

	for _, project in body.projects do
		local instance = pathToInstance(project)
		assert(instance, `Couldn't find instance for '{project}'`)

		table.insert(projects, instance)
	end

	body.options.setupFiles = { script.setup }
	body.options.reporters = {}

	local Jest = getJest()
	return Jest.runCLI(script, body.options, projects):expect()
end

local URL = "http://127.0.0.1:28860"

function poll(): TestBody?
	local reqSuccess, response = pcall(function()
		return HttpService:RequestAsync({
			Url = `{URL}/poll`,
			Compress = Enum.HttpCompression.None,
			Headers = { ["Content-Type"] = "application/json" },
		})
	end)

	if not reqSuccess then
		-- They aren't running the CLI
		return
	end

	if not response.Success then
		error(`Server rejected poll: ({response.StatusCode})\n{response.Body}`)
	end

	if not response.Body then
		error("Server didn't send back anything")
	end

	return HttpService:JSONDecode(response.Body) :: TestBody
end

function sendOutput(output: TestOutput)
	local reqSuccess, response = pcall(function()
		return HttpService:RequestAsync({
			Url = `{URL}/output`,
			Method = "POST",
			Body = HttpService:JSONEncode(output),
			Headers = { ["Content-Type"] = "application/json" },
			Compress = Enum.HttpCompression.None,
		})
	end)

	if not reqSuccess then
		error(`Failed to send output to server: {response.StatusCode}\n{response.Body}`)
	end

	if not response.Success then
		error(`Server rejected output: {response.StatusCode}\n{response.Body}`)
	end
end

function loop()
	local body: TestBody
	do
		local success, bodyOrErr = pcall(poll)
		if not success then
			warn(`Failed to poll server: {bodyOrErr}`)
			return
		end

		if not bodyOrErr then
			return
		end

		body = bodyOrErr
	end

	local output: TestOutput
	do
		local success, outputOrErr = pcall(runTests, body)
		if not success then
			warn(`Test run error: {outputOrErr}`)
			return
		end

		output = outputOrErr
	end

	local success, err = pcall(function()
		sendOutput(output)
	end)
	if not success then
		warn(`Failed to send output: {err}`)
	end
end

while true do
	loop()
	task.wait(0.5)
end
