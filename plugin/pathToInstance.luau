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

return pathToInstance
