# reload-daemon.ps1 — Rebuild and restart the localdomain daemon service
# Must be run as Administrator

$ErrorActionPreference = "Stop"

$BinDir = "C:\ProgramData\LocalDomain\bin"
$ServiceName = "localdomain-daemon"

Write-Host "Building daemon..."
cargo build -p localdomain-daemon
if ($LASTEXITCODE -ne 0) { throw "Build failed" }

# Stop the service and kill any lingering process
$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($svc) {
    Write-Host "Stopping service..."
    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
    for ($i = 0; $i -lt 20; $i++) {
        Start-Sleep -Milliseconds 500
        $s = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
        if ($s.Status -eq 'Stopped') { break }
    }
}

# Kill any lingering daemon process that may hold the file lock
$procs = Get-Process -Name "localdomain-daemon" -ErrorAction SilentlyContinue
if ($procs) {
    Write-Host "Killing lingering daemon process..."
    $procs | Stop-Process -Force
    Start-Sleep -Seconds 2
}

# Copy new binary (retry up to 5 times if file is still locked)
$copied = $false
for ($i = 0; $i -lt 5; $i++) {
    try {
        Copy-Item -Path "target\debug\localdomain-daemon.exe" -Destination "$BinDir\localdomain-daemon.exe" -Force
        $copied = $true
        break
    } catch {
        Write-Host "File still locked, retrying in 2s..."
        Start-Sleep -Seconds 2
    }
}
if (-not $copied) { throw "Failed to copy daemon binary — file still locked" }

# Start the service
if ($svc) {
    Write-Host "Starting service..."
    sc.exe start $ServiceName | Out-Null
} else {
    Write-Host "Service not installed. Run install-daemon.ps1 first."
    exit 1
}

Write-Host "Daemon reloaded."
