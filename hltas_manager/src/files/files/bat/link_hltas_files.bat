@echo off
setlocal enableDelayedExpansion

cd ..\..\HALF_LIFE_DIR
set link_dir=%CD%
cd "%~dp0"

for %%f in (*.hltas) do (
	if exist "%link_dir%\%%f" (
		del "%link_dir%\%%f"
	)
	
	mklink /h "%link_dir%\%%f" "%~dp0%%f"
)