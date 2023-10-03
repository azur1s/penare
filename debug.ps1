# Build
cargo xtask bundle penare

$vst3_name = "PenareDebug.vst3"

# Rename file
$build = Join-Path $pwd "target/bundled/Penare.vst3/Contents/x86_64-win/Penare.vst3"
$build_to = Join-Path $pwd "target/bundled/Penare.vst3/Contents/x86_64-win/$vst3_name"
Write-Host "Renaming $build to $build_to"
Move-Item $build $build_to -Force
$build = $build_to

# Move the VST3 file to the target directory
$pwd = Get-Location
$to = $env:VST3_DIR
Write-Host "Moving $build to $to"
if ($to -eq $null) {
    Write-Host "VST3_DIR is not set. Skipping moving the VST3 file."
    exit 0
}
gsudo Copy-Item $build $to -Force
# Set "NIH_LOG" environment variable to a file path
$env:NIH_LOG = "$pwd/penare.log"
# Open Ableton
Start-Process "C:\ProgramData\Ableton\Live 11 Suite\Program\Ableton Live 11 Suite.exe"