$ErrorActionPreference = "Stop"

if (Test-Path .\target) {
    Remove-Item .\target -Recurse -Force
}
if (Test-Path .\home-automation-windows.zip) {
    Remove-Item .\home-automation-windows.zip -Force
}

New-Item -ItemType Directory -Path .\target | Out-Null
Copy-Item ..\target\release\home-automation-server.exe target\
Copy-Item ..\target\release\home-automation-streamdeck-client.exe target\
Compress-Archive -Path .\target\* -DestinationPath .\home-automation-windows.zip