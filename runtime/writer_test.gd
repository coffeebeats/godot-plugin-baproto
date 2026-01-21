##
## writer_test.gd
##
## Test suite for `Writer` class. Tests writing operations and roundtrip serialization.
##

extends GutTest

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Reader := preload("res://runtime/reader.gd")
const Writer := preload("res://runtime/writer.gd")

# -- TEST METHODS -------------------------------------------------------------------- #


func test_writer_read_bool_true() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing a boolean valid.
	writer.write_bool(true)
	var data := writer.to_bytes()

	# Then: Reading returns that written value.
	var reader := Reader.new(data)
	assert_true(reader.read_bool())
	assert_true(reader.is_valid())


func test_writer_read_bool_false() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing false.
	writer.write_bool(false)
	var data := writer.to_bytes()

	# Then: Reading returns that written value.
	var reader := Reader.new(data)
	assert_false(reader.read_bool())
	assert_true(reader.is_valid())


func test_writer_read_multiple_bools() -> void:
	# Given: A writer with multiple bools.
	var writer := Writer.new()
	var pattern := [true, false, true, true, false]

	for val in pattern:
		writer.write_bool(val)

	var data := writer.to_bytes()

	# When: Reading all bools.
	var reader := Reader.new(data)

	# Then: All values match.
	for i in range(pattern.size()):
		assert_eq(reader.read_bool(), pattern[i], "Bool %d mismatch" % i)

	assert_true(reader.is_valid())


func test_writer_read_bits_single_byte() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing 5 bits with value 0b11010 (26).
	writer.write_bits(26, 5)
	var data := writer.to_bytes()

	# Then: Reading returns the same value.
	var reader := Reader.new(data)
	assert_eq(reader.read_bits(5), 26)
	assert_true(reader.is_valid())


func test_writer_read_bits_cross_byte_boundary() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing bits that cross byte boundaries.
	writer.write_bits(0b111, 3) # First 3 bits
	writer.write_bits(0x1FF, 9) # Next 9 bits cross byte boundary
	var data := writer.to_bytes()

	# Then: Reading returns correct values.
	var reader := Reader.new(data)
	assert_eq(reader.read_bits(3), 0b111)
	assert_eq(reader.read_bits(9), 0x1FF)
	assert_true(reader.is_valid())


func test_writer_read_bits_various_widths() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing values of various bit widths.
	writer.write_bits(1, 1)
	writer.write_bits(3, 2)
	writer.write_bits(15, 4)
	writer.write_bits(255, 8)
	writer.write_bits(65535, 16)
	writer.write_bits(0xFFFFFFFF, 32)
	var data := writer.to_bytes()

	# Then: All values read correctly.
	var reader := Reader.new(data)
	assert_eq(reader.read_bits(1), 1)
	assert_eq(reader.read_bits(2), 3)
	assert_eq(reader.read_bits(4), 15)
	assert_eq(reader.read_bits(8), 255)
	assert_eq(reader.read_bits(16), 65535)
	assert_eq(reader.read_bits(32), 0xFFFFFFFF)
	assert_true(reader.is_valid())


func test_writer_read_bits_64() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing a 64-bit value.
	var val := 0x123456789ABCDEF0
	writer.write_bits(val, 64)
	var data := writer.to_bytes()

	# Then: Reading returns the same value.
	var reader := Reader.new(data)
	assert_eq(reader.read_bits(64), val)
	assert_true(reader.is_valid())


func test_writer_read_u8() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing u8 values.
	writer.write_u8(0)
	writer.write_u8(127)
	writer.write_u8(255)
	var data := writer.to_bytes()

	# Then: Reading returns correct values.
	var reader := Reader.new(data)
	assert_eq(reader.read_u8(), 0)
	assert_eq(reader.read_u8(), 127)
	assert_eq(reader.read_u8(), 255)
	assert_true(reader.is_valid())


