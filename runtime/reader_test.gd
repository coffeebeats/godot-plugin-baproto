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


func test_reader_varint_eof_restores_position() -> void:
	# Given: A reader with incomplete varint.
	var reader := Reader.new(PackedByteArray([0x80]))

	# When: Reading a varint that only has a continuation bit.
	var start := reader.get_position()
	var result := reader.read_varint_unsigned()

	# Then: There is an EOF error.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)

	# Then: No data is returned.
	assert_eq(result, 0)

	# Then: The cursor position is reset.
	assert_eq(reader.get_position(), start)


func test_reader_varint_invalid_data_restores_position() -> void:
	# Given: A reader with varint that never terminates.
	var data := PackedByteArray()
	data.resize(10)
	data.fill(0xFF)
	var reader := Reader.new(data)

	# When: Reading invalid varint.
	var start := reader.get_position()
	var result := reader.read_varint_unsigned()

	# Then: There is an invalid data error.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_INVALID_DATA)

	# Then: No data is returned.
	assert_eq(result, 0)

	# Then: The cursor position is reset.
	assert_eq(reader.get_position(), start)


func test_reader_f64_eof_restores_position() -> void:
	# Given: A reader with only 4 bytes (f64 needs 8).
	var data := PackedByteArray()
	data.resize(4)
	data.fill(0x00)
	var reader := Reader.new(data)

	# When: Reading f64 that encounters EOF.
	var start := reader.get_position()
	var result := reader.read_f64()

	# Then: There is an EOF error.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)

	# Then: No data is returned.
	assert_eq(result, 0)

	# Then: The cursor position is reset.
	assert_eq(reader.get_position(), start)


func test_reader_string_eof_on_varint_restores_position() -> void:
	# Given: A reader with incomplete varint for string length.
	var reader := Reader.new(PackedByteArray([0x80]))

	# When: Reading string that encounters EOF in varint.
	var start := reader.get_position()
	var result := reader.read_string()

	# Then: There is an EOF error.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)

	# Then: No data is returned.
	assert_eq(result, "")

	# Then: The cursor position is reset.
	assert_eq(reader.get_position(), start)


func test_reader_string_eof_on_data_preserves_position() -> void:
	# Given: A reader with valid varint but insufficient string data.
	var reader := Reader.new(PackedByteArray([0x0A]))  # Length = 10, but no data

	# When: Reading string that encounters EOF in data.
	var start := reader.get_position()
	var result := reader.read_string()

	# Then: There is an EOF error.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)

	# Then: No data is returned.
	assert_eq(result, "")

	# Then: The cursor position is reset.
	assert_eq(reader.get_position(), start)


# -- TEST HOOKS ---------------------------------------------------------------------- #


func before_all() -> void:
	# NOTE: Hide unactionable errors when using object doubles.
	ProjectSettings.set("debug/gdscript/warnings/native_method_override", false)
