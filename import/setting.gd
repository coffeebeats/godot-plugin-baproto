##
## import/setting.gd
##
## Lightweight wrapper around Godot's 'ProjectSettings' API. Provides a cleaner
## interface for initializing, reading, and writing project settings.
##

@tool
extends RefCounted

# -- DEPENDENCIES -------------------------------------------------------------------- #

const ProjectSetting := preload("./setting.gd")

# -- INITIALIZATION ------------------------------------------------------------------ #

var _name: String

# -- PUBLIC METHODS ------------------------------------------------------------------ #


## `binary_path` returns a new `ProjectSetting` instance for the 'baproto' binary path
## setting.
static func binary_path() -> ProjectSetting:
	return ProjectSetting.new(
		"baproto/binary/path_override",
		"",
		false,
		TYPE_STRING,
		PROPERTY_HINT_FILE,
		"*.exe" if OS.get_name() == "Windows" else "*"
	)


## `output_directory` returns a new `ProjectSetting` instance for the output directory
## setting.
static func output_directory() -> ProjectSetting:
	return ProjectSetting.new(
		"baproto/generate/output_directory",
		"res://baproto",
		true,
		TYPE_STRING,
		PROPERTY_HINT_DIR
	)


## `clear` completely removes the setting from `ProjectSettings`.
##
## NOTE: This is the same thing as calling `set_value` with `null`.
func clear() -> void:
	set_value(null)


## `get_value` retrieves the current value of the setting.
func get_value() -> Variant:
	var value: Variant = ProjectSettings.get_setting(_name)
	assert(value != null, "invalid state; missing setting value")
	return value


## `set_value` updates the setting to the given value and saves `ProjectSettings`.
func set_value(value: Variant) -> void:
	ProjectSettings.set_setting(_name, value)
	ProjectSettings.save()


# -- ENGINE METHODS (OVERRIDES) ------------------------------------------------------ #


## `_init` creates a new `ProjectSetting` and initializes it in `ProjectSettings`.
func _init(
	name: String,
	default_value: Variant,
	basic: bool = true,
	type: int = TYPE_NIL,
	hint: int = PROPERTY_HINT_NONE,
	hint_string: String = ""
) -> void:
	_name = name

	if ProjectSettings.has_setting(_name):
		return

	ProjectSettings.set_setting(_name, default_value)
	ProjectSettings.set_initial_value(_name, default_value)
	ProjectSettings.set_as_basic(_name, basic)

	if type != TYPE_NIL:
		var property_info := {
			"name": _name,
			"type": type,
			"hint": hint,
		}

		if not hint_string.is_empty():
			property_info["hint_string"] = hint_string

		ProjectSettings.add_property_info(property_info)

	ProjectSettings.save()

	prints(
		"[baproto]", "Created project setting: %s (default=%s)" % [_name, default_value]
	)