func test_writer_read_u16() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing u16 values.
	writer.write_u16(0)
	writer.write_u16(32767)
	writer.write_u16(65535)
	var data := writer.to_bytes()

	# Then: Reading returns correct values.
	var reader := Reader.new(data)
	assert_eq(reader.read_u16(), 0)
	assert_eq(reader.read_u16(), 32767)
	assert_eq(reader.read_u16(), 65535)
	assert_true(reader.is_valid())


func test_writer_read_u32() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing u32 values.
	writer.write_u32(0)
	writer.write_u32(2147483647)
	writer.write_u32(0xFFFFFFFF)
	var data := writer.to_bytes()

	# Then: Reading returns correct values.
	var reader := Reader.new(data)
	assert_eq(reader.read_u32(), 0)
	assert_eq(reader.read_u32(), 2147483647)
	assert_eq(reader.read_u32(), 0xFFFFFFFF)
	assert_true(reader.is_valid())


func test_writer_read_i8() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing i8 values.
	writer.write_i8(0)
	writer.write_i8(127)
	writer.write_i8(-128)
	writer.write_i8(-1)
	var data := writer.to_bytes()

	# Then: Reading returns correct signed values.
	var reader := Reader.new(data)
	assert_eq(reader.read_i8(), 0)
	assert_eq(reader.read_i8(), 127)
	assert_eq(reader.read_i8(), -128)
	assert_eq(reader.read_i8(), -1)
	assert_true(reader.is_valid())


func test_writer_read_i16() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing i16 values.
	writer.write_i16(0)
	writer.write_i16(32767)
	writer.write_i16(-32768)
	writer.write_i16(-1)
	var data := writer.to_bytes()

	# Then: Reading returns correct signed values.
	var reader := Reader.new(data)
	assert_eq(reader.read_i16(), 0)
	assert_eq(reader.read_i16(), 32767)
	assert_eq(reader.read_i16(), -32768)
	assert_eq(reader.read_i16(), -1)
	assert_true(reader.is_valid())


func test_writer_read_i32() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing i32 values.
	writer.write_i32(0)
	writer.write_i32(2147483647)
	writer.write_i32(-2147483648)
	writer.write_i32(-1)
	var data := writer.to_bytes()

	# Then: Reading returns correct signed values.
	var reader := Reader.new(data)
	assert_eq(reader.read_i32(), 0)
	assert_eq(reader.read_i32(), 2147483647)
	assert_eq(reader.read_i32(), -2147483648)
	assert_eq(reader.read_i32(), -1)
	assert_true(reader.is_valid())


func test_writer_read_i64() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing i64 edge values.
	writer.write_i64(0)
	writer.write_i64(9223372036854775807) # Max i64
	writer.write_i64(-9223372036854775808) # Min i64
	writer.write_i64(-1)
	var data := writer.to_bytes()

	# Then: Reading returns correct signed values.
	var reader := Reader.new(data)
	assert_eq(reader.read_i64(), 0)
	assert_eq(reader.read_i64(), 9223372036854775807)
	assert_eq(reader.read_i64(), -9223372036854775808)
	assert_eq(reader.read_i64(), -1)
	assert_true(reader.is_valid())


func test_writer_read_f32() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing f32 values.
	writer.write_f32(0.0)
	writer.write_f32(1.0)
	writer.write_f32(-1.0)
	writer.write_f32(3.14159)
	var data := writer.to_bytes()

	# Then: Reading returns approximately equal values (f32 precision loss).
	var reader := Reader.new(data)
	assert_almost_eq(reader.read_f32(), 0.0, 0.0001)
	assert_almost_eq(reader.read_f32(), 1.0, 0.0001)
	assert_almost_eq(reader.read_f32(), -1.0, 0.0001)
	assert_almost_eq(reader.read_f32(), 3.14159, 0.0001)
	assert_true(reader.is_valid())


