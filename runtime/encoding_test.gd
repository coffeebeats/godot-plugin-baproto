##
## encoding_test.gd
##
## Test suite for encoding utilities.
##

extends GutTest

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Encoding := preload("res://runtime/encoding.gd")

# -- TEST METHODS -------------------------------------------------------------------- #


func test_zigzag_encode_zero() -> void:
	# Given: A zero value.
	# When: ZigZag encoding is applied.
	var encoded := Encoding.zigzag_encode(0)

	# Then: The result is 0.
	assert_eq(encoded, 0)


func test_zigzag_encode_positive(
	params = use_parameters([[1, 2], [2, 4], [100, 200]])
) -> void:
	# Given: A positive integer.
	var input: int = params[0]
	var expected: int = params[1]

	# When: ZigZag encoding is applied.
	var encoded := Encoding.zigzag_encode(input)

	# Then: Positive values map to even numbers (2n).
	assert_eq(encoded, expected)


func test_zigzag_encode_negative(
	params = use_parameters([[-1, 1], [-2, 3], [-100, 199]])
) -> void:
	# Given: A negative integer.
	var input: int = params[0]
	var expected: int = params[1]

	# When: ZigZag encoding is applied.
	var encoded := Encoding.zigzag_encode(input)

	# Then: Negative values map to odd numbers (2|n|-1).
	assert_eq(encoded, expected)


func test_zigzag_roundtrip() -> void:
	# Given: Various signed integers.
	var values := [0, 1, -1, 127, -128, 32767, -32768, 2147483647, -2147483648]

	for val in values:
		# When: Encoding then decoding.
		var encoded := Encoding.zigzag_encode(val)
		var decoded := Encoding.zigzag_decode(encoded)

		# Then: The original value is recovered.
		assert_eq(decoded, val, "Roundtrip failed for %d" % val)

# -- TEST HOOKS ---------------------------------------------------------------------- #


func before_all() -> void:
	# NOTE: Hide unactionable errors when using object doubles.
	ProjectSettings.set("debug/gdscript/warnings/native_method_override", false)