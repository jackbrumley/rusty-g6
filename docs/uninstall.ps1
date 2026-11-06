<#
.SYNOPSIS
  Rusty G6 uninstaller for Windows.
.EXAMPLE
  irm https://brumley.info/rusty-g6/uninstall.ps1 | iex
#>

param(
  [switch]$PurgeData
)

$AppName = "rusty-g6"

function Log($Message) { Write-Host "[rusty-g6-uninstall] $Message" }

if (-not $PurgeData -and $env:RUSTY_G6_PURGE_DATA -eq "1") { $PurgeData = $true }

Log "Preparing Rusty G6 uninstall"

if (Get-Process -Name $AppName -ErrorAction SilentlyContinue) {
  Log "Closing running Rusty G6"
  Stop-Process -Name $AppName -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 1
}

$RegistryPaths = @(
  "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
  "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
  "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
)

$found = $false
foreach ($path in $RegistryPaths) {
  $entries = Get-ItemProperty -Path $path -ErrorAction SilentlyContinue | Where-Object {
    $_.DisplayName -and ($_.DisplayName -like "rusty-g6*" -or $_.DisplayName -like "Rusty G6*")
  }
  foreach ($entry in $entries) {
    $found = $true
    Log "Removing $($entry.DisplayName) $($entry.DisplayVersion)"
    try {
      if ($entry.PSChildName -match "^\{[0-9A-Fa-f\-]+\}$") {
        # MSI installs are system-wide; removing one needs an Administrator PowerShell.
        $p = Start-Process -FilePath "msiexec.exe" -ArgumentList "/x `"$($entry.PSChildName)`" /qb /norestart" -Wait -PassThru
        if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) { Log "WARNING: MSI uninstall returned $($p.ExitCode) (system installs need PowerShell as Administrator)" }
      } elseif ($entry.QuietUninstallString) {
        Start-Process -FilePath "cmd.exe" -ArgumentList "/c `"$($entry.QuietUninstallString)`"" -Wait | Out-Null
      } elseif ($entry.UninstallString) {
        $exe = $entry.UninstallString.Trim('"')
        if (Test-Path $exe) {
          Start-Process -FilePath $exe -ArgumentList "/S" -Wait | Out-Null
        } else {
          Start-Process -FilePath "cmd.exe" -ArgumentList "/c `"$($entry.UninstallString) /S`"" -Wait | Out-Null
        }
      }
    } catch {
      Log "WARNING: could not remove $($entry.DisplayName): $_"
    }
  }
}
if (-not $found) { Log "No installed copy of Rusty G6 was found." }

# Start-with-Windows entry written by the app's autostart setting.
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name $AppName -ErrorAction SilentlyContinue

# ~/.config/rusty-g6, plus %APPDATA%\rusty-g6 from releases up to 1.0.12 in
# case the app never started again to move it.
$UserDataDirs = @(
  (Join-Path $env:USERPROFILE ".config\rusty-g6"),
  (Join-Path $env:APPDATA "rusty-g6")
)
foreach ($UserDataDir in $UserDataDirs) {
  if (-not (Test-Path $UserDataDir)) { continue }
  if ($PurgeData) {
    Log "Removing user data: $UserDataDir"
    Remove-Item -Path $UserDataDir -Recurse -Force -ErrorAction SilentlyContinue
  } else {
    Log "User data left at $UserDataDir"
  }
}

Log "Uninstall complete"
