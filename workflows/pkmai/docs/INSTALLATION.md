# PKM-AI Installation Guide

## Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/pkm-ai/pkm-ai/main/scripts/install.sh | bash
```

This installs PKM-AI to `~/.pkm-ai`. After installation, source the environment:

```bash
source ~/.pkm-ai/.envrc
```

## Installation Options

### Default Installation (~/.pkm-ai)

```bash
curl -fsSL https://raw.githubusercontent.com/pkm-ai/pkm-ai/main/scripts/install.sh | bash
```

### Custom Directory

```bash
curl -fsSL https://raw.githubusercontent.com/pkm-ai/pkm-ai/main/scripts/install.sh | bash -s -- -d /path/to/pkm-ai
```

### Update Existing Installation

```bash
~/.pkm-ai/scripts/update.sh
```

## Manual Installation

### Git Clone

```bash
# Clone to default location
git clone https://github.com/pkm-ai/pkm-ai.git ~/.pkm-ai

# Or to custom location
git clone https://github.com/pkm-ai/pkm-ai.git /path/to/pkm-ai
```

### Git Submodule (for projects)

```bash
# In your project root
git submodule add https://github.com/pkm-ai/pkm-ai.git .pkm-ai/skills

# Initialize submodules
git submodule update --init --recursive
```

## Post-Installation

### 1. Source Environment

Add to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
source ~/.pkm-ai/.envrc
```

### 2. Verify Installation

```bash
~/.pkm-ai/scripts/list-skills.sh
```

### 3. Environment Variables

After sourcing `.envrc`, these variables are available:

| Variable | Description |
|----------|-------------|
| `PKM_AI_DIR` | Installation directory |
| `PKM_AI_SHARED` | Shared conventions path |
| `PKM_AI_SKILLS` | Skills directory |

## Installation Methods Comparison

| Method | Command | Best For |
|--------|---------|----------|
| **Quick install** | `curl ... \| bash` | One-time install |
| **Git clone** | `git clone ... ~/.pkm-ai` | Manual control |
| **Submodule** | `git submodule add ...` | Project integration |
| **Update** | `~/.pkm-ai/scripts/update.sh` | Keep current |

## Uninstallation

```bash
# Remove installation
rm -rf ~/.pkm-ai

# Remove from shell profile (edit manually)
# Remove the "source ~/.pkm-ai/.envrc" line
```

## Troubleshooting

### Permission Denied

```bash
# If install fails with permission errors
mkdir -p ~/.pkm-ai
chmod 755 ~/.pkm-ai
```

### Git Not Found

PKM-AI requires Git for installation. Install Git first:

```bash
# Ubuntu/Debian
sudo apt install git

# macOS
brew install git
```

### SSL Errors

If you get SSL errors with curl:

```bash
# Use git clone instead
git clone https://github.com/pkm-ai/pkm-ai.git ~/.pkm-ai
```
