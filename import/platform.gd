##
## platform.gd
##
## Platform detection utility for determining OS and architecture.
## Provides static methods to identify the current platform and construct
## platform-specific binary names.
##

extends RefCounted

# -- PUBLIC METHODS ------------------------------------------------------------------ #

## `get_binary_name` returns the platform-specific binary name for the current system,
## or an empty string if the platform is unsupported.
##
## Examples:
##   - macOS ARM64: "baproto-gdscript-darwin-arm64"
##   - Linux x86_64: "baproto-gdscript-linux-x86_64"
##   - Windows x86_64: "baproto-gdscript-windows-x86_64.exe"
static func get_binary_name() -> String:
	var os := _get_os_name()
	if os.is_empty():
		return ""

	var arch := _get_architecture()
	if arch.is_empty():
		return ""

	var binary_name := "baproto-gdscript-%s-%s" % [os, arch]

	# Windows requires .exe extension
	if os == "windows":
		binary_name += ".exe"

	return binary_name

# -- PRIVATE METHODS ----------------------------------------------------------------- #

## `_get_os_name` detects the operating system and returns the normalized name
## used in binary paths.
static func _get_os_name() -> String:
	var os_name := OS.get_name()

	match os_name:
		"macOS":
			return "darwin"
		"Linux":
			return "linux"
		"Windows":
			return "windows"
		_:
			push_warning("[baproto] Unsupported OS: %s" % os_name)
			return ""

## `_get_architecture` detects the CPU architecture and returns the normalized
## name used in binary paths.
static func _get_architecture() -> String:
	if OS.has_feature("x86_64"):
		return "x86_64"
	elif OS.has_feature("arm64"):
		return "arm64"
	else:
		push_warning("[baproto] Unsupported architecture")
		return ""
