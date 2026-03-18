#!/bin/bash
set -e

echo "==> Installing Tracelon on Arch Linux"

# Check dependencies
for cmd in rustc cargo node pnpm makepkg; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "Error: $cmd not found. Install it first."
    exit 1
  fi
done

# Build and install via makepkg
makepkg -si --noconfirm

echo "==> Done! Run 'tracelon' or find it in your app launcher."
