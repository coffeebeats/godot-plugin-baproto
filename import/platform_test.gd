##
## import/platform_test.gd
##
## Test suite for platform detection and binary resolution utilities.
##

extends GutTest

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Platform := preload("./platform.gd")

# -- TEST METHODS -------------------------------------------------------------------- #


func test_platform_clear_cache_forces_new_resolution() -> void:
	# Given: The binary has been resolved and cached.
	Platform.clear_cache()
	var first_result := Platform.resolve_binary_path()

	# When: Clearing the cache and resolving again.
	Platform.clear_cache()
	var second_result := Platform.resolve_binary_path()

	# Then: Resolution happens again (results should still match).
	assert_eq(second_result, first_result, "Re-resolved path should match original")


func test_platform_get_os_name_returns_supported_os() -> void:
	# Given: The current platform.
	# When: Getting the OS name.
	var os_name := Platform.get_os_name()

	# Then: A supported OS name is returned.
	assert_true(
		os_name == "darwin" or os_name == "linux" or os_name == "windows",
		"OS name should be one of: darwin, linux, windows"
	)


func test_platform_get_architecture_returns_supported_arch() -> void:
	# Given: The current platform.
	# When: Getting the architecture.
	var arch := Platform.get_architecture()

	# Then: A supported architecture is returned.
	assert_true(
		arch == "x86_64" or arch == "arm64",
		"Architecture should be either x86_64 or arm64"
	)


func test_platform_resolve_binary_path_returns_path_or_empty() -> void:
	# Given: The binary resolution system.
	# When: Resolving the binary path.
	Platform.clear_cache()
	var resolved_path := Platform.resolve_binary_path()

	# Then: Either a path is returned or an empty string.
	# Note: We can't assert which, as it depends on the test environment.
	assert_true(
		resolved_path.is_empty() or FileAccess.file_exists(resolved_path),
		"Resolved path should be empty or point to an existing file"
	)


func test_platform_resolve_binary_path_caches_result() -> void:
	# Given: The binary has been resolved once.
	Platform.clear_cache()
	var first_result := Platform.resolve_binary_path()

	# When: Resolving again without clearing cache.
	var second_result := Platform.resolve_binary_path()

	# Then: The same result is returned.
	assert_eq(second_result, first_result, "Cached result should be returned")


func test_platform_resolve_binary_path_returns_absolute_path() -> void:
	# Given: The binary resolution system.
	# When: Resolving the binary path.
	Platform.clear_cache()
	var resolved_path := Platform.resolve_binary_path()

	# Then: If a path is found, it should be absolute.
	if not resolved_path.is_empty():
		assert_true(
			resolved_path.is_absolute_path(),
			"Resolved path should be an absolute filesystem path"
		)


# -- TEST HOOKS ---------------------------------------------------------------------- #


func before_all() -> void:
	# NOTE: Hide unactionable errors when using object doubles.
	ProjectSettings.set("debug/gdscript/warnings/native_method_override", false)


func before_each() -> void:
	Platform.clear_cache()  # Ensure each test case is isolated.
