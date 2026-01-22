@tool
extends EditorPlugin

const _NAMESPACE := "BAProto"

const ImportPlugin := preload("./import/plugin.gd")

var _import_plugin: EditorImportPlugin = null


func _enter_tree() -> void:
	# Initialize ProjectSettings for output directory configuration
	if not ProjectSettings.has_setting("baproto/generate/output_directory"):
		ProjectSettings.set_setting("baproto/generate/output_directory", "res://baproto")
		ProjectSettings.set_initial_value("baproto/generate/output_directory", "res://baproto")
		ProjectSettings.set_as_basic("baproto/generate/output_directory", true)
		ProjectSettings.add_property_info({
			"name": "baproto/generate/output_directory",
			"type": TYPE_STRING,
			"hint": PROPERTY_HINT_DIR,
		})
		ProjectSettings.save()

	_import_plugin = ImportPlugin.new()
	add_import_plugin(_import_plugin)

	# Register runtime autoload
	add_autoload_singleton(_NAMESPACE, "res://addons/baproto/runtime/runtime.gd")

	print("[baproto] Successfully loaded addon")


func _exit_tree() -> void:
	# Remove import plugin
	if _import_plugin:
		remove_import_plugin(_import_plugin)
		_import_plugin = null

	# Remove runtime autoload
	remove_autoload_singleton(_NAMESPACE)

	print("[baproto] Successfully unloaded addon")
