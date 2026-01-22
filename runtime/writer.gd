##
## writer.gd
##
## `Writer` is a bit-level binary writer with dynamic buffer growth. Provides methods
## for writing bits, bytes, varints, and various data types to a packed byte array.
##

extends "./bitstream.gd"

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Encoding := preload("./encoding.gd")

# -- INITIALIZATION ------------------------------------------------------------------ #

static var _f32_bytes := PackedByteArray([0, 0, 0, 0])
static var _f64_bytes := PackedByteArray([0, 0, 0, 0, 0, 0, 0, 0])

# -- PUBLIC METHODS ------------------------------------------------------------------ #


## `write_bool` writes a single bit.
func write_bool(value: bool) -> void:
	_ensure_capacity(1)

	@warning_ignore("integer_division")
	var byte_index := _position / 8
	var bit_offset := _position % 8

	if value:
		_buffer[byte_index] = _buffer[byte_index] | (1 << bit_offset)
	else:
		_buffer[byte_index] = _buffer[byte_index] & ~(1 << bit_offset)

	_position += 1


## `write_bits` writes an unsigned value using the specified number of bits (1-64).
func write_bits(value: int, count: int) -> void:
	if count < 1 or count > 64:
		_set_error(ERR_INVALID_PARAMETER)
		return

	_ensure_capacity(count)

	# Write bits LSB-first
	var val := value
	var remaining := count
	while remaining > 0:
		@warning_ignore("integer_division")
		var byte_index := _position / 8
		var bit_offset := _position % 8
		var bits_in_byte := mini(8 - bit_offset, remaining)

		# Mask to extract the bits we want to write
		var mask := (1 << bits_in_byte) - 1
		var bits_to_write := val & mask

		# Clear target bits and write new value
		_buffer[byte_index] = (
			(_buffer[byte_index] & ~(mask << bit_offset))
			| (bits_to_write << bit_offset)
		)

		val >>= bits_in_byte
		_position += bits_in_byte
		remaining -= bits_in_byte


## `write_zigzag` writes a ZigZag-encoded signed value using fixed bits.
func write_zigzag(value: int, bit_count: int) -> void:
	write_bits(Encoding.zigzag_encode(value), bit_count)


## `write_u8` writes an unsigned 8-bit integer.
func write_u8(value: int) -> void:
	write_bits(value & 0xFF, 8)


## `write_u16` writes an unsigned 16-bit integer.
func write_u16(value: int) -> void:
	write_bits(value & 0xFFFF, 16)


## `write_u32` writes an unsigned 32-bit integer.
func write_u32(value: int) -> void:
	write_bits(value & 0xFFFFFFFF, 32)


## `write_i8` writes a signed 8-bit integer (two's complement).
func write_i8(value: int) -> void:
	write_bits(value & 0xFF, 8)


## `write_i16` writes a signed 16-bit integer (two's complement).
func write_i16(value: int) -> void:
	write_bits(value & 0xFFFF, 16)


## `write_i32` writes a signed 32-bit integer (two's complement).
func write_i32(value: int) -> void:
	write_bits(value & 0xFFFFFFFF, 32)


## `write_i64` writes a signed 64-bit integer.
func write_i64(value: int) -> void:
	write_bits(value, 64)


## `write_f32` writes an IEEE 754 single-precision float.
func write_f32(value: float) -> void:
	_f32_bytes.encode_float(0, value)
	var bits := _f32_bytes.decode_u32(0)
	write_bits(bits, 32)


## `write_f64` writes an IEEE 754 double-precision float.
func write_f64(value: float) -> void:
	_f64_bytes.encode_double(0, value)
	var lo := _f64_bytes.decode_u32(0)
	var hi := _f64_bytes.decode_u32(4)
	write_bits(lo, 32)
	write_bits(hi, 32)


## `write_varint_unsigned` writes an unsigned LEB128 varint.
func write_varint_unsigned(value: int) -> void:
	var val := value
	# Handle negative values (treat as large unsigned)
	if val < 0:
		# For negative values, we need to write all 10 bytes
		for i in range(Encoding.VARINT_BYTES_MAX):
			var byte := val & 0x7F
			val >>= 7
			if i < Encoding.VARINT_BYTES_MAX - 1:
				byte |= 0x80
			write_bits(byte, 8)
		return

	# Positive values
	while true:
		var byte := val & 0x7F
		val >>= 7
		if val != 0:
			byte |= 0x80
		write_bits(byte, 8)
		if val == 0:
			break


## `write_varint_signed` writes a signed ZigZag + LEB128 varint (byte-aligned).
func write_varint_signed(value: int) -> void:
	write_varint_unsigned(Encoding.zigzag_encode(value))


## `write_bytes` writes raw bytes.
func write_bytes(data: PackedByteArray) -> void:
	for i in range(data.size()):
		write_bits(data[i], 8)


## `write_string` writes a varint-prefixed UTF-8 string (byte-aligned).
func write_string(value: String) -> void:
	var utf8 := value.to_utf8_buffer()
	write_varint_unsigned(utf8.size())
	write_bytes(utf8)


# -- ENGINE METHODS (OVERRIDES) ------------------------------------------------------ #


func _init() -> void:
	_buffer = PackedByteArray()
