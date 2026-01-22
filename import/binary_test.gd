##
## binary_test.gd
##
## Test suite for binary resolution utilities.
##

extends GutTest

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Binary := preload("./binary.gd")
const Platform := preload("./platform.gd")

# -- TEST METHODS -------------------------------------------------------------------- #


func test_platform_get_binary_name_returns_non_empty() -> void:
	# Given: The current platform.
	# When: Getting the binary name.
	var binary_name := Platform.get_binary_name()

	# Then: A non-empty string is returned.
	assert_false(binary_name.is_empty(), "Binary name should not be empty for supported platforms")


func test_platform_get_binary_name_includes_platform() -> void:
	# Given: The current platform.
	# When: Getting the binary name.
	var binary_name := Platform.get_binary_name()

	# Then: The name includes platform identifiers.
	assert_true(
		binary_name.contains("darwin") or binary_name.contains("linux") or binary_name.contains("windows"),
		"Binary name should contain OS identifier"
	)
	assert_true(
		binary_name.contains("x86_64") or binary_name.contains("arm64"),
		"Binary name should contain architecture identifier"
	)


func test_platform_get_binary_name_windows_has_exe_extension() -> void:
	# Given: The current OS is Windows.
	if OS.get_name() != "Windows":
		pass_test("Skipping Windows-specific test on non-Windows platform")
		return

	# When: Getting the binary name.
	var binary_name := Platform.get_binary_name()

	# Then: The name ends with .exe.
	assert_true(binary_name.ends_with(".exe"), "Windows binary should have .exe extension")


func test_platform_get_binary_name_non_windows_no_extension() -> void:
	# Given: The current OS is not Windows.
	if OS.get_name() == "Windows":
		pass_test("Skipping non-Windows test on Windows platform")
		return

	# When: Getting the binary name.
	var binary_name := Platform.get_binary_name()

	# Then: The name does not end with .exe.
	assert_false(binary_name.ends_with(".exe"), "Non-Windows binary should not have .exe extension")


func test_binary_resolve_returns_path_or_empty() -> void:
	# Given: The binary resolution system.
	# When: Resolving the binary path.
	Binary.clear_cache()
	var resolved_path := Binary.resolve()

	# Then: Either a path is returned or an empty string.
	# Note: We can't assert which, as it depends on the test environment.
	assert_true(
		resolved_path.is_empty() or FileAccess.file_exists(resolved_path),
		"Resolved path should be empty or point to an existing file"
	)


func test_binary_resolve_caches_result() -> void:
	# Given: The binary has been resolved once.
	Binary.clear_cache()
	var first_result := Binary.resolve()

	# When: Resolving again without clearing cache.
	var second_result := Binary.resolve()

	# Then: The same result is returned.
	assert_eq(second_result, first_result, "Cached result should be returned")


func test_binary_clear_cache_forces_new_resolution() -> void:
	# Given: The binary has been resolved and cached.
	Binary.clear_cache()
	var first_result := Binary.resolve()

	# When: Clearing the cache and resolving again.
	Binary.clear_cache()
	var second_result := Binary.resolve()

	# Then: Resolution happens again (results should still match).
	assert_eq(second_result, first_result, "Re-resolved path should match original")


# -- TEST HOOKS ---------------------------------------------------------------------- #


func before_all() -> void:
	# NOTE: Hide unactionable errors when using object doubles.
	ProjectSettings.set("debug/gdscript/warnings/native_method_override", false)


func before_each() -> void:
	# Clear cache before each test for isolation.
	Binary.clear_cache()
