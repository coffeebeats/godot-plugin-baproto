extends "res://addons/gut/test.gd"

const Bitstream := preload("res://runtime.gd")

# ---------------------------------------------------------------------------- #
#                           Tests: ZigZag Encoding                             #
# ---------------------------------------------------------------------------- #


func test_zigzag_encode_zero():
	# Given: A zero value.
	# When: ZigZag encoding is applied.
	var encoded := Bitstream.zigzag_encode(0)
	# Then: The result is 0.
	assert_eq(encoded, 0)


func test_zigzag_encode_positive():
	# Given: Positive integers.
	# When: ZigZag encoding is applied.
	# Then: Positive values map to even numbers (2n).
	assert_eq(Bitstream.zigzag_encode(1), 2)
	assert_eq(Bitstream.zigzag_encode(2), 4)
	assert_eq(Bitstream.zigzag_encode(100), 200)


func test_zigzag_encode_negative():
	# Given: Negative integers.
	# When: ZigZag encoding is applied.
	# Then: Negative values map to odd numbers (2|n|-1).
	assert_eq(Bitstream.zigzag_encode(-1), 1)
	assert_eq(Bitstream.zigzag_encode(-2), 3)
	assert_eq(Bitstream.zigzag_encode(-100), 199)


func test_zigzag_roundtrip():
	# Given: Various signed integers.
	var values := [0, 1, -1, 127, -128, 32767, -32768, 2147483647, -2147483648]
	for val in values:
		# When: Encoding then decoding.
		var encoded := Bitstream.zigzag_encode(val)
		var decoded := Bitstream.zigzag_decode(encoded)
		# Then: The original value is recovered.
		assert_eq(decoded, val, "Roundtrip failed for %d" % val)


# ---------------------------------------------------------------------------- #
#                              Tests: Bool/Bits                                #
# ---------------------------------------------------------------------------- #


func test_write_read_bool_true():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing true.
	writer.write_bool(true)
	var data := writer.to_bytes()
	# Then: Reading returns true.
	var reader := Bitstream.Reader.new(data)
	assert_true(reader.read_bool())
	assert_true(reader.is_valid())


func test_write_read_bool_false():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing false.
	writer.write_bool(false)
	var data := writer.to_bytes()
	# Then: Reading returns false.
	var reader := Bitstream.Reader.new(data)
	assert_false(reader.read_bool())
	assert_true(reader.is_valid())


func test_write_read_multiple_bools():
	# Given: A writer with multiple bools.
	var writer := Bitstream.Writer.new()
	var pattern := [true, false, true, true, false, false, true, false, true]
	for val in pattern:
		writer.write_bool(val)
	var data := writer.to_bytes()
	# When: Reading all bools.
	var reader := Bitstream.Reader.new(data)
	# Then: All values match.
	for i in range(pattern.size()):
		assert_eq(reader.read_bool(), pattern[i], "Bool %d mismatch" % i)
	assert_true(reader.is_valid())


func test_write_read_bits_single_byte():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing 5 bits with value 0b11010 (26).
	writer.write_bits(26, 5)
	var data := writer.to_bytes()
	# Then: Reading returns the same value.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_bits(5), 26)
	assert_true(reader.is_valid())


func test_write_read_bits_cross_byte_boundary():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing bits that cross byte boundaries.
	writer.write_bits(0b111, 3) # First 3 bits
	writer.write_bits(0x1FF, 9) # Next 9 bits cross byte boundary
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_bits(3), 0b111)
	assert_eq(reader.read_bits(9), 0x1FF)
	assert_true(reader.is_valid())


func test_write_read_bits_various_widths():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing values of various bit widths.
	writer.write_bits(1, 1)
	writer.write_bits(3, 2)
	writer.write_bits(15, 4)
	writer.write_bits(255, 8)
	writer.write_bits(65535, 16)
	writer.write_bits(0xFFFFFFFF, 32)
	var data := writer.to_bytes()
	# Then: All values read correctly.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_bits(1), 1)
	assert_eq(reader.read_bits(2), 3)
	assert_eq(reader.read_bits(4), 15)
	assert_eq(reader.read_bits(8), 255)
	assert_eq(reader.read_bits(16), 65535)
	assert_eq(reader.read_bits(32), 0xFFFFFFFF)
	assert_true(reader.is_valid())


