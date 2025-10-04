--!strict

local ReplicatedStorage = game:GetService("ReplicatedStorage")
local HttpService = game:GetService("HttpService")
local LogService = game:GetService("LogService")

function getJest(): any
	local Jest = ReplicatedStorage:FindFirstChild("Jest", true)
	assert(Jest, "Couldn't find Jest!")
	assert(Jest:IsA("ModuleScript"), "Jest is not a ModuleScript!")

	return Jest
end

type TestBody = {
	projects: { string },
	options: {},
}

function pathToInstance(path: string): Instance?
	local parts = path:split("/")

	local instance: Instance? = game:GetService(parts[1])
	for i = 2, #parts do
		if not instance then
			return nil
		end

		instance = instance:FindFirstChild(parts[i])
	end

	return instance
end

function runJest(body: TestBody): {}
	LogService:ClearOutput()

	local Jest = getJest()

	local projects: {Instance} = {}
	for _, project in body.projects do
		local instance = pathToInstance(project)
		if instance then
			table.insert(projects, instance)
		else
			warn(`Couldn't find instance for '{project}'`)
		end
	end

	return Jest:runCLI(script, body.options, projects)
end

local URL = "http://127.0.0.1:28860"

function loop()
	local success, response = pcall(function()
		return HttpService:RequestAsync({
			Url = `{URL}/poll`,
			Compress = Enum.HttpCompression.None,
			Headers = { ["Content-Type"] = "application/json" },
		})
	end)

	if not success then
		-- They aren't running the CLI
		return
	end

	if not response.Success then
		warn(`Poll request error from server: ({response.StatusCode}) {response.Body}`)
		return
	end

	if not response.Body then
		warn("There was no response body")
		return
	end

	local body = HttpService:JSONDecode(response.Body) :: TestBody
end

while true do
	loop()
	task.wait(1)
end
