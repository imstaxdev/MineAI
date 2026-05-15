$ErrorActionPreference = "Stop"

npm.cmd install --prefix apps/mineia-launcher
npm.cmd run tauri --prefix apps/mineia-launcher -- build