func test_write_read_bits_64():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing a 64-bit value.
	var val := 0x123456789ABCDEF0
	writer.write_bits(val, 64)
	var data := writer.to_bytes()
	# Then: Reading returns the same value.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_bits(64), val)
	assert_true(reader.is_valid())


# ---------------------------------------------------------------------------- #
#                              Tests: ZigZag Bits                              #
# ---------------------------------------------------------------------------- #


func test_write_read_zigzag_positive():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing a positive ZigZag value in 8 bits.
	writer.write_zigzag(42, 8)
	var data := writer.to_bytes()
	# Then: Reading returns the original value.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_zigzag(8), 42)
	assert_true(reader.is_valid())


func test_write_read_zigzag_negative():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing a negative ZigZag value in 8 bits.
	writer.write_zigzag(-42, 8)
	var data := writer.to_bytes()
	# Then: Reading returns the original value.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_zigzag(8), -42)
	assert_true(reader.is_valid())


func test_write_read_zigzag_edge_cases():
	# Given: A writer with edge case values.
	var writer := Bitstream.Writer.new()
	writer.write_zigzag(0, 8)
	writer.write_zigzag(-1, 8)
	writer.write_zigzag(63, 8)
	writer.write_zigzag(-64, 8)
	var data := writer.to_bytes()
	# When: Reading all values.
	var reader := Bitstream.Reader.new(data)
	# Then: All values match.
	assert_eq(reader.read_zigzag(8), 0)
	assert_eq(reader.read_zigzag(8), -1)
	assert_eq(reader.read_zigzag(8), 63)
	assert_eq(reader.read_zigzag(8), -64)
	assert_true(reader.is_valid())


# ---------------------------------------------------------------------------- #
#                         Tests: Fixed-width Integers                          #
# ---------------------------------------------------------------------------- #


func test_write_read_u8():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing u8 values.
	writer.write_u8(0)
	writer.write_u8(127)
	writer.write_u8(255)
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_u8(), 0)
	assert_eq(reader.read_u8(), 127)
	assert_eq(reader.read_u8(), 255)
	assert_true(reader.is_valid())


func test_write_read_u16():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing u16 values.
	writer.write_u16(0)
	writer.write_u16(32767)
	writer.write_u16(65535)
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_u16(), 0)
	assert_eq(reader.read_u16(), 32767)
	assert_eq(reader.read_u16(), 65535)
	assert_true(reader.is_valid())


func test_write_read_u32():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing u32 values.
	writer.write_u32(0)
	writer.write_u32(2147483647)
	writer.write_u32(0xFFFFFFFF)
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_u32(), 0)
	assert_eq(reader.read_u32(), 2147483647)
	assert_eq(reader.read_u32(), 0xFFFFFFFF)
	assert_true(reader.is_valid())


func test_write_read_i8():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing i8 values.
	writer.write_i8(0)
	writer.write_i8(127)
	writer.write_i8(-128)
	writer.write_i8(-1)
	var data := writer.to_bytes()
	# Then: Reading returns correct signed values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_i8(), 0)
	assert_eq(reader.read_i8(), 127)
	assert_eq(reader.read_i8(), -128)
	assert_eq(reader.read_i8(), -1)
	assert_true(reader.is_valid())


func test_write_read_i16():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing i16 values.
	writer.write_i16(0)
	writer.write_i16(32767)
	writer.write_i16(-32768)
	writer.write_i16(-1)
	var data := writer.to_bytes()
	# Then: Reading returns correct signed values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_i16(), 0)
	assert_eq(reader.read_i16(), 32767)
	assert_eq(reader.read_i16(), -32768)
	assert_eq(reader.read_i16(), -1)
	assert_true(reader.is_valid())


func test_write_read_i32():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing i32 values.
	writer.write_i32(0)
	writer.write_i32(2147483647)
	writer.write_i32(-2147483648)
	writer.write_i32(-1)
	var data := writer.to_bytes()
	# Then: Reading returns correct signed values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_i32(), 0)
	assert_eq(reader.read_i32(), 2147483647)
	assert_eq(reader.read_i32(), -2147483648)
	assert_eq(reader.read_i32(), -1)
	assert_true(reader.is_valid())


func test_write_read_i64():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing i64 edge values.
	writer.write_i64(0)
	writer.write_i64(9223372036854775807) # Max i64
	writer.write_i64(-9223372036854775808) # Min i64
	writer.write_i64(-1)
	var data := writer.to_bytes()
	# Then: Reading returns correct signed values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_i64(), 0)
	assert_eq(reader.read_i64(), 9223372036854775807)
	assert_eq(reader.read_i64(), -9223372036854775808)
	assert_eq(reader.read_i64(), -1)
	assert_true(reader.is_valid())


