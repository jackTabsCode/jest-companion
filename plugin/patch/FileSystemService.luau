-- jest-companion inserted this file

local HttpService = game:GetService("HttpService")

local FileSystemService = {}

local URL = "http://127.0.0.1:28860/fs"

local function encodeSegments(p)
	local parts = string.split(p, "/")
	for i, seg in parts do
		parts[i] = HttpService:UrlEncode(seg)
	end
	return table.concat(parts, "/")
end

local function request(op, method, path, body)
	local encoded = encodeSegments(path)
	local url
	if op == "file" then
		url = `{URL}/file/{encoded}`
	elseif op == "dir" then
		url = `{URL}/dir/{encoded}`
	elseif op == "exists" then
		url = `{URL}/exists/{encoded}`
	else
		error(`unknown op {op}`)
	end

	local response = HttpService:RequestAsync({
		Url = url,
		Method = method,
		Body = body,
		Headers = { ["Content-Type"] = "text/plain" },
		Compress = Enum.HttpCompression.None,
	})

	if not response.Success then
		error(`Server rejected filesystem op:{response.StatusCode}\n{response.Body}`)
	end

	return response
end

function FileSystemService:WriteFile(path, contents)
	print("WriteFile", path)
	request("file", "PUT", path, contents)
end

function FileSystemService:CreateDirectories(path)
	print("CreateDirectories", path)
	request("dir", "PUT", path, "")
end

function FileSystemService:Exists(path)
	print("Exists", path)
	local resp = request("exists", "GET", path, "")
	return resp.Success
end

function FileSystemService:Remove(path)
	print("Remove", path)
	request("file", "DELETE", path, "")
end

function FileSystemService:IsRegularFile(path)
	print("IsRegularFile", path)
	return self:Exists(path)
end

return FileSystemService
