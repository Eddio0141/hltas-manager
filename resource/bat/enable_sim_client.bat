@echo off
setlocal enableDelayedExpansion

cd ..\..\HALF_LIFE_DIR
set half_life_dir=%CD%
cd "%~dp0"

set sim_steam_api_dll=_sim.dll
set steam_api_dll=steam_api.dll

if exist "%half_life_dir%\%sim_steam_api_dll%" (
	echo "using simulator client steam dll"
	ren "%half_life_dir%\%steam_api_dll%" "_reset.dll"
	ren "%half_life_dir%\%sim_steam_api_dll%" "%steam_api_dll%"
)