# ---------------------------------------------------------------------------- #
#                              Tests: IEEE Floats                              #
# ---------------------------------------------------------------------------- #


func test_write_read_f32():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing f32 values.
	writer.write_f32(0.0)
	writer.write_f32(1.0)
	writer.write_f32(-1.0)
	writer.write_f32(3.14159)
	var data := writer.to_bytes()
	# Then: Reading returns approximately equal values (f32 precision loss).
	var reader := Bitstream.Reader.new(data)
	assert_almost_eq(reader.read_f32(), 0.0, 0.0001)
	assert_almost_eq(reader.read_f32(), 1.0, 0.0001)
	assert_almost_eq(reader.read_f32(), -1.0, 0.0001)
	assert_almost_eq(reader.read_f32(), 3.14159, 0.0001)
	assert_true(reader.is_valid())


func test_write_read_f64():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing f64 values.
	writer.write_f64(0.0)
	writer.write_f64(1.0)
	writer.write_f64(-1.0)
	writer.write_f64(3.141592653589793)
	var data := writer.to_bytes()
	# Then: Reading returns the exact values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_f64(), 0.0)
	assert_eq(reader.read_f64(), 1.0)
	assert_eq(reader.read_f64(), -1.0)
	assert_eq(reader.read_f64(), 3.141592653589793)
	assert_true(reader.is_valid())


func test_write_read_f32_special_values():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing special float values.
	writer.write_f32(INF)
	writer.write_f32(-INF)
	writer.write_f32(NAN)
	var data := writer.to_bytes()
	# Then: Reading returns the special values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_f32(), INF)
	assert_eq(reader.read_f32(), -INF)
	assert_true(is_nan(reader.read_f32()))
	assert_true(reader.is_valid())


func test_write_read_f64_special_values():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing special float values.
	writer.write_f64(INF)
	writer.write_f64(-INF)
	writer.write_f64(NAN)
	var data := writer.to_bytes()
	# Then: Reading returns the special values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_f64(), INF)
	assert_eq(reader.read_f64(), -INF)
	assert_true(is_nan(reader.read_f64()))
	assert_true(reader.is_valid())


func test_write_read_f32_negative_zero():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing negative zero.
	writer.write_f32(-0.0)
	var data := writer.to_bytes()
	# Then: Reading returns zero (sign may not be preserved in f32).
	var reader := Bitstream.Reader.new(data)
	var val := reader.read_f32()
	assert_eq(val, 0.0)
	assert_true(reader.is_valid())


func test_write_read_f64_negative_zero():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing negative zero.
	writer.write_f64(-0.0)
	var data := writer.to_bytes()
	# Then: Reading returns zero.
	var reader := Bitstream.Reader.new(data)
	var val := reader.read_f64()
	assert_eq(val, 0.0)
	assert_true(reader.is_valid())


# ---------------------------------------------------------------------------- #
#                               Tests: Varints                                 #
# ---------------------------------------------------------------------------- #


func test_write_read_varint_unsigned_single_byte():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing small unsigned values (fit in single byte).
	writer.write_varint_unsigned(0)
	writer.write_varint_unsigned(1)
	writer.write_varint_unsigned(127)
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_varint_unsigned(), 0)
	assert_eq(reader.read_varint_unsigned(), 1)
	assert_eq(reader.read_varint_unsigned(), 127)
	assert_true(reader.is_valid())


func test_write_read_varint_unsigned_multi_byte():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing values requiring multiple bytes.
	writer.write_varint_unsigned(128)
	writer.write_varint_unsigned(16383)
	writer.write_varint_unsigned(16384)
	writer.write_varint_unsigned(2097151)
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_varint_unsigned(), 128)
	assert_eq(reader.read_varint_unsigned(), 16383)
	assert_eq(reader.read_varint_unsigned(), 16384)
	assert_eq(reader.read_varint_unsigned(), 2097151)
	assert_true(reader.is_valid())


func test_write_read_varint_unsigned_large():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing a large value.
	var large_val := 0x7FFFFFFFFFFFFFFF # Max positive int64
	writer.write_varint_unsigned(large_val)
	var data := writer.to_bytes()
	# Then: Reading returns the same value.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_varint_unsigned(), large_val)
	assert_true(reader.is_valid())


