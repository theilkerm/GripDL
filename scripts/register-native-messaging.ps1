# PowerShell script to register GripDL as a Native Messaging Host on macOS
# Note: This is for reference - macOS typically uses bash scripts

param(
    [Parameter(Mandatory=$true)]
    [string]$AppPath
)

$AppName = "com.gripdl.app"
$ExecutablePath = Join-Path $AppPath "Contents\MacOS\gripdl"

if (-not (Test-Path $ExecutablePath)) {
    Write-Error "Executable not found at $ExecutablePath"
    exit 1
}

$ManifestDir = "$env:HOME\Library\Application Support\Mozilla\NativeMessagingHosts"
New-Item -ItemType Directory -Force -Path $ManifestDir | Out-Null

$ManifestFile = Join-Path $ManifestDir "$AppName.json"

$manifest = @{
    name = $AppName
    description = "GripDL Native Messaging Host"
    path = $ExecutablePath
    type = "stdio"
    allowed_extensions = @("gripdl@example.com")
} | ConvertTo-Json

Set-Content -Path $ManifestFile -Value $manifest

Write-Host "Native Messaging Host registered at: $ManifestFile"
Write-Host "Please restart Firefox for changes to take effect."

