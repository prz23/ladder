:: windows bat
set PROJECT_PATH=%cd%
set EXE_PATH=%PROJECT_PATH%\target\debug\supermatrix.exe
set BATH_PATH=%PROJECT_PATH%\target\

::set $env:RUST_LOG='info'
start %EXE_PATH% --chain=local --base-path=%BATH_PATH% --key=Alice --validator
::start %EXE_PATH% --dev --base-path=%BATH_PATH%