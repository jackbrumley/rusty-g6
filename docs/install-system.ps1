# Rusty G6 system-wide (MSI) install for Windows. Run from PowerShell as Administrator:
#   irm https://brumley.info/rusty-g6/install-system.ps1 | iex
$env:RUSTY_G6_SYSTEM = "1"
irm https://brumley.info/rusty-g6/install.ps1 | iex
