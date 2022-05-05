@echo off
setlocal enabledelayedexpansion

set args=

for %%a in (%*) do (
	set args=!args!%%a% 
)

cd ..\..\
set root_dir=%cd%
cd %~dp0

set args=SUB_COMMAND %args%

"%root_dir%\hltas_manager.exe" %args% 2> "%root_dir%\stderr.txt"
type "%root_dir%\stderr.txt"
