##
## runtime.gd
##
## `BitStream` is a namespace for binary stream utilities. It provides bit-level binary
## serialization with `Writer` and `Reader` classes. Supports LEB128 varints, ZigZag
## encoding, and IEEE 754 floats.
##

class_name BitStream


# -- PUBLIC METHODS ------------------------------------------------------------------ #

## `zigzag_encode` converts a signed integer to an unsigned ZigZag-encoded value. ZigZag
## encoding maps negative numbers to odd numbers and positive numbers to even numbers.
static func zigzag_encode(value: int) -> int:
	return (value << 1) ^ (value >> 63)


## `zigzag_decode` converts a ZigZag-encoded value back to a signed integer.
static func zigzag_decode(value: int) -> int:
	return ((value >> 1) ^ (- (value & 1)))

# -- DEFINITIONS: _Base -------------------------------------------------------------- #

## `_Base` provides shared state and utilities for bit streams.
class _Base extends RefCounted:
	## `VARINT_BYTES_MAX` is the maximum byte count for a LEB128 varint (64 bit
	## value).
	const VARINT_BYTES_MAX := 10

	var _buffer: PackedByteArray
	var _position: int = 0

	## `bit_position` returns the current bit position.
	func bit_position() -> int:
		return _position

	## `capacity` returns the total number of allocated bits in the buffer.
	func capacity() -> int:
		return _buffer.size() * 8

# -- DEFINITIONS: Reader ------------------------------------------------------------- #


