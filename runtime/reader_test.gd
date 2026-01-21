##
## reader_test.gd
##
## Test suite for `Reader` class. Tests error handling, navigation, and read
## capabilities.
##

extends GutTest

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Reader := preload("res://runtime/reader.gd")

# -- TEST METHODS -------------------------------------------------------------------- #


func test_reader_can_read_bits() -> void:
	# Given: A reader with 2 bytes.
	var reader := Reader.new(PackedByteArray([0xFF, 0xFF]))

	# Then: can_read_bits returns correct values.
	assert_true(reader.can_read_bits(1))
	assert_true(reader.can_read_bits(16))
	assert_false(reader.can_read_bits(17))


func test_reader_can_read_bytes() -> void:
	# Given: A reader with 3 bytes.
	var reader := Reader.new(PackedByteArray([0x11, 0x22, 0x33]))

	# Then: can_read_bytes returns correct values.
	assert_true(reader.can_read_bytes(1))
	assert_true(reader.can_read_bytes(3))
	assert_false(reader.can_read_bytes(4))


func test_reader_read_past_end_bool() -> void:
	# Given: An empty reader.
	var reader := Reader.new(PackedByteArray())

	# When: Reading past the end.
	var result := reader.read_bool()

	# Then: Returns false and sets error.
	assert_false(result)
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_reader_read_past_end_bits() -> void:
	# Given: A reader with 1 byte.
	var reader := Reader.new(PackedByteArray([0xFF]))

	# When: Reading more bits than available.
	var result := reader.read_bits(16)

	# Then: Returns 0 and sets error.
	assert_eq(result, 0)
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_reader_read_past_end_bytes() -> void:
	# Given: A reader with 2 bytes.
	var reader := Reader.new(PackedByteArray([0x11, 0x22]))

	# When: Reading more bytes than available.
	var result := reader.read_bytes(5)

	# Then: Returns empty and sets error.
	assert_eq(result.size(), 0)
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_reader_read_bits_invalid_count() -> void:
	# Given: A reader.
	var reader := Reader.new(PackedByteArray([0xFF, 0xFF]))

	# When: Reading with invalid bit count (0 or >64).
	var result1 := reader.read_bits(0)

	# Then: Returns 0 and sets error.
	assert_eq(result1, 0)
	assert_false(reader.is_valid())

# -- TEST HOOKS ---------------------------------------------------------------------- #


func before_all() -> void:
	# NOTE: Hide unactionable errors when using object doubles.
	ProjectSettings.set("debug/gdscript/warnings/native_method_override", false)