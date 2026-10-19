param(
  [string]$BuildDir = "dist"
)
if (-not (Test-Path $BuildDir)) {
  Write-Error "Build dir not found: $BuildDir"
  exit 1
}
Write-Host "OK: $BuildDir exists"