func test_write_read_varint_signed_positive():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing positive signed values.
	writer.write_varint_signed(0)
	writer.write_varint_signed(1)
	writer.write_varint_signed(63)
	writer.write_varint_signed(64)
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_varint_signed(), 0)
	assert_eq(reader.read_varint_signed(), 1)
	assert_eq(reader.read_varint_signed(), 63)
	assert_eq(reader.read_varint_signed(), 64)
	assert_true(reader.is_valid())


func test_write_read_varint_signed_negative():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing negative signed values.
	writer.write_varint_signed(-1)
	writer.write_varint_signed(-64)
	writer.write_varint_signed(-65)
	writer.write_varint_signed(-1000000)
	var data := writer.to_bytes()
	# Then: Reading returns correct values.
	var reader := Bitstream.Reader.new(data)
	assert_eq(reader.read_varint_signed(), -1)
	assert_eq(reader.read_varint_signed(), -64)
	assert_eq(reader.read_varint_signed(), -65)
	assert_eq(reader.read_varint_signed(), -1000000)
	assert_true(reader.is_valid())


func test_varint_byte_alignment():
	# Given: A writer with bits before a varint.
	var writer := Bitstream.Writer.new()
	writer.write_bits(0b101, 3) # 3 bits, not byte-aligned
	writer.write_varint_unsigned(300) # Should align first
	var data := writer.to_bytes()
	# When: Reading with the same pattern.
	var reader := Bitstream.Reader.new(data)
	var bits := reader.read_bits(3)
	var varint := reader.read_varint_unsigned()
	# Then: Values are correct.
	assert_eq(bits, 0b101)
	assert_eq(varint, 300)
	assert_true(reader.is_valid())


# ---------------------------------------------------------------------------- #
#                              Tests: Composite                                #
# ---------------------------------------------------------------------------- #


func test_write_read_bytes():
	# Given: A writer and some bytes.
	var writer := Bitstream.Writer.new()
	var original := PackedByteArray([0x00, 0x11, 0x22, 0x33, 0xFF])
	# When: Writing and reading bytes.
	writer.write_bytes(original)
	var data := writer.to_bytes()
	var reader := Bitstream.Reader.new(data)
	var result := reader.read_bytes(5)
	# Then: The bytes match.
	assert_eq(result, original)
	assert_true(reader.is_valid())


func test_write_read_bytes_empty():
	# Given: A writer and empty bytes.
	var writer := Bitstream.Writer.new()
	var original := PackedByteArray()
	# When: Writing and reading empty bytes.
	writer.write_bytes(original)
	var data := writer.to_bytes()
	var reader := Bitstream.Reader.new(data)
	var result := reader.read_bytes(0)
	# Then: The result is empty.
	assert_eq(result.size(), 0)
	assert_true(reader.is_valid())


func test_write_read_string():
	# Given: A writer and a string.
	var writer := Bitstream.Writer.new()
	var original := "Hello, World!"
	# When: Writing and reading the string.
	writer.write_string(original)
	var data := writer.to_bytes()
	var reader := Bitstream.Reader.new(data)
	var result := reader.read_string()
	# Then: The strings match.
	assert_eq(result, original)
	assert_true(reader.is_valid())


func test_write_read_string_empty():
	# Given: A writer and an empty string.
	var writer := Bitstream.Writer.new()
	var original := ""
	# When: Writing and reading the empty string.
	writer.write_string(original)
	var data := writer.to_bytes()
	var reader := Bitstream.Reader.new(data)
	var result := reader.read_string()
	# Then: The result is empty.
	assert_eq(result, "")
	assert_true(reader.is_valid())


func test_write_read_string_unicode():
	# Given: A writer and a Unicode string.
	var writer := Bitstream.Writer.new()
	var original := "Hello, \u4e16\u754c! \U0001F600" # "Hello, 世界! 😀"
	# When: Writing and reading the Unicode string.
	writer.write_string(original)
	var data := writer.to_bytes()
	var reader := Bitstream.Reader.new(data)
	var result := reader.read_string()
	# Then: The strings match.
	assert_eq(result, original)
	assert_true(reader.is_valid())


