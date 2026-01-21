##
## bitstream_test.gd
##
## Test suite for the `BitStream` base class.
##

extends GutTest

# -- DEPENDENCIES -------------------------------------------------------------------- #

const BitStream := preload("./bitstream.gd")

# -- TEST METHODS -------------------------------------------------------------------- #


func test_bitstream_clear_resets_data() -> void:
	# Given: A bitstream with test data.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34, 0x56]))

	# Given: The position is advanced.
	bs.seek(8)

	# Given: An error is induced for the stream.
	bs.seek(1000)

	# When: The bitstream is reset.
	bs.clear()

	# Then: There is no underlying data or error.
	assert_true(bs.is_valid())
	assert_eq(bs.get_capacity(), 0)
	assert_eq(bs.get_position(), 0)


func test_bitstream_initial_state() -> void:
	# Given: A new empty bitstream.
	var bs := BitStream.new()

	# Then: Initial state is correct.
	assert_eq(bs.get_position(), 0)
	assert_eq(bs.get_capacity(), 0)
	assert_eq(bs.length(), 0)
	assert_true(bs.is_valid())
	assert_eq(bs.get_error(), OK)


func test_bitstream_with_data_initial_state() -> void:
	# Given: A new bitstream with data.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34, 0x56]))

	# Then: Initial state reflects the data.
	assert_eq(bs.get_position(), 0)
	assert_eq(bs.get_capacity(), 24)
	assert_eq(bs.length(), 0)
	assert_true(bs.is_valid())


func test_bitstream_error_persistence() -> void:
	# Given: A bitstream with some data.
	var bs := BitStream.new(PackedByteArray([0xFF]))

	# Given: An error is triggered.
	bs.seek(16)

	# Then: The error is set.
	assert_false(bs.is_valid())
	assert_eq(bs.get_error(), ERR_INVALID_PARAMETER)

	# When: Another seek is attempted that would be valid.
	bs.seek(4)

	# Then: The error persisted.
	assert_false(bs.is_valid())
	assert_eq(bs.get_error(), ERR_INVALID_PARAMETER)


func test_bitstream_seek_invalid() -> void:
	# Given: A bitstream with some data.
	var bs := BitStream.new(PackedByteArray([0xFF]))

	# When: Seeking past the end.
	bs.seek(100)

	# Then: Error is set.
	assert_false(bs.is_valid())
	assert_eq(bs.get_error(), ERR_INVALID_PARAMETER)


func test_bitstream_seek_negative() -> void:
	# Given: A bitstream with some data.
	var bs := BitStream.new(PackedByteArray([0xFF]))

	# When: Seeking to negative position.
	bs.seek(-1)

	# Then: Error is set.
	assert_false(bs.is_valid())
	assert_eq(bs.get_error(), ERR_INVALID_PARAMETER)


func test_bitstream_seek() -> void:
	# Given: A bitstream with test data.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34, 0x56]))

	# When: Seeking to bit position 8.
	bs.seek(8)

	# Then: Position is updated correctly.
	assert_eq(bs.get_position(), 8)
	assert_true(bs.is_valid())


func test_bitstream_seek_to_start() -> void:
	# Given: A bitstream with some position advanced.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34]))
	bs.seek(16)

	# When: Seeking back to start.
	bs.seek(0)

	# Then: Position is at the beginning.
	assert_eq(bs.get_position(), 0)
	assert_true(bs.is_valid())


func test_bitstream_seek_to_end() -> void:
	# Given: A bitstream with test data.
	var bs := BitStream.new(PackedByteArray([0xFF]))

	# When: Seeking to the exact end position.
	bs.seek(8)

	# Then: Position is at end and valid.
	assert_eq(bs.get_position(), 8)
	assert_true(bs.is_valid())
	assert_true(bs.is_at_end())


func test_bitstream_get_capacity() -> void:
	# Given: A bitstream with 3 bytes.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34, 0x56]))

	# Then: Capacity is 24 bits.
	assert_eq(bs.get_capacity(), 24)


func test_bitstream_length_empty() -> void:
	# Given: A new empty bitstream.
	var bs := BitStream.new()

	# Then: Length is 0.
	assert_eq(bs.length(), 0)


func test_bitstream_length_with_position() -> void:
	# Given: A bitstream with data.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34, 0x56]))

	# When: Position is advanced to 3 bits.
	bs.seek(3)

	# Then: Length is 1 byte (rounded up).
	assert_eq(bs.length(), 1)

	# When: Position is advanced to 9 bits.
	bs.seek(9)

	# Then: Length is 2 bytes (rounded up).
	assert_eq(bs.length(), 2)


func test_bitstream_to_bytes_empty() -> void:
	# Given: An empty bitstream.
	var bs := BitStream.new()

	# When: Converting to bytes.
	var bytes := bs.to_bytes()

	# Then: Result is empty.
	assert_eq(bytes.size(), 0)


func test_bitstream_to_bytes_full() -> void:
	# Given: A bitstream with data at max position.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34, 0x56]))
	bs.seek(24)

	# When: Converting to bytes.
	var bytes := bs.to_bytes()

	# Then: All bytes are returned.
	assert_eq(bytes.size(), 3)
	assert_eq(bytes, PackedByteArray([0x12, 0x34, 0x56]))


func test_bitstream_to_bytes_partial() -> void:
	# Given: A bitstream with data and partial position.
	var bs := BitStream.new(PackedByteArray([0x12, 0x34, 0x56]))
	bs.seek(10)

	# When: Converting to bytes.
	var bytes := bs.to_bytes()

	# Then: Only bytes up to position are returned (rounded up).
	assert_eq(bytes.size(), 2)
	assert_eq(bytes, PackedByteArray([0x12, 0x34]))


func test_bitstream_is_at_end() -> void:
	# Given: A bitstream with some data.
	var bs := BitStream.new(PackedByteArray([0xFF]))

	# Then: The stream initially is not at the end.
	assert_false(bs.is_at_end())

	# When: The cursor seeks to the end.
	bs.seek(8)

	# Then: The stream correctly detects it's at the end.
	assert_true(bs.is_at_end())


func test_bitstream_is_at_end_beyond() -> void:
	# Given: A bitstream with some data.
	var bs := BitStream.new(PackedByteArray([0xFF]))

	# When: Seeking beyond the end (which causes an error).
	bs.seek(16)

	# Then: The bitstream has an error.
	assert_eq(bs.get_error(), ERR_INVALID_PARAMETER)

	# Then: The stream is still at the original position.
	assert_eq(bs.get_position(), 0)


# -- TEST HOOKS ---------------------------------------------------------------------- #


func before_all() -> void:
	# NOTE: Hide unactionable errors when using object doubles.
	ProjectSettings.set("debug/gdscript/warnings/native_method_override", false)