## `Reader` is a bit-level binary reader with error tracking.
class Reader extends _Base:
	## Error state for read operations.
	var _error: Error = OK

	func _init(data: PackedByteArray) -> void:
		_buffer = data
		_position = 0
		_error = OK

	## `bits_remaining` returns the number of bits left to read.
	func bits_remaining() -> int:
		return maxi(0, capacity() - _position)

	## `can_read_bits` returns true if the specified number of bits is available.
	func can_read_bits(bit_count: int) -> bool:
		return _position + bit_count <= capacity()

	## `can_read_bytes` returns true if the specified number of bytes is available.
	func can_read_bytes(byte_count: int) -> bool:
		return can_read_bits(byte_count * 8)

	## `get_error` returns the current error state.
	func get_error() -> Error:
		return _error


	## `is_at_end` returns true if all bits have been read.
	func is_at_end() -> bool:
		return _position >= capacity()


	## `is_valid` returns true if no error has occurred.
	func is_valid() -> bool:
		return _error == OK
	
	## `_set_error` sets the error state if not already set.
	func _set_error(err: Error) -> void:
		if _error == OK:
			_error = err

	## `_align_to_byte` advances the bit position to the next byte boundary.
	func _align_to_byte() -> void:
		if _position % 8 != 0:
			@warning_ignore("INTEGER_DIVISION")
			_position = ((_position + 7) / 8) * 8

	## `read_bool` reads a single bit and returns it as a bool.
	func read_bool() -> bool:
		if _error != OK:
			return false

		if not can_read_bits(1):
			_set_error(ERR_FILE_EOF)
			return false

		@warning_ignore("INTEGER_DIVISION")
		var byte_index := _position / 8
		var bit_offset := _position % 8
		var result := (_buffer[byte_index] >> bit_offset) & 1
		_position += 1
		return result == 1

	## `read_bits` reads an unsigned value from the specified number of bits (1-64).
	func read_bits(bit_count: int) -> int:
		if _error != OK:
			return 0

		if bit_count < 1 or bit_count > 64:
			_set_error(ERR_INVALID_PARAMETER)
			return 0

		if not can_read_bits(bit_count):
			_set_error(ERR_FILE_EOF)
			return 0

		# Read bits LSB-first
		var result := 0
		var remaining := bit_count
		var shift := 0
		while remaining > 0:
			@warning_ignore("INTEGER_DIVISION")
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
		return BitStream.zigzag_decode(read_bits(bit_count))

	## `read_u8` reads an unsigned 8-bit integer.
	func read_u8() -> int:
		return read_bits(8)

	## `read_u16` reads an unsigned 16-bit integer.
	func read_u16() -> int:
		return read_bits(16)

	## `read_u32` reads an unsigned 32-bit integer.
	func read_u32() -> int:
		return read_bits(32)

	## `read_u64` reads an unsigned 64-bit integer.
	##
	## NOTE: Values > 2^63-1 will appear as negative in GDScript.
	func read_u64() -> int:
		assert(
			false,
			"unsupported; GDScript does not support unsigned 64-bit integers.",
		)

		return read_bits(64)

	## `read_i8` reads a signed 8-bit integer (two's complement).
	func read_i8() -> int:
		var val := read_bits(8)
		if val >= 0x80:
			return val - 0x100
		return val

	## `read_i16` reads a signed 16-bit integer (two's complement).
	func read_i16() -> int:
		var val := read_bits(16)
		if val >= 0x8000:
			return val - 0x10000
		return val

	## `read_i32` reads a signed 32-bit integer (two's complement).
	func read_i32() -> int:
		var val := read_bits(32)
		if val >= 0x80000000:
			return val - 0x100000000
		return val

	## `read_i64` reads a signed 64-bit integer.
	func read_i64() -> int:
		return read_bits(64)

	## `read_f32` reads an IEEE 754 single-precision float.
	func read_f32() -> float:
		var bits := read_bits(32)
		if _error != OK:
			return 0.0
		var buf := PackedByteArray()
		buf.resize(4)
		buf.encode_u32(0, bits)
		return buf.decode_float(0)

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

		for i in range(_Base.VARINT_BYTES_MAX):
			if not can_read_bits(8):
				_set_error(ERR_FILE_EOF)
				return 0

			var byte := read_bits(8)
			result |= (byte & 0x7F) << shift
			shift += 7

			if (byte & 0x80) == 0:
				return result

		# Varint too long
		_set_error(ERR_INVALID_DATA)
		return 0

	## `read_varint_signed` reads a signed ZigZag + LEB128 varint (byte-aligned).
	func read_varint_signed() -> int:
		return BitStream.zigzag_decode(read_varint_unsigned())

	## `read_bytes` reads the specified number of raw bytes (byte-aligned).
	func read_bytes(byte_count: int) -> PackedByteArray:
		_align_to_byte()
		if not can_read_bytes(byte_count):
			_set_error(ERR_FILE_EOF)
			return PackedByteArray()

		var result := PackedByteArray()
		result.resize(byte_count)
		for i in range(byte_count):
			result[i] = read_bits(8)
		return result

	## `read_string` reads a varint-prefixed UTF-8 string (byte-aligned).
	func read_string() -> String:
		var length := read_varint_unsigned()
		if _error != OK:
			return ""
		var utf8 := read_bytes(length)
		if _error != OK:
			return ""
		return utf8.get_string_from_utf8()

	## `skip_bits` advances the bit position by the specified amount.
	func skip_bits(bit_count: int) -> void:
		if not can_read_bits(bit_count):
			_set_error(ERR_FILE_EOF)
			return
		_position += bit_count

	## `seek` sets the bit position to an absolute value.
	func seek(new_position: int) -> void:
		if new_position < 0 or new_position > capacity():
			_set_error(ERR_INVALID_PARAMETER)
			return
		_position = new_position


# -- DEFINITIONS: Writer ------------------------------------------------------------- #