func test_writer_read_f64() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing f64 values.
	writer.write_f64(0.0)
	writer.write_f64(1.0)
	writer.write_f64(-1.0)
	writer.write_f64(3.141592653589793)
	var data := writer.to_bytes()

	# Then: Reading returns the exact values.
	var reader := Reader.new(data)
	assert_eq(reader.read_f64(), 0.0)
	assert_eq(reader.read_f64(), 1.0)
	assert_eq(reader.read_f64(), -1.0)
	assert_eq(reader.read_f64(), 3.141592653589793)
	assert_true(reader.is_valid())


func test_writer_read_f32_special_values() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing special float values.
	writer.write_f32(INF)
	writer.write_f32(-INF)
	writer.write_f32(NAN)
	var data := writer.to_bytes()

	# Then: Reading returns the special values.
	var reader := Reader.new(data)
	assert_eq(reader.read_f32(), INF)
	assert_eq(reader.read_f32(), -INF)
	assert_true(is_nan(reader.read_f32()))
	assert_true(reader.is_valid())


func test_writer_read_f64_special_values() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing special float values.
	writer.write_f64(INF)
	writer.write_f64(-INF)
	writer.write_f64(NAN)
	var data := writer.to_bytes()

	# Then: Reading returns the special values.
	var reader := Reader.new(data)
	assert_eq(reader.read_f64(), INF)
	assert_eq(reader.read_f64(), -INF)
	assert_true(is_nan(reader.read_f64()))
	assert_true(reader.is_valid())


func test_writer_read_f32_negative_zero() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing negative zero.
	writer.write_f32(-0.0)
	var data := writer.to_bytes()

	# Then: Reading returns zero (sign may not be preserved in f32).
	var reader := Reader.new(data)
	var val := reader.read_f32()
	assert_eq(val, 0.0)
	assert_true(reader.is_valid())


func test_writer_read_f64_negative_zero() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing negative zero.
	writer.write_f64(-0.0)
	var data := writer.to_bytes()

	# Then: Reading returns zero.
	var reader := Reader.new(data)
	var val := reader.read_f64()
	assert_eq(val, 0.0)
	assert_true(reader.is_valid())


func test_writer_read_varint_unsigned_single_byte(
	params = use_parameters([[0], [1], [127]])
) -> void:
	# Given: A writer.
	var writer := Writer.new()
	var value: int = params[0]

	# When: Writing a small unsigned value (fits in single byte).
	writer.write_varint_unsigned(value)
	var data := writer.to_bytes()

	# Then: Reading returns the correct value.
	var reader := Reader.new(data)
	assert_eq(reader.read_varint_unsigned(), value)
	assert_true(reader.is_valid())


func test_writer_read_varint_unsigned_multi_byte() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing values requiring multiple bytes.
	writer.write_varint_unsigned(128)
	writer.write_varint_unsigned(16383)
	writer.write_varint_unsigned(16384)
	writer.write_varint_unsigned(2097151)
	var data := writer.to_bytes()

	# Then: Reading returns correct values.
	var reader := Reader.new(data)
	assert_eq(reader.read_varint_unsigned(), 128)
	assert_eq(reader.read_varint_unsigned(), 16383)
	assert_eq(reader.read_varint_unsigned(), 16384)
	assert_eq(reader.read_varint_unsigned(), 2097151)
	assert_true(reader.is_valid())


func test_writer_read_varint_unsigned_large() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing a large value.
	var large_val := 0x7FFFFFFFFFFFFFFF # Max positive int64
	writer.write_varint_unsigned(large_val)
	var data := writer.to_bytes()

	# Then: Reading returns the same value.
	var reader := Reader.new(data)
	assert_eq(reader.read_varint_unsigned(), large_val)
	assert_true(reader.is_valid())


