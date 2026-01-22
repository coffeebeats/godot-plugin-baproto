##
## import/platform.gd
##
## Platform detection and binary resolution utility. Provides static methods to identify
## the current platform, construct binary names, and locate the compiler binary.
##

extends RefCounted

# -- DEPENDENCIES -------------------------------------------------------------------- #

const ProjectSetting := preload("./setting.gd")

# -- CONFIGURATION ------------------------------------------------------------------- #

static var _cached_path: String = ""
static var _cache_initialized: bool = false

# -- PUBLIC METHODS ------------------------------------------------------------------ #


## `clear_cache` clears the cached binary path, forcing the next `resolve()` call to
## search again. Useful for testing or when the binary location changes.
static func clear_cache() -> void:
	_cached_path = ""
	_cache_initialized = false


## `get_architecture` detects the CPU architecture and returns the normalized name used
## in binary paths (e.g., "x86_64", "arm64").
static func get_architecture() -> String:
	if OS.has_feature("x86_64"):
		return "x86_64"
	elif OS.has_feature("arm64"):
		return "arm64"
	else:
		push_warning("[baproto] Unsupported architecture")
		return ""


## `get_os_name` detects the operating system and returns the normalized name used in
## binary paths (e.g., "darwin", "linux", "windows").
static func get_os_name() -> String:
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


## `resolve_binary_path` locates the `baproto-gdscript` binary using multiple resolution
## strategies. Returns the absolute filesystem path to the binary, or an empty string if
## not found.
##
## Resolution priority:
##   1. ProjectSettings override: baproto/binary/path (if set and valid)
##   2. res://addons/baproto/bin/{platform}/baproto-gdscript[.exe] (production)
##   3. res://addons/baproto/.target/debug/baproto-gdscript[.exe] (development)
##   4. res://addons/baproto/.target/release/baproto-gdscript[.exe] (local release)
static func resolve_binary_path() -> String:
	if _cache_initialized:
		if FileAccess.file_exists(_cached_path):
			return _cached_path

		_cache_initialized = false

	var checked_locations: Array[String] = []
	var resolved_path := ""

	# Strategy 0: ProjectSettings override
	var path_override: String = ProjectSetting.binary_path().get_value()
	if not path_override.is_empty():
		checked_locations.append(path_override)
		resolved_path = _check_path(path_override)
		if not resolved_path.is_empty():
			_cached_path = resolved_path
			_cache_initialized = true
			return resolved_path

	var binary_name := _get_binary_name()
	if binary_name.is_empty():
		push_error("[baproto] Cannot resolve binary: unsupported platform")
		_cache_initialized = false
		_cached_path = ""
		return ""

	# Strategy 1: Production deployment in bin/{platform}/
	var platform_dir := _get_platform_dir()
	if not platform_dir.is_empty():
		var production_path := "res://addons/baproto/bin/%s/%s" % [platform_dir, binary_name]
		checked_locations.append(production_path)
		resolved_path = _check_path(production_path)
		if not resolved_path.is_empty():
			print("[baproto] Using production binary: %s" % resolved_path)
			_cached_path = resolved_path
			_cache_initialized = true
			return resolved_path

	# Strategy 2: Development build in .target/debug/
	var debug_path := "res://addons/baproto/.target/debug/%s" % binary_name
	checked_locations.append(debug_path)
	resolved_path = _check_path(debug_path)
	if not resolved_path.is_empty():
		print("[baproto] Using debug binary: %s" % resolved_path)
		_cached_path = resolved_path
		_cache_initialized = true
		return resolved_path

	# Strategy 3: Local release build in .target/release/
	var release_path := "res://addons/baproto/.target/release/%s" % binary_name
	checked_locations.append(release_path)
	resolved_path = _check_path(release_path)
	if not resolved_path.is_empty():
		print("[baproto] Using release binary: %s" % resolved_path)
		_cached_path = resolved_path
		_cache_initialized = true
		return resolved_path

	# Binary not found in any location
	push_error(
		"[baproto] Binary not found (searching for: %s). Checked locations:\n  %s"
		% [binary_name, "\n  ".join(checked_locations)]
	)

	return ""

# -- PRIVATE METHODS ----------------------------------------------------------------- #


## `_check_path` verifies if a file exists at the specified resource path and returns
## the absolute filesystem path if found or an empty string if not.
static func _check_path(path: String) -> String:
	path = ProjectSettings.globalize_path(path)

	if FileAccess.file_exists(path):
		return path

	return ""


## `_get_platform_dir` returns the platform-specific directory name used in the 'bin/'
## directory structure (e.g. "darwin-arm64" or "linux-x86_64").
static func _get_platform_dir() -> String:
	var os := get_os_name()
	if os.is_empty():
		return ""

	var arch := get_architecture()
	if arch.is_empty():
		return ""

	return "%s-%s" % [os, arch]


## `_get_binary_name` returns the simple binary name without platform suffix. Just
## "baproto-gdscript" or "baproto-gdscript.exe" on Windows.
static func _get_binary_name() -> String:
	var os := get_os_name()
	if os.is_empty():
		return ""

	var binary_name := "baproto-gdscript"
	if os == "windows":
		binary_name += ".exe"

	return binary_name