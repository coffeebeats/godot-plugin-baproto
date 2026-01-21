##
## bitstream.gd
##
## A base class for bit-level binary serialization classes. This class wraps an
## underlying buffer (`PackedByteArray`) and provides utilities for capacity management
## and managing a cursor position within the buffer.
##

extends RefCounted

# -- INITIALIZATION ------------------------------------------------------------------ #

var _buffer: PackedByteArray
var _error: Error = OK
var _position: int = 0

# -- PUBLIC METHODS ------------------------------------------------------------------ #


## `clear` resets the class to its initial state.
func clear() -> void:
	_buffer.resize(0)
	_error = OK
	_position = 0


## `get_capacity` returns the total number of allocated bits in the buffer.
func get_capacity() -> int:
	return _buffer.size() * 8


## `get_error` returns the current error state.
func get_error() -> Error:
	return _error


## `get_position` returns the current bit position.
func get_position() -> int:
	return _position


## `is_at_end` returns true if all bits have been read.
func is_at_end() -> bool:
	return _position >= get_capacity()


## `is_valid` returns true if no error has occurred.
func is_valid() -> bool:
	return _error == OK


## `length` returns the number of bytes used (rounded up to nearest byte).
func length() -> int:
	@warning_ignore("integer_division")
	return (_position + 7) / 8


## `seek` sets the bit position to an absolute value.
func seek(position: int) -> void:
	if position < 0 or position > get_capacity():
		_set_error(ERR_INVALID_PARAMETER)
		return

	_position = position


## `set_data` resets the `BitStream` to the provided `data` array. Any errors or cursor
## positioning will be reset.
func set_data(data: PackedByteArray) -> void:
	_buffer = data
	_error = OK
	_position = 0


## `to_bytes` returns the underyling buffer trimmed to the exact byte length needed.
func to_bytes() -> PackedByteArray:
	@warning_ignore("integer_division")
	return _buffer.slice(0, (_position + 7) / 8)


# -- ENGINE METHODS (OVERRIDES) ------------------------------------------------------ #


func _init(data: PackedByteArray = PackedByteArray()) -> void:
	_buffer = data


# -- PRIVATE METHODS ----------------------------------------------------------------- #


## `_align_to_byte` advances the bit position to the next byte boundary.
func _align_to_byte() -> void:
	if _position % 8 != 0:
		@warning_ignore("integer_division")
		_position = ((_position + 7) / 8) * 8


## `_ensure_capacity` grows the buffer, if needed, to fit additional capacity.
func _ensure_capacity(capacity: int) -> void:
	var total := _position + capacity

	@warning_ignore("integer_division")
	var needed: int = ceil((total + 7) / 8)
	if needed > _buffer.size():
		var size_prev := _buffer.size()
		var size := maxi(size_prev, 1)

		while size < needed:
			size *= 2

		_buffer.resize(size)

		for i in range(size_prev, size):
			_buffer[i] = 0


## `_set_error` sets the error state if not already set.
func _set_error(err: Error) -> void:
	if _error == OK:
		_error = err
