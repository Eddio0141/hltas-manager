@echo off
set "game_dir=GAME_DIR"
set "client_dll_full=%game_dir%\client.dll"

if exist %client_dll_full% (
	ren "%client_dll_full%" vanilla_client.dll
)