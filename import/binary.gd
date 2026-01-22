##
## binary.gd
##
## Binary discovery utility for locating the baproto-gdscript compiler binary.
## Searches multiple locations in priority order and caches the result.
##

extends RefCounted

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Platform := preload("./platform.gd")

# -- CONFIGURATION ------------------------------------------------------------------- #

static var _cached_path: String = ""
static var _cache_initialized: bool = false

# -- PUBLIC METHODS ------------------------------------------------------------------ #

## `resolve` locates the baproto-gdscript binary using multiple resolution
## strategies. Returns the absolute filesystem path to the binary, or an empty
## string if not found.
##
## Resolution priority:
##   1. res://addons/baproto/bin/{platform}/baproto-gdscript[.exe] (production)
##   2. res://addons/baproto/.target/debug/baproto-gdscript (development)
##   3. res://addons/baproto/.target/release/baproto-gdscript (local release)
##
## NOTE: The user's PATH is not searched, as there doesn't seem to be an easy way to
static func resolve() -> String:
	if _cache_initialized:
		return _cached_path

	var binary_name := Platform.get_binary_name()
	if binary_name.is_empty():
		push_error("[baproto] Cannot resolve binary: unsupported platform")
		_cache_initialized = true
		_cached_path = ""
		return ""

	var checked_locations: Array[String] = []
	var resolved_path := ""

	# Strategy 1: Production deployment in bin/{platform}/
	var production_path := "res://addons/baproto/bin/%s/%s" % [
		_get_platform_dir(),
		binary_name
	]
	checked_locations.append(production_path)
	resolved_path = _check_path(production_path)
	if not resolved_path.is_empty():
		_cached_path = resolved_path
		_cache_initialized = true
		return resolved_path

	# Strategy 2: Development build in .target/debug/
	var debug_path := "res://addons/baproto/.target/debug/baproto-gdscript"
	if OS.get_name() == "Windows":
		debug_path += ".exe"
	checked_locations.append(debug_path)
	resolved_path = _check_path(debug_path)
	if not resolved_path.is_empty():
		_cached_path = resolved_path
		_cache_initialized = true
		return resolved_path

	# Strategy 3: Local release build in .target/release/
	var release_path := "res://addons/baproto/.target/release/baproto-gdscript"
	if OS.get_name() == "Windows":
		release_path += ".exe"
	checked_locations.append(release_path)
	resolved_path = _check_path(release_path)
	if not resolved_path.is_empty():
		_cached_path = resolved_path
		_cache_initialized = true
		return resolved_path

	# Binary not found in any location
	push_error("[baproto] Binary not found. Checked locations:\n  %s" % "\n  ".join(checked_locations))
	_cache_initialized = true
	_cached_path = ""
	return ""

## `clear_cache` clears the cached binary path, forcing the next `resolve()` call
## to search again. Useful for testing or when the binary location changes.
static func clear_cache() -> void:
	_cached_path = ""
	_cache_initialized = false

# -- PRIVATE METHODS ------------------------------------------------------------- #

## `_get_platform_dir` returns the platform-specific directory name used in the
## bin/ directory structure.
static func _get_platform_dir() -> String:
	var os_name := OS.get_name()
	var os := ""

	match os_name:
		"macOS":
			os = "darwin"
		"Linux":
			os = "linux"
		"Windows":
			os = "windows"
		_:
			return ""

	var arch := ""
	if OS.has_feature("x86_64"):
		arch = "x86_64"
	elif OS.has_feature("arm64"):
		arch = "arm64"
	else:
		return ""

	return "%s-%s" % [os, arch]

## `_check_path` verifies if a file exists at the given res:// path and returns
## the absolute filesystem path if found, or an empty string if not.
static func _check_path(res_path: String) -> String:
	var fs_path := ProjectSettings.globalize_path(res_path)
	if FileAccess.file_exists(fs_path):
		return fs_path
	return ""
