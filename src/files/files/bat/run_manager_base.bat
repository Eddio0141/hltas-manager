@echo off
setlocal enabledelayedexpansion

echo "I recommend using the powershell scripts instead by running 'init' here without '--bat'"
echo "If you are using this then make sure you close TASView cuz I can't solve dumb thing with it"

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
