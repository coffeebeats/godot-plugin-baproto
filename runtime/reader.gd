##
## reader.gd
##
## `Reader` is a bit-level binary reader with error tracking. Provides methods for
## reading bits, bytes, varints, and various data types from a packed byte array.
##

extends "./bitstream.gd"

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Encoding := preload("./encoding.gd")

# -- PUBLIC METHODS ------------------------------------------------------------------ #


## `can_read_bits` returns true if the specified number of bits is available.
func can_read_bits(count: int) -> bool:
	return _position + count <= get_capacity()


## `can_read_bytes` returns true if the specified number of bytes is available.
func can_read_bytes(count: int) -> bool:
	return can_read_bits(count * 8)


## `read_bool` reads a single bit and returns it as a bool.
func read_bool() -> bool:
	if _error != OK:
		return false

	if not can_read_bits(1):
		_set_error(ERR_FILE_EOF)
		return false

	@warning_ignore("integer_division")
	var byte_index := _position / 8
	var bit_offset := _position % 8
	var result := (_buffer[byte_index] >> bit_offset) & 1
	_position += 1

	return result == 1


## `read_bits` reads an unsigned value from the specified number of bits (1-64).
func read_bits(count: int) -> int:
	if _error != OK:
		return 0

	if count < 1 or count > 64:
		_set_error(ERR_INVALID_PARAMETER)
		return 0

	if not can_read_bits(count):
		_set_error(ERR_FILE_EOF)
		return 0

	var result := 0
	var remaining := count
	var shift := 0

	while remaining > 0:
		@warning_ignore("integer_division")
		var byte_index := _position / 8
		var bit_offset := _position % 8
		var bits_in_byte := mini(8 - bit_offset, remaining)

		var mask := (1 << bits_in_byte) - 1
		var bits := (_buffer[byte_index] >> bit_offset) & mask

		result |= bits << shift

		shift += bits_in_byte
		_position += bits_in_byte
		remaining -= bits_in_byte

	return result


## `read_zigzag` reads a ZigZag-encoded signed value from fixed bits.
func read_zigzag(bit_count: int) -> int:
	return Encoding.zigzag_decode(read_bits(bit_count))


## `read_u8` reads an unsigned 8-bit integer.
func read_u8() -> int:
	return read_bits(8)


## `read_u16` reads an unsigned 16-bit integer.
func read_u16() -> int:
	return read_bits(16)


## `read_u32` reads an unsigned 32-bit integer.
func read_u32() -> int:
	return read_bits(32)


## `read_i8` reads a signed 8-bit integer (two's complement).
func read_i8() -> int:
	var value := read_bits(8)
	return value - 0x100 if value >= 0x80 else value


## `read_i16` reads a signed 16-bit integer (two's complement).
func read_i16() -> int:
	var value := read_bits(16)
	return value - 0x10000 if value >= 0x8000 else value


## `read_i32` reads a signed 32-bit integer (two's complement).
func read_i32() -> int:
	var value := read_bits(32)
	return value - 0x100000000 if value >= 0x80000000 else value


## `read_i64` reads a signed 64-bit integer.
func read_i64() -> int:
	return read_bits(64)


## `read_f32` reads an IEEE 754 single-precision float.
func read_f32() -> float:
	var bits := read_bits(32)

	if _error != OK:
		return 0.0

	var b := PackedByteArray()
	b.resize(4)
	b.encode_u32(0, bits)

	return b.decode_float(0)


## `read_f64` reads an IEEE 754 double-precision float.
func read_f64() -> float:
	var lo := read_bits(32)
	var hi := read_bits(32)

	if _error != OK:
		return 0.0

	var buf := PackedByteArray()
	buf.resize(8)
	buf.encode_u32(0, lo)
	buf.encode_u32(4, hi)

	return buf.decode_double(0)


## `read_varint_unsigned` reads an unsigned LEB128 varint (byte-aligned).
func read_varint_unsigned() -> int:
	_align_to_byte()

	var result := 0
	var shift := 0

	for i in range(Encoding.VARINT_BYTES_MAX):
		if not can_read_bits(8):
			_set_error(ERR_FILE_EOF)
			return 0

		var byte := read_bits(8)
		result |= (byte & 0x7F) << shift
		shift += 7

		if (byte & 0x80) == 0:
			return result

	_set_error(ERR_INVALID_DATA) # Too long

	return 0


## `read_varint_signed` reads a signed ZigZag + LEB128 varint (byte-aligned).
func read_varint_signed() -> int:
	return Encoding.zigzag_decode(read_varint_unsigned())


## `read_bytes` reads the specified number of raw bytes (byte-aligned).
func read_bytes(count: int) -> PackedByteArray:
	_align_to_byte()

	if not can_read_bytes(count):
		_set_error(ERR_FILE_EOF)
		return PackedByteArray()

	var result := PackedByteArray()
	result.resize(count)

	for i in range(count):
		result[i] = read_bits(8)

	return result


## `read_string` reads a varint-prefixed UTF-8 string (byte-aligned).
func read_string() -> String:
	var size := read_varint_unsigned()
	if _error != OK:
		return ""

	var utf8 := read_bytes(size)
	if _error != OK:
		return ""

	return utf8.get_string_from_utf8()

# -- ENGINE METHODS (OVERRIDES) ------------------------------------------------------ #


func _init(data: PackedByteArray) -> void:
	_buffer = data