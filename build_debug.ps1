# Build
cargo xtask bundle penare

# Move the VST3 file to the target directory
$pwd = Get-Location
$build = Join-Path $pwd "target/bundled/Penare.vst3/Contents/x86_64-win/Penare.vst3"
$to = $env:VST3_DIR
Write-Host "Moving $build to $to"
if ($to -eq $null) {
    Write-Host "VST3_DIR is not set. Skipping moving the VST3 file."
    exit 0
}
gsudo Move-Item $build $to -Force
# Open Ableton
Start-Process "C:\ProgramData\Ableton\Live 11 Suite\Program\Ableton Live 11 Suite.exe"