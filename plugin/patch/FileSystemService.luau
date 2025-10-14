-- jest-companion inserted this file

local FileSystemService = {}

function FileSystemService:WriteFile(path, contents)
	print(`FileSystemService:WriteFile({path}, {contents}`)
end

function FileSystemService:IsRegularFile(path)
	print(`FileSystemService:IsRegularFile({path})`)
	return true
end

function FileSystemService:CreateDirectories(path)
	print(`FileSystemService:CreateDirectories({path})`)
end

function FileSystemService:Exists(path)
	print(`FileSystemService:Exists({path})`)
	return true
end

function FileSystemService:Remove(path)
	print(`FileSystemService:Remove({path})`)
end

return FileSystemService
