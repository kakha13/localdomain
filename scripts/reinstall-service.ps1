$ErrorActionPreference = "Stop"

$BinDir = "C:\ProgramData\LocalDomain\bin"
$ServiceName = "localdomain-daemon"
$DaemonSrc = "E:\AppsD\localdomain\target\debug\localdomain-daemon.exe"
$DaemonDest = "$BinDir\localdomain-daemon.exe"

# Create directories
New-Item -ItemType Directory -Force -Path "$BinDir" | Out-Null
New-Item -ItemType Directory -Force -Path "C:\ProgramData\LocalDomain\certs" | Out-Null
New-Item -ItemType Directory -Force -Path "C:\ProgramData\LocalDomain\caddy" | Out-Null
New-Item -ItemType Directory -Force -Path "C:\ProgramData\LocalDomain\logs" | Out-Null

# Copy daemon binary
Copy-Item -Path $DaemonSrc -Destination $DaemonDest -Force
Write-Host "Copied daemon binary to $DaemonDest"

# Remove existing service if present
$svc = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($svc) {
    Write-Host "Stopping existing service..."
    Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
    sc.exe delete $ServiceName | Out-Null
    Start-Sleep -Seconds 2
}

# Create and start service
$result = sc.exe create $ServiceName binPath=$DaemonDest start=auto DisplayName="LocalDomain Daemon"
Write-Host "sc create: $result"

$result = sc.exe start $ServiceName
Write-Host "sc start: $result"

Start-Sleep -Seconds 2
Get-Service -Name $ServiceName | Format-List Name,Status
Write-Host "Done!"