func test_bytes_byte_alignment():
	# Given: A writer with bits before bytes.
	var writer := Bitstream.Writer.new()
	writer.write_bits(0b1111, 4) # 4 bits, not byte-aligned
	writer.write_bytes(PackedByteArray([0xAB, 0xCD]))
	var data := writer.to_bytes()
	# When: Reading with the same pattern.
	var reader := Bitstream.Reader.new(data)
	var bits := reader.read_bits(4)
	var bytes := reader.read_bytes(2)
	# Then: Values are correct.
	assert_eq(bits, 0b1111)
	assert_eq(bytes, PackedByteArray([0xAB, 0xCD]))
	assert_true(reader.is_valid())


# ---------------------------------------------------------------------------- #
#                            Tests: Error Handling                             #
# ---------------------------------------------------------------------------- #


func test_read_past_end_bool():
	# Given: An empty reader.
	var reader := Bitstream.Reader.new(PackedByteArray())
	# When: Reading past the end.
	var result := reader.read_bool()
	# Then: Returns false and sets error.
	assert_false(result)
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_read_past_end_bits():
	# Given: A reader with 1 byte.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF]))
	# When: Reading more bits than available.
	var result := reader.read_bits(16)
	# Then: Returns 0 and sets error.
	assert_eq(result, 0)
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_read_past_end_bytes():
	# Given: A reader with 2 bytes.
	var reader := Bitstream.Reader.new(PackedByteArray([0x11, 0x22]))
	# When: Reading more bytes than available.
	var result := reader.read_bytes(5)
	# Then: Returns empty and sets error.
	assert_eq(result.size(), 0)
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_error_persistence():
	# Given: A reader that will error.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF]))
	# When: Triggering an error then reading more.
	reader.read_bits(16) # Error
	var result := reader.read_bits(8) # Subsequent read
	# Then: Error persists and returns default.
	assert_eq(result, 0)
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_read_bits_invalid_count():
	# Given: A reader.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF, 0xFF]))
	# When: Reading with invalid bit count (0 or >64).
	var result1 := reader.read_bits(0)
	# Then: Returns 0 and sets error.
	assert_eq(result1, 0)
	assert_false(reader.is_valid())


func test_can_read_bits():
	# Given: A reader with 2 bytes.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF, 0xFF]))
	# Then: can_read_bits returns correct values.
	assert_true(reader.can_read_bits(1))
	assert_true(reader.can_read_bits(16))
	assert_false(reader.can_read_bits(17))


func test_can_read_bytes():
	# Given: A reader with 3 bytes.
	var reader := Bitstream.Reader.new(PackedByteArray([0x11, 0x22, 0x33]))
	# Then: can_read_bytes returns correct values.
	assert_true(reader.can_read_bytes(1))
	assert_true(reader.can_read_bytes(3))
	assert_false(reader.can_read_bytes(4))


func test_bits_remaining():
	# Given: A reader with 2 bytes.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF, 0xFF]))
	# Then: bits_remaining is correct.
	assert_eq(reader.bits_remaining(), 16)
	# When: Reading some bits.
	reader.read_bits(5)
	# Then: bits_remaining updates.
	assert_eq(reader.bits_remaining(), 11)


func test_is_at_end():
	# Given: A reader with 1 byte.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF]))
	# Then: Not at end initially.
	assert_false(reader.is_at_end())
	# When: Reading all bits.
	reader.read_bits(8)
	# Then: Is at end.
	assert_true(reader.is_at_end())


# ---------------------------------------------------------------------------- #
#                              Tests: Navigation                               #
# ---------------------------------------------------------------------------- #


func test_skip_bits():
	# Given: A reader with test data.
	var reader := Bitstream.Reader.new(PackedByteArray([0x12, 0x34, 0x56]))
	# When: Skipping 8 bits.
	reader.skip_bits(8)
	# Then: Next read starts from skipped position.
	assert_eq(reader.read_u8(), 0x34)
	assert_true(reader.is_valid())


func test_skip_bits_past_end():
	# Given: A reader with 1 byte.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF]))
	# When: Skipping past the end.
	reader.skip_bits(16)
	# Then: Error is set.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_FILE_EOF)


func test_seek():
	# Given: A reader with test data.
	var reader := Bitstream.Reader.new(PackedByteArray([0x12, 0x34, 0x56]))
	# When: Seeking to bit position 8.
	reader.seek(8)
	# Then: Next read is from that position.
	assert_eq(reader.read_u8(), 0x34)
	assert_true(reader.is_valid())


