:: windows bat
set PROJECT_PATH=%cd%
set EXE_PATH=%PROJECT_PATH%\target\debug\abmatrix.exe
set BATH_PATH=%PROJECT_PATH%\target\bob\

::set $env:RUST_LOG='info'
start %EXE_PATH% --chain=local --base-path=%BATH_PATH% --key=Bob --bootnodes /ip4/192.168.1.109/tcp/30333/p2p/QmQZ8TjTqeDj3ciwr93EJ95hxfDsb9pEYDizUAbWpigtQN --port 30334 --validator --listener 
::start %EXE_PATH% --dev --base-path=%BATH_PATH%