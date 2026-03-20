#!/bin/bash
# PKM-AI Installation Script
# Installs PKM-AI skills to ~/.pkm-ai (default) or custom location

set -e

PKM_AI_DIR="${PKM_AI_DIR:-$HOME/.pkm-ai}"
BRANCH="${BRANCH:-main}"
FORCE_UPDATE="${FORCE_UPDATE:-false}"
CREATE_SYMLINKS_ONLY="${CREATE_SYMLINKS_ONLY:-false}"

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Install PKM-AI skills to a local directory for use across projects.

OPTIONS:
    -d DIR     Installation directory (default: ~/.pkm-ai)
    -b BRANCH  Git branch to install (default: main)
    -f         Force update if directory exists
    -l         Create Claude Code command symlinks only (no install)
    -h         Show this help message

EXAMPLES:
    # Default install to ~/.pkm-ai
    curl -fsSL https://raw.githubusercontent.com/Rubentxu/pkm-ai/main/workflows/pkmai/scripts/install.sh | bash

    # Install to custom location
    curl -fsSL https://raw.githubusercontent.com/Rubentxu/pkm-ai/main/workflows/pkmai/scripts/install.sh | bash -s -- -d /path/to/pkm-ai

    # Update existing installation
    ~/.pkm-ai/scripts/update.sh

    # Create Claude Code symlinks only (after manual install)
    ~/.pkm-ai/scripts/install.sh -l

EOF
}

# Check if Claude Code is installed
is_claude_code_installed() {
    command -v claude >/dev/null 2>&1 || [ -d "$HOME/.claude" ]
}

# Create Claude Code command symlinks
create_claude_code_symlinks() {
    local skills_dir="$PKM_AI_DIR/workflows/pkmai/skills"
    local commands_dir="$HOME/.claude/commands"

    echo ""
    echo "Creating Claude Code command symlinks..."

    # Check if Claude Code commands directory exists
    if [ ! -d "$commands_dir" ]; then
        echo "Creating Claude Code commands directory: $commands_dir"
        mkdir -p "$commands_dir"
    fi

    local symlinks_created=0

    # Create sdd-* symlinks
    for skill_dir in "$skills_dir"/sdd-*/; do
        if [ -d "$skill_dir" ] && [ -f "${skill_dir}SKILL.md" ]; then
            local skill_name=$(basename "$skill_dir")
            local target="$commands_dir/$skill_name"
            local source="${skill_dir}SKILL.md"

            if [ -L "$target" ]; then
                rm "$target"
                echo "  Updated: $target -> $source"
            elif [ -e "$target" ]; then
                echo "  Skipped: $target (exists and is not a symlink)"
                continue
            else
                echo "  Created: $target -> $source"
            fi
            ln -s "$source" "$target"
            ((symlinks_created++))
        fi
    done

    # Create pkm-* symlinks (if any exist)
    for skill_dir in "$skills_dir"/pkm-*/; do
        if [ -d "$skill_dir" ] && [ -f "${skill_dir}SKILL.md" ]; then
            local skill_name=$(basename "$skill_dir")
            local target="$commands_dir/$skill_name"
            local source="${skill_dir}SKILL.md"

            if [ -L "$target" ]; then
                rm "$target"
                echo "  Updated: $target -> $source"
            elif [ -e "$target" ]; then
                echo "  Skipped: $target (exists and is not a symlink)"
                continue
            else
                echo "  Created: $target -> $source"
            fi
            ln -s "$source" "$target"
            ((symlinks_created++))
        fi
    done

    # Create root-level skill symlinks (branch-pr, issue-creation, etc.)
    for skill_dir in "$skills_dir"/*/; do
        if [ -d "$skill_dir" ] && [ -f "${skill_dir}SKILL.md" ]; then
            local skill_name=$(basename "$skill_dir")
            # Skip if already handled by sdd-* or pkm-* patterns
            if [[ "$skill_name" == sdd-* ]] || [[ "$skill_name" == pkm-* ]]; then
                continue
            fi
            local target="$commands_dir/$skill_name"
            local source="${skill_dir}SKILL.md"

            if [ -L "$target" ]; then
                rm "$target"
                echo "  Updated: $target -> $source"
            elif [ -e "$target" ]; then
                echo "  Skipped: $target (exists and is not a symlink)"
                continue
            else
                echo "  Created: $target -> $source"
            fi
            ln -s "$source" "$target"
            ((symlinks_created++))
        fi
    done

    echo ""
    echo "Created $symlinks_created Claude Code command symlinks."
    echo "Available commands in Claude Code: /sdd-*, /branch-pr, /issue-creation, /skill-registry"
}

# Ask user about creating symlinks
ask_create_symlinks() {
    if is_claude_code_installed; then
        echo ""
        echo "=========================================="
        echo "Claude Code detected!"
        echo "=========================================="
        echo ""
        echo "Would you like to create command symlinks for Claude Code?"
        echo "This will make PKM-AI skills available as /commands in Claude Code."
        echo ""
        echo "Symlinks will be created in: $HOME/.claude/commands/"
        echo ""

        while true; do
            read -p "Create Claude Code command symlinks? [Y/n] " -n 1 -r yn
            echo
            case $yn in
                [Yy]|"" )
                    create_claude_code_symlinks
                    break
                    ;;
                [Nn] )
                    echo "Skipping Claude Code symlink creation."
                    echo "You can run '~/.pkm-ai/scripts/install.sh -l' later to create them."
                    break
                    ;;
                * )
                    echo "Please answer yes or no."
                    ;;
            esac
        done
    else
        echo ""
        echo "Claude Code not detected. Skipping symlink creation."
        echo "If you install Claude Code later, run '~/.pkm-ai/scripts/install.sh -l' to create command symlinks."
    fi
}

# Parse arguments
while getopts "d:b:flh" opt; do
    case $opt in
        d) PKM_AI_DIR="$OPTARG";;
        b) BRANCH="$OPTARG";;
        f) FORCE_UPDATE="true";;
        l) CREATE_SYMLINKS_ONLY="true";;
        h) usage; exit 0;;
        \?) usage; exit 1;;
    esac
done

# Resolve ~ to absolute path
PKM_AI_DIR="${PKM_AI_DIR/#\~/$HOME}"
PKM_AI_DIR="$(realpath "$PKM_AI_DIR" 2>/dev/null || echo "$PKM_AI_DIR")"

# Symlinks-only mode
if [ "$CREATE_SYMLINKS_ONLY" = "true" ]; then
    if [ ! -d "$PKM_AI_DIR" ]; then
        echo "Error: PKM-AI directory not found at $PKM_AI_DIR"
        echo "Please install PKM-AI first, or specify the correct directory with -d"
        exit 1
    fi

    if [ ! -d "$PKM_AI_DIR/workflows/pkmai/skills" ]; then
        echo "Error: Skills directory not found at $PKM_AI_DIR/workflows/pkmai/skills"
        echo "The installation may be corrupted."
        exit 1
    fi

    create_claude_code_symlinks
    exit 0
fi

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
echo "Documentation: $PKM_AI_DIR/workflows/pkmai/docs/"

# Offer to create Claude Code symlinks
ask_create_symlinks