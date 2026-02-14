# download-caddy.ps1 — Download Caddy for Windows
# Downloads to C:\ProgramData\LocalDomain\bin\caddy.exe

$ErrorActionPreference = "Stop"

$BinDir = "C:\ProgramData\LocalDomain\bin"
$CaddyPath = "$BinDir\caddy.exe"

if (Test-Path $CaddyPath) {
    Write-Host "Caddy already exists at $CaddyPath"
    & $CaddyPath version
    exit 0
}

# Determine architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "386" }
$url = "https://caddyserver.com/api/download?os=windows&arch=$arch"

Write-Host "Downloading Caddy for windows/$arch..."
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

Invoke-WebRequest -Uri $url -OutFile $CaddyPath -UseBasicParsing
Write-Host "Caddy downloaded to $CaddyPath"
& $CaddyPath version
