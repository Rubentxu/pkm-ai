#!/bin/bash
# PKM-AI Installation Script
# Installs PKM-AI skills to ~/.pkm-ai (default) or custom location

set -e

PKM_AI_DIR="${PKM_AI_DIR:-$HOME/.pkm-ai}"
BRANCH="${BRANCH:-main}"
FORCE_UPDATE="${FORCE_UPDATE:-false}"

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Install PKM-AI skills to a local directory for use across projects.

OPTIONS:
    -d DIR     Installation directory (default: ~/.pkm-ai)
    -b BRANCH  Git branch to install (default: main)
    -f         Force update if directory exists
    -h         Show this help message

EXAMPLES:
    # Default install to ~/.pkm-ai
    curl -fsSL https://raw.githubusercontent.com/Rubentxu/pkm-ai/trunk/workflows/pkmai/main/scripts/install.sh | bash

    # Install to custom location
    curl -fsSL https://raw.githubusercontent.com/Rubentxu/pkm-ai/trunk/workflows/pkmai/main/scripts/install.sh | bash -s -- -d /path/to/pkm-ai

    # Update existing installation
    ~/.pkm-ai/scripts/update.sh

EOF
}

# Parse arguments
while getopts "d:b:fh" opt; do
    case $opt in
        d) PKM_AI_DIR="$OPTARG";;
        b) BRANCH="$OPTARG";;
        f) FORCE_UPDATE="true";;
        h) usage; exit 0;;
        \?) usage; exit 1;;
    esac
done

# Resolve ~ to absolute path
PKM_AI_DIR="${PKM_AI_DIR/#\~/$HOME}"
PKM_AI_DIR="$(realpath "$PKM_AI_DIR" 2>/dev/null || echo "$PKM_AI_DIR")"

echo "Installing PKM-AI skills to $PKM_AI_DIR"

# Check if directory exists
if [ -d "$PKM_AI_DIR" ]; then
    if [ "$FORCE_UPDATE" = "true" ]; then
        echo "Force update enabled, pulling latest..."
        cd "$PKM_AI_DIR"
        git fetch origin --tags
        git checkout "$BRANCH"
        git pull origin "$BRANCH"
        git submodule update --init --recursive 2>/dev/null || true
    else
        echo "Installation already exists at $PKM_AI_DIR"
        echo "Run '~/.pkm-ai/scripts/update.sh' to update, or use -f to force update"
    fi
else
    # Create parent directory if needed
    mkdir -p "$(dirname "$PKM_AI_DIR")"

    # Clone repository
    echo "Cloning pkm-ai repository..."
    git clone -b "$BRANCH" --depth 1 https://github.com/Rubentxu/pkm-ai.git "$PKM_AI_DIR"

    # Initialize submodules
    git submodule update --init --recursive 2>/dev/null || true
fi

# Create environment file
cat > "$PKM_AI_DIR/.envrc" <<EOF
# PKM-AI Environment Configuration
# Source this file: source ~/.pkm-ai/.envrc

export PKM_AI_DIR="$PKM_AI_DIR"
export PKM_AI_SHARED="$PKM_AI_DIR/workflows/pkmai/sdd/_shared"
export PKM_AI_SKILLS="$PKM_AI_DIR/workflows/pkmai/skills"

# Add to PATH if needed
# export PATH="\$PKM_AI_DIR/scripts:\$PATH"
EOF

# Get version
VERSION=$(cd "$PKM_AI_DIR" && git describe --tags 2>/dev/null || echo "0.0.0")

echo ""
echo "=========================================="
echo "PKM-AI v$VERSION installed successfully!"
echo "=========================================="
echo ""
echo "Location: $PKM_AI_DIR"
echo ""
echo "To use in your shell, run:"
echo "  source $PKM_AI_DIR/.envrc"
echo ""
echo "To list available skills:"
echo "  ~/.pkm-ai/scripts/list-skills.sh"
echo ""
echo "Documentation: $PKM_AI_DIR/docs/"
