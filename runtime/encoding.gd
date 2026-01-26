##
## encoding.gd
##
## A shared library providing bit-level binary serialization utilities. Supports ZigZag
## encoding and common constants for bit stream operations.
##

extends RefCounted

# -- DEFINITIONS --------------------------------------------------------------------- #

## `VARINT_BYTES_MAX` is the maximum byte count for a LEB128 varint (64 bit value).
const VARINT_BYTES_MAX := 10

# -- PUBLIC METHODS ------------------------------------------------------------------ #


## `zigzag_encode` converts a signed integer to an unsigned ZigZag-encoded value. ZigZag
## encoding maps negative numbers to odd numbers and positive numbers to even numbers.
static func zigzag_encode(value: int) -> int:
	return (value << 1) ^ (value >> 63)


## `zigzag_decode` converts a ZigZag-encoded value back to a signed integer.
static func zigzag_decode(value: int) -> int:
	return (value >> 1) ^ (-(value & 1))


# -- ENGINE METHODS (OVERRIDES) ------------------------------------------------------ #


func _init() -> void:
	assert(false, "Invalid config; this 'Object' should not be instantiated!")
