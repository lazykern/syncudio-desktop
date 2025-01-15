#! /bin/bash

# check if tauri, bun, pnpm, or npm is installed
if ! command -v tauri &> /dev/null && ! command -v bun &> /dev/null && ! command -v pnpm &> /dev/null && ! command -v npm &> /dev/null; then
    echo "tauri, bun, pnpm, or npm is not installed"
    exit 1
fi

if command -v tauri &> /dev/null; then
    NO_STRIP=true tauri build --verbose
elif command -v bun &> /dev/null; then
    NO_STRIP=true bun tauri build --verbose
elif command -v pnpm &> /dev/null; then
    NO_STRIP=true pnpm tauri build --verbose
elif command -v npm &> /dev/null; then
    NO_STRIP=true npm run tauri build --verbose
fi

