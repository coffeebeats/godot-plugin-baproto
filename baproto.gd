@tool
extends EditorPlugin

const _NAMESPACE := "BAProto"


func _enter_tree() -> void:
	add_autoload_singleton(_NAMESPACE, "res://baproto.gd")

	print("[%s]: Successfully loaded addon.")


func _exit_tree() -> void:
	remove_autoload_singleton(_NAMESPACE)

	print("[%s]: Successfully unloaded addon.")
