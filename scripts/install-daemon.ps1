# install-daemon.ps1 — Build and install the localdomain daemon as a Windows Service
# Must be run as Administrator

$ErrorActionPreference = "Stop"

$BinDir = "C:\ProgramData\LocalDomain\bin"
$DaemonBin = "target\release\localdomain-daemon.exe"
$ServiceName = "localdomain-daemon"

if (-not (Test-Path $DaemonBin)) {
    Write-Host "Building daemon in release mode..."
    cargo build -p localdomain-daemon --release
    if ($LASTEXITCODE -ne 0) { throw "Build failed" }
}

# Create directories
New-Item -ItemType Directory -Force -Path "$BinDir" | Out-Null
New-Item -ItemType Directory -Force -Path "C:\ProgramData\LocalDomain\certs" | Out-Null
New-Item -ItemType Directory -Force -Path "C:\ProgramData\LocalDomain\caddy" | Out-Null
New-Item -ItemType Directory -Force -Path "C:\ProgramData\LocalDomain\logs" | Out-Null

# Stop and remove existing service if present
$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($svc) {
    Write-Host "Stopping existing service..."
    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
    sc.exe delete $ServiceName | Out-Null
    Start-Sleep -Seconds 1
}

# Copy binary
Copy-Item -Path $DaemonBin -Destination "$BinDir\localdomain-daemon.exe" -Force

# Create and start service
sc.exe create $ServiceName binPath="$BinDir\localdomain-daemon.exe" start=auto DisplayName="LocalDomain Daemon" | Out-Null
sc.exe start $ServiceName | Out-Null

Write-Host "Daemon installed and started as Windows Service."
