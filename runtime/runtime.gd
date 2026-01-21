##
## baproto.gd
##
## A non-instantiable shared library that provides the BAProto runtime namespace.
## Provides access to `Reader`, `Writer`, and encoding utilities for bit-level binary
## serialization.
##

extends Node

# -- DEPENDENCIES -------------------------------------------------------------------- #

const Encoding := preload("./encoding.gd")
const Reader := preload("./reader.gd")
const Writer := preload("./writer.gd")
