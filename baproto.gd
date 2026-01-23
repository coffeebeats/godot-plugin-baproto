##
## baproto.gd
##
## A Godot addon that provides an editor plugin which compiles build-a-proto schema files
## into GDScript.
##

@tool
extends EditorPlugin

# -- DEFINITIONS --------------------------------------------------------------------- #

const NAMESPACE := "BAProto"

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Platform := preload("./import/platform.gd")
const ImportPlugin := preload("./import/plugin.gd")
const ProjectSetting := preload("./import/setting.gd")

# -- INITIALIZATION ------------------------------------------------------------------ #

var _import_plugin: EditorImportPlugin = null

# -- ENGINE METHODS (OVERRIDES) ------------------------------------------------------ #


func _enter_tree() -> void:
	# Initialize project settings
	ProjectSetting.binary_path()
	ProjectSetting.output_directory()

	# Clear platform cache
	Platform.clear_cache()

	# Register import plugin
	_import_plugin = ImportPlugin.new()
	add_import_plugin(_import_plugin)

	print("[baproto] Successfully loaded addon.")


func _exit_tree() -> void:
	# Remove import plugin
	if _import_plugin:
		remove_import_plugin(_import_plugin)
		_import_plugin = null

	print("[baproto] Successfully unloaded addon.")
