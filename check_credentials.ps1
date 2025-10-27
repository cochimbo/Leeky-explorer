# Check Windows Credential Manager for leeky-explorer entries
Write-Host "=== Checking Windows Credential Manager ===" -ForegroundColor Cyan
Write-Host ""

# Method 1: cmdkey (basic)
Write-Host "Method 1: Using cmdkey" -ForegroundColor Yellow
cmdkey /list | Select-String -Pattern "leeky" -Context 2,2

Write-Host ""
Write-Host "Method 2: Detailed search" -ForegroundColor Yellow
$all = cmdkey /list
$inLeeky = $false
for ($i = 0; $i -lt $all.Length; $i++) {
    if ($all[$i] -match "leeky") {
        $inLeeky = $true
        Write-Host $all[$i] -ForegroundColor Green
    }
    elseif ($inLeeky -and $all[$i] -match "^\s*$") {
        $inLeeky = $false
    }
    elseif ($inLeeky) {
        Write-Host $all[$i] -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "=== Checking connections.json ===" -ForegroundColor Cyan
$connectionsFile = "$env:APPDATA\leeky-explorer\connections.json"
if (Test-Path $connectionsFile) {
    $content = Get-Content $connectionsFile | ConvertFrom-Json
    Write-Host "Found $($content.connections.Count) connections:" -ForegroundColor Yellow
    foreach ($conn in $content.connections) {
        Write-Host "  - $($conn.name) ($($conn.connection_type))" -ForegroundColor White
        Write-Host "    Host: $($conn.host):$($conn.port)" -ForegroundColor Gray
        Write-Host "    User: $($conn.username)" -ForegroundColor Gray
        if ($conn.auth.Password) {
            Write-Host "    Auth: Password (stored=$($conn.auth.Password.stored))" -ForegroundColor $(if ($conn.auth.Password.stored) { "Green" } else { "Red" })
            if ($conn.auth.Password.password) {
                Write-Host "    WARNING: Plain-text password found in JSON!" -ForegroundColor Red
            }
        }
    }
}
else {
    Write-Host "No connections file found at $connectionsFile" -ForegroundColor Red
}

Write-Host ""
Write-Host "=== Checking recent logs ===" -ForegroundColor Cyan
$logFile = "$env:APPDATA\leeky-explorer\leeky.log"
if (Test-Path $logFile) {
    Write-Host "Last 10 keychain-related log entries:" -ForegroundColor Yellow
    Get-Content $logFile | Select-String -Pattern "keychain|password|stored|credential" | Select-Object -Last 10
}
else {
    Write-Host "No log file found at $logFile" -ForegroundColor Red
}
