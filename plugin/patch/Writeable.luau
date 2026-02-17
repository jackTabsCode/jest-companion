-- jest-companion inserted this file

local HttpService = game:GetService("HttpService")

export type Writeable = {
	isTTY: boolean?,
}

local Writeable = {}
Writeable.__index = Writeable

function Writeable.new(options: { write: (data: string) -> () }?): Writeable
	assert(
		options == nil or options.write == nil,
		"jest-companion does not support providing a `write` function to Writable"
	)
	local self = setmetatable({}, Writeable)
	self.isTTY = true
	return (self :: any) :: Writeable
end

function Writeable:write(data: string)
	task.spawn(function()
		local response = HttpService:RequestAsync({
			Url = `http://127.0.0.1:28860/write`,
			Method = "POST",
			Body = data,
			Headers = { ["Content-Type"] = "text/plain" },
			Compress = Enum.HttpCompression.None,
		})

		if not response.Success then
			error(`Server rejected write: {response.StatusCode}\n{response.Body}`)
		end
	end)
end

return {
	Writeable = Writeable,
}
