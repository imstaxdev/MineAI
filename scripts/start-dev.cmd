@echo off
cd /d "%~dp0..\apps\mineia-launcher"
npm.cmd run dev > "%~dp0..\logs\vite.stdout.log" 2> "%~dp0..\logs\vite.stderr.log"