func test_writer_read_varint_signed_positive() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing positive signed values.
	writer.write_varint_signed(0)
	writer.write_varint_signed(1)
	writer.write_varint_signed(63)
	writer.write_varint_signed(64)
	var data := writer.to_bytes()

	# Then: Reading returns correct values.
	var reader := Reader.new(data)
	assert_eq(reader.read_varint_signed(), 0)
	assert_eq(reader.read_varint_signed(), 1)
	assert_eq(reader.read_varint_signed(), 63)
	assert_eq(reader.read_varint_signed(), 64)
	assert_true(reader.is_valid())


func test_writer_read_varint_signed_negative() -> void:
	# Given: A writer.
	var writer := Writer.new()

	# When: Writing negative signed values.
	writer.write_varint_signed(-1)
	writer.write_varint_signed(-64)
	writer.write_varint_signed(-65)
	writer.write_varint_signed(-1000000)
	var data := writer.to_bytes()

	# Then: Reading returns correct values.
	var reader := Reader.new(data)
	assert_eq(reader.read_varint_signed(), -1)
	assert_eq(reader.read_varint_signed(), -64)
	assert_eq(reader.read_varint_signed(), -65)
	assert_eq(reader.read_varint_signed(), -1000000)
	assert_true(reader.is_valid())


func test_writer_varint_alignment() -> void:
	# Given: A writer with bits before a varint.
	var writer := Writer.new()
	writer.write_bits(0b101, 3) # 3 bits, not byte-aligned
	writer.write_varint_unsigned(300)
	var data := writer.to_bytes()

	# When: Reading with the same pattern.
	var reader := Reader.new(data)
	var bits := reader.read_bits(3)
	var varint := reader.read_varint_unsigned()

	# Then: Values are correct.
	assert_eq(bits, 0b101)
	assert_eq(varint, 300)
	assert_true(reader.is_valid())

	# Then: Verify no padding was added (3 bits + varint bytes).
	assert_eq(reader.get_position(), 19) # 3 + 2 * 8 = 19


func test_writer_read_bytes() -> void:
	# Given: A writer and some bytes.
	var writer := Writer.new()
	var original := PackedByteArray([0x00, 0x11, 0x22, 0x33, 0xFF])

	# When: Writing and reading bytes.
	writer.write_bytes(original)
	var data := writer.to_bytes()
	var reader := Reader.new(data)
	var result := reader.read_bytes(5)

	# Then: The bytes match.
	assert_eq(result, original)
	assert_true(reader.is_valid())


func test_writer_read_bytes_empty() -> void:
	# Given: A writer and empty bytes.
	var writer := Writer.new()
	var original := PackedByteArray()

	# When: Writing and reading empty bytes.
	writer.write_bytes(original)
	var data := writer.to_bytes()
	var reader := Reader.new(data)
	var result := reader.read_bytes(0)

	# Then: The result is empty.
	assert_eq(result.size(), 0)
	assert_true(reader.is_valid())


func test_writer_read_string() -> void:
	# Given: A writer and a string.
	var writer := Writer.new()
	var original := "Hello, World!"

	# When: Writing and reading the string.
	writer.write_string(original)
	var data := writer.to_bytes()
	var reader := Reader.new(data)
	var result := reader.read_string()

	# Then: The strings match.
	assert_eq(result, original)
	assert_true(reader.is_valid())


func test_writer_read_string_empty() -> void:
	# Given: A writer and an empty string.
	var writer := Writer.new()
	var original := ""

	# When: Writing and reading the empty string.
	writer.write_string(original)
	var data := writer.to_bytes()
	var reader := Reader.new(data)
	var result := reader.read_string()

	# Then: The result is empty.
	assert_eq(result, "")
	assert_true(reader.is_valid())


func test_writer_read_string_unicode() -> void:
	# Given: A writer and a Unicode string.
	var writer := Writer.new()
	var original := "Hello, \u4e16\u754c! \U0001F600" # "Hello, 世界! 😀"

	# When: Writing and reading the Unicode string.
	writer.write_string(original)
	var data := writer.to_bytes()
	var reader := Reader.new(data)
	var result := reader.read_string()

	# Then: The strings match.
	assert_eq(result, original)
	assert_true(reader.is_valid())


