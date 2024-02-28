@echo off
cargo build --release
flutter build windows --release
copy "target/release/minecraft_hold_api.dll" "build/windows/x64/runner/Release"
