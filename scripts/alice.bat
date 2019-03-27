:: windows bat
set PROJECT_PATH=%cd%
set EXE_PATH=%PROJECT_PATH%\target\debug\abmatrix.exe
set BATH_PATH=%PROJECT_PATH%\target\alice\

::set $env:RUST_LOG='info'
start %EXE_PATH% --chain=local --base-path=%BATH_PATH% --key=Alice --node-key 0000000000000000000000000000000000000000000000000000000000000001 --port 30333 --validator --listener --sender
::start %EXE_PATH% --dev --base-path=%BATH_PATH%