func test_seek_to_start():
	# Given: A reader that has read some data.
	var reader := Bitstream.Reader.new(PackedByteArray([0x12, 0x34]))
	reader.read_u8()
	# When: Seeking back to start.
	reader.seek(0)
	# Then: Can re-read from the beginning.
	assert_eq(reader.read_u8(), 0x12)
	assert_true(reader.is_valid())


func test_seek_invalid():
	# Given: A reader.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF]))
	# When: Seeking past the end.
	reader.seek(100)
	# Then: Error is set.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_INVALID_PARAMETER)


func test_seek_negative():
	# Given: A reader.
	var reader := Bitstream.Reader.new(PackedByteArray([0xFF]))
	# When: Seeking to negative position.
	reader.seek(-1)
	# Then: Error is set.
	assert_false(reader.is_valid())
	assert_eq(reader.get_error(), ERR_INVALID_PARAMETER)


# ---------------------------------------------------------------------------- #
#                          Tests: Writer Utilities                             #
# ---------------------------------------------------------------------------- #


func test_writer_bit_position():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# Then: Initial position is 0.
	assert_eq(writer.bit_position(), 0)
	# When: Writing bits.
	writer.write_bits(0xFF, 5)
	# Then: Position updates.
	assert_eq(writer.bit_position(), 5)


func test_writer_byte_length():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# Then: Initial byte length is 0.
	assert_eq(writer.length(), 0)
	# When: Writing 3 bits.
	writer.write_bits(0b111, 3)
	# Then: Byte length is 1 (rounded up).
	assert_eq(writer.length(), 1)
	# When: Writing 6 more bits.
	writer.write_bits(0b111111, 6)
	# Then: Byte length is 2.
	assert_eq(writer.length(), 2)


func test_writer_clear():
	# Given: A writer with data.
	var writer := Bitstream.Writer.new()
	writer.write_u32(0xDEADBEEF)
	# When: Clearing.
	writer.clear()
	# Then: Writer is reset.
	assert_eq(writer.bit_position(), 0)
	assert_eq(writer.length(), 0)


func test_writer_buffer_growth():
	# Given: A writer.
	var writer := Bitstream.Writer.new()
	# When: Writing more than the initial buffer size (64 bytes).
	for i in range(100):
		writer.write_i64(i)
	var data := writer.to_bytes()
	# Then: All data is preserved.
	var reader := Bitstream.Reader.new(data)
	for i in range(100):
		assert_eq(reader.read_i64(), i)
	assert_true(reader.is_valid())


# ---------------------------------------------------------------------------- #
#                          Tests: Complex Roundtrips                           #
# ---------------------------------------------------------------------------- #


func test_roundtrip_mixed_types():
	# Given: A writer with various types.
	var writer := Bitstream.Writer.new()
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
	var reader := Bitstream.Reader.new(data)

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


func test_roundtrip_all_integer_types():
	# Given: A writer with all integer types.
	var writer := Bitstream.Writer.new()
	writer.write_u8(200)
	writer.write_u16(50000)
	writer.write_u32(3000000000)
	writer.write_i8(-100)
	writer.write_i16(-30000)
	writer.write_i32(-2000000000)
	writer.write_i64(-9000000000000000000)
	var data := writer.to_bytes()

	# When: Reading all values.
	var reader := Bitstream.Reader.new(data)

	# Then: All values match.
	assert_eq(reader.read_u8(), 200)
	assert_eq(reader.read_u16(), 50000)
	assert_eq(reader.read_u32(), 3000000000)
	assert_eq(reader.read_i8(), -100)
	assert_eq(reader.read_i16(), -30000)
	assert_eq(reader.read_i32(), -2000000000)
	assert_eq(reader.read_i64(), -9000000000000000000)
	assert_true(reader.is_valid())


func test_roundtrip_nested_messages_pattern():
	# Given: A pattern simulating nested message encoding.
	var writer := Bitstream.Writer.new()
	# Outer message header
	writer.write_varint_unsigned(2) # Message type
	# Inner message 1
	writer.write_bool(true)
	writer.write_bits(255, 8)
	# Inner message 2
	writer.write_bool(false)
	writer.write_bits(128, 8)
	var data := writer.to_bytes()

	# When: Reading the nested structure.
	var reader := Bitstream.Reader.new(data)

	# Then: Structure is preserved.
	assert_eq(reader.read_varint_unsigned(), 2)
	assert_true(reader.read_bool())
	assert_eq(reader.read_bits(8), 255)
	assert_false(reader.read_bool())
	assert_eq(reader.read_bits(8), 128)
	assert_true(reader.is_valid())