## `Writer` is a bit-level binary writer with dynamic buffer growth.
class Writer extends _Base:
	## Initial buffer size in bytes.
	const INITIAL_BUFFER_SIZE := 64

	func _init() -> void:
		_buffer = PackedByteArray()
		_buffer.resize(INITIAL_BUFFER_SIZE)
		_position = 0

	## `_ensure_capacity` grows the buffer if needed to fit additional bits.
	func _ensure_capacity(bits_needed: int) -> void:
		var total_bits := _position + bits_needed
		@warning_ignore("INTEGER_DIVISION")
		var bytes_needed := (total_bits + 7) / 8 # Ceiling division
		if bytes_needed > _buffer.size():
			var new_size := _buffer.size()
			while new_size < bytes_needed:
				new_size *= 2
			_buffer.resize(new_size)

	## `_align_to_byte` advances the bit position to the next byte boundary.
	func _align_to_byte() -> void:
		if _position % 8 != 0:
			@warning_ignore("INTEGER_DIVISION")
			_position = ((_position + 7) / 8) * 8

	## `write_bool` writes a single bit.
	func write_bool(value: bool) -> void:
		_ensure_capacity(1)
		@warning_ignore("INTEGER_DIVISION")
		var byte_index := _position / 8
		var bit_offset := _position % 8
		if value:
			_buffer[byte_index] = _buffer[byte_index] | (1 << bit_offset)
		else:
			_buffer[byte_index] = _buffer[byte_index] & ~(1 << bit_offset)
		_position += 1

	## `write_bits` writes an unsigned value using the specified number of bits (1-64).
	func write_bits(value: int, bit_count: int) -> void:
		if bit_count < 1 or bit_count > 64:
			return
		_ensure_capacity(bit_count)

		# Write bits LSB-first
		var remaining := bit_count
		var val := value
		while remaining > 0:
			@warning_ignore("INTEGER_DIVISION")
			var byte_index := _position / 8
			var bit_offset := _position % 8
			var bits_in_byte := mini(8 - bit_offset, remaining)

			# Mask to extract the bits we want to write
			var mask := (1 << bits_in_byte) - 1
			var bits_to_write := val & mask

			# Clear target bits and write new value
			_buffer[byte_index] = (_buffer[byte_index] & ~(mask << bit_offset)) | (bits_to_write << bit_offset)

			val >>= bits_in_byte
			_position += bits_in_byte
			remaining -= bits_in_byte

	## `write_zigzag` writes a ZigZag-encoded signed value using fixed bits.
	func write_zigzag(value: int, bit_count: int) -> void:
		write_bits(BitStream.zigzag_encode(value), bit_count)

	## `write_u8` writes an unsigned 8-bit integer.
	func write_u8(value: int) -> void:
		write_bits(value & 0xFF, 8)

	## `write_u16` writes an unsigned 16-bit integer.
	func write_u16(value: int) -> void:
		write_bits(value & 0xFFFF, 16)

	## `write_u32` writes an unsigned 32-bit integer.
	func write_u32(value: int) -> void:
		write_bits(value & 0xFFFFFFFF, 32)

	## `write_u64` writes an unsigned 64-bit integer.
	func write_u64(value: int) -> void:
		write_bits(value, 64)

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
		var buf := PackedByteArray()
		buf.resize(4)
		buf.encode_float(0, value)
		var bits := buf.decode_u32(0)
		write_bits(bits, 32)

	## `write_f64` writes an IEEE 754 double-precision float.
	func write_f64(value: float) -> void:
		var buf := PackedByteArray()
		buf.resize(8)
		buf.encode_double(0, value)
		var lo := buf.decode_u32(0)
		var hi := buf.decode_u32(4)
		write_bits(lo, 32)
		write_bits(hi, 32)

	## `write_varint_unsigned` writes an unsigned LEB128 varint (byte-aligned).
	func write_varint_unsigned(value: int) -> void:
		_align_to_byte()
		var val := value
		# Handle negative values (treat as large unsigned)
		if val < 0:
			# For negative values, we need to write all 10 bytes
			for i in range(VARINT_BYTES_MAX):
				var byte := val & 0x7F
				val >>= 7
				if i < VARINT_BYTES_MAX - 1:
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
		write_varint_unsigned(BitStream.zigzag_encode(value))

	## `write_bytes` writes raw bytes (byte-aligned).
	func write_bytes(data: PackedByteArray) -> void:
		_align_to_byte()
		for i in range(data.size()):
			write_bits(data[i], 8)

	## `write_string` writes a varint-prefixed UTF-8 string (byte-aligned).
	func write_string(value: String) -> void:
		var utf8 := value.to_utf8_buffer()
		write_varint_unsigned(utf8.size())
		write_bytes(utf8)

	## `to_bytes` returns the buffer trimmed to the exact byte length needed.
	func to_bytes() -> PackedByteArray:
		@warning_ignore("INTEGER_DIVISION")
		var result := _buffer.slice(0, (_position + 7) / 8)
		return result

	## `length` returns the number of bytes written (rounded up).
	func length() -> int:
		@warning_ignore("INTEGER_DIVISION")
		return (_position + 7) / 8

	## `clear` resets the writer to its initial state.
	func clear() -> void:
		_buffer.fill(0)
		_position = 0