func test_writer_bytes_alignment() -> void:
	# Given: A writer with bits before bytes.
	var writer := Writer.new()
	writer.write_bits(0b1111, 4) # 4 bits, not byte-aligned
	writer.write_bytes(PackedByteArray([0xAB, 0xCD]))
	var data := writer.to_bytes()

	# When: Reading with the same pattern.
	var reader := Reader.new(data)
	var bits := reader.read_bits(4)
	var bytes := reader.read_bytes(2)

	# Then: Values are correct.
	assert_eq(bits, 0b1111)
	assert_eq(bytes, PackedByteArray([0xAB, 0xCD]))
	assert_true(reader.is_valid())

	# Then: Verify no padding was added.
	assert_eq(reader.get_position(), 20) # 4 + 2 * 8 = 20


func test_writer_roundtrip_mixed_types() -> void:
	# Given: A writer with various types.
	var writer := Writer.new()
	writer.write_bool(true)
	writer.write_bits(42, 7)
	writer.write_zigzag(-100, 16)
	writer.write_u32(0xCAFEBABE)
	writer.write_i16(-1234)
	writer.write_f32(2.5)
	writer.write_varint_signed(-9999)
	writer.write_string("test")
	var data := writer.to_bytes()

	# When: Reading all values.
	var reader := Reader.new(data)

	# Then: All values match.
	assert_true(reader.read_bool())
	assert_eq(reader.read_bits(7), 42)
	assert_eq(reader.read_zigzag(16), -100)
	assert_eq(reader.read_u32(), 0xCAFEBABE)
	assert_eq(reader.read_i16(), -1234)
	assert_almost_eq(reader.read_f32(), 2.5, 0.0001)
	assert_eq(reader.read_varint_signed(), -9999)
	assert_eq(reader.read_string(), "test")
	assert_true(reader.is_valid())


func test_writer_roundtrip_all_integer_types() -> void:
	# Given: A writer with all integer types.
	var writer := Writer.new()
	writer.write_u8(200)
	writer.write_u16(50000)
	writer.write_u32(3000000000)
	writer.write_i8(-100)
	writer.write_i16(-30000)
	writer.write_i32(-2000000000)
	writer.write_i64(-9000000000000000000)
	var data := writer.to_bytes()

	# When: Reading all values.
	var reader := Reader.new(data)

	# Then: All values match.
	assert_eq(reader.read_u8(), 200)
	assert_eq(reader.read_u16(), 50000)
	assert_eq(reader.read_u32(), 3000000000)
	assert_eq(reader.read_i8(), -100)
	assert_eq(reader.read_i16(), -30000)
	assert_eq(reader.read_i32(), -2000000000)
	assert_eq(reader.read_i64(), -9000000000000000000)
	assert_true(reader.is_valid())


func test_writer_roundtrip_nested_messages_pattern() -> void:
	# Given: A pattern simulating nested message encoding.
	var writer := Writer.new()

	# Outer message header.
	writer.write_varint_unsigned(2) # Message type

	# Inner message 1.
	writer.write_bool(true)
	writer.write_bits(255, 8)

	# Inner message 2.
	writer.write_bool(false)
	writer.write_bits(128, 8)

	var data := writer.to_bytes()

	# When: Reading the nested structure.
	var reader := Reader.new(data)

	# Then: Structure is preserved.
	assert_eq(reader.read_varint_unsigned(), 2)
	assert_true(reader.read_bool())
	assert_eq(reader.read_bits(8), 255)
	assert_false(reader.read_bool())
	assert_eq(reader.read_bits(8), 128)
	assert_true(reader.is_valid())


# -- TEST HOOKS ---------------------------------------------------------------------- #


func before_all() -> void:
	# NOTE: Hide unactionable errors when using object doubles.
	ProjectSettings.set("debug/gdscript/warnings/native_method_override", false)
