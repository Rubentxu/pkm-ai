#!/bin/bash
# PKM-AI Update Script
# Updates installed PKM-AI skills to latest version

set -e

PKM_AI_DIR="${PKM_AI_DIR:-$HOME/.pkm-ai}"

# Resolve ~ to absolute path
PKM_AI_DIR="${PKM_AI_DIR/#\~/$HOME}"
PKM_AI_DIR="$(realpath "$PKM_AI_DIR" 2>/dev/null || echo "$PKM_AI_DIR")"

if [ ! -d "$PKM_AI_DIR/.git" ]; then
    echo "Error: $PKM_AI_DIR is not a git repository."
    echo "Run install.sh first to install PKM-AI."
    exit 1
fi

echo "Updating PKM-AI..."

cd "$PKM_AI_DIR"

# Fetch latest
git fetch origin --tags

# Get current version
OLD_VERSION=$(git describe --tags 2>/dev/null || echo "unknown")

# Pull latest
git pull origin main

# Update submodules
git submodule update --init --recursive 2>/dev/null || true

# Get new version
NEW_VERSION=$(git describe --tags 2>/dev/null || echo "unknown")

echo ""
echo "Updated: $OLD_VERSION -> $NEW_VERSION"
echo "Location: $PKM_AI_DIR"
