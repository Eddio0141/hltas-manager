@echo off
GAME_DIR
set "client_dll_full=%game_dir%\vanilla_client.dll"

if exist %client_dll_full% (
	ren "%client_dll_full%" client.dll
)