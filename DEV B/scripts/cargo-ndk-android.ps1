$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root "android\app\src\main\jniLibs"
if (-not $env:ANDROID_NDK_HOME -and -not $env:NDK_HOME) {
    Write-Warning "Set ANDROID_NDK_HOME (or NDK_HOME) to your Android NDK root before building."
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust toolchain missing: `cargo` not found. Install Rust via rustup, then retry."
    Write-Host "Example:"
    Write-Host "  winget install Rustlang.Rustup"
    Write-Host "  rustup toolchain install stable"
    exit 1
}

if (-not (Get-Command cargo-ndk -ErrorAction SilentlyContinue)) {
    Write-Error "cargo-ndk missing. Install it with:"
    Write-Host "  cargo install cargo-ndk"
    exit 1
}
Push-Location $root
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o $out build -p shadowprompt-engine --release
Pop-Location
Write-Host "JNI libs output: $out"
