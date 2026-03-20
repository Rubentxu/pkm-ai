#!/bin/bash
# PKM-AI List Skills Script
# Lists all available PKM-AI skills

shopt -s extglob

PKM_AI_DIR="${PKM_AI_DIR:-$HOME/.pkm-ai}"

# Resolve ~ to absolute path
PKM_AI_DIR="${PKM_AI_DIR/#\~/$HOME}"
PKM_AI_DIR="$(realpath "$PKM_AI_DIR" 2>/dev/null || echo "$PKM_AI_DIR")"

if [ ! -d "$PKM_AI_DIR" ]; then
    echo "PKM-AI not installed at $PKM_AI_DIR"
    echo "Run install.sh first to install PKM-AI."
    exit 1
fi

# Get version
VERSION=$(cd "$PKM_AI_DIR" && git describe --tags 2>/dev/null || echo "unknown")

echo ""
echo "=========================================="
echo "PKM-AI Skills v$VERSION"
echo "=========================================="
echo ""
echo "Location: $PKM_AI_DIR"
echo ""

echo "=== SDD Phase Skills (9) ==="
echo ""

for skill in "$PKM_AI_DIR/skills"/sdd-*; do
    if [ -d "$skill" ] && [ -f "$skill/SKILL.md" ]; then
        name=$(basename "$skill")
        # Extract description from SKILL.md
        desc=$(grep -A2 "^description:" "$skill/SKILL.md" 2>/dev/null | tail -1 | sed 's/.*:.*>//' | tr -d '<' | cut -c1-60)
        if [ -z "$desc" ]; then
            desc="No description"
        fi
        printf "  %-20s %s\n" "$name" "$desc"
    fi
done

echo ""
echo "=== Utility Skills (3) ==="
echo ""

for skill in "$PKM_AI_DIR/skills"/!(sdd-*); do
    if [ -d "$skill" ] && [ -f "$skill/SKILL.md" ]; then
        name=$(basename "$skill")
        desc=$(grep -A2 "^description:" "$skill/SKILL.md" 2>/dev/null | tail -1 | sed 's/.*:.*>//' | tr -d '<' | cut -c1-60)
        if [ -z "$desc" ]; then
            desc="No description"
        fi
        printf "  %-20s %s\n" "$name" "$desc"
    fi
done

echo ""
echo "=========================================="
echo ""
echo "For details on a skill, read:"
echo "  $PKM_AI_DIR/skills/{skill-name}/SKILL.md"
echo ""
echo "Documentation: $PKM_AI_DIR/docs/"
