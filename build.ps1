# Build
cargo xtask bundle penare --release

# Move the VST3 file to the target directory
$pwd = Get-Location
$build = Join-Path $pwd "target/bundled/Penare.vst3/Contents/x86_64-win/Penare.vst3"
$to = $env:VST3_DIR
Write-Host "Moving $build to $to"
if ($to -eq $null) {
    Write-Host "VST3_DIR is not set. Skipping moving the VST3 file."
    exit 0
}
gsudo Copy-Item $build $to -Force

$pkgid = cargo pkgid
$version = $pkgid.Split('#')[1]

Write-Host "Creating zip file for Penare $version"

$vst3 = $build
$clap = Join-Path $pwd "target/bundled/Penare.clap"
7z a -tzip -mx=9 "target/Penare.$version.zip" $vst3 $clap

Write-Host "Press any key to continue..."
$null = $Host.UI.RawUI.ReadKey('NoEcho,IncludeKeyDown')