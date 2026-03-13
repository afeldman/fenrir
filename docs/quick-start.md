# Quick Start Guide

Get up and running with Fenrir Browser in minutes. This guide covers the essential steps to start using Fenrir effectively.

## Installation Quick Start

### macOS/Linux
```bash
# Clone and build
git clone https://github.com/your-org/fenrir.git
cd fenrir
cargo build --release

# Run
./target/release/fenrir
```

### Windows
```powershell
# Clone and build
git clone https://github.com/your-org/fenrir.git
cd fenrir
cargo build --release

# Run
.\target\release\fenrir.exe
```

## First Launch

### Welcome Screen
When you first launch Fenrir, you'll see:
1. **Welcome message** with privacy overview
2. **Initial setup wizard** (optional)
3. **Privacy settings** configuration

### Skip Setup (Recommended for First-Time Users)
Click "Skip Setup" to start with default settings, which are privacy-focused.

## Basic Navigation

### Toolbar Overview
```
[Back] [Forward] [Refresh] [Home] [URL Bar] [Bookmarks] [Settings] [AI Tools]
```

### Essential Shortcuts
| Action | Shortcut |
|--------|----------|
| New Tab | `Cmd/Ctrl + T` |
| Close Tab | `Cmd/Ctrl + W` |
| Next Tab | `Cmd/Ctrl + Tab` |
| Previous Tab | `Cmd/Ctrl + Shift + Tab` |
| Refresh | `Cmd/Ctrl + R` |
| Find | `Cmd/Ctrl + F` |
| Developer Tools | `Cmd/Ctrl + Shift + I` |
| Privacy Mode | `Cmd/Ctrl + Shift + P` |

### URL Navigation
1. Click the URL bar or press `Cmd/Ctrl + L`
2. Type your search or URL
3. Press Enter to navigate

## Privacy Features Quick Setup

### Enable Essential Privacy
1. Open **Settings** → **Privacy & Security**
2. Enable:
   - **Tracker Blocking** (Recommended)
   - **Enhanced Privacy Mode** (Optional)
   - **Local AI Processing** (If using AI features)

### Configure Search Engine
1. Go to **Settings** → **Search**
2. Select a privacy-focused search engine:
   - DuckDuckGo (Default)
   - StartPage
   - Searx (Self-hosted option)

## AI Features Quick Start

### Enable Local AI
1. Open **Settings** → **AI Features**
2. Click "Enable Local AI"
3. Download a model (optional - starts with small default model)

### Basic AI Commands
In the URL bar, type:
- `ai:summarize [URL]` - Summarize a webpage
- `ai:explain [text]` - Explain selected text
- `ai:translate [text]` - Translate text

### MCP Integration
1. Open **Settings** → **AI Features** → **MCP**
2. Add MCP server URL or local provider
3. Test connection with "Test Connection" button

## Essential Configuration

### Import Data (Optional)
```bash
# Fenrir can import from:
# - Firefox
# - Chrome/Chromium
# - Safari
# - Bookmarks HTML file
```

Steps:
1. **Settings** → **Import & Export**
2. Select browser/data type
3. Choose what to import (bookmarks, history, settings)
4. Click "Import"

### Set Homepage
1. **Settings** → **General**
2. Under "Homepage", enter your preferred URL
3. Or select "New Tab Page" for blank start

### Configure Downloads
1. **Settings** → **Downloads**
2. Choose download location
3. Enable/disable download prompts

## Common Tasks

### Bookmark a Page
1. Click the star icon in URL bar
2. Choose folder (optional)
3. Click "Save"

### Manage Tabs
- **Pin Tab**: Right-click tab → "Pin Tab"
- **Duplicate Tab**: Right-click tab → "Duplicate"
- **Move Tab**: Drag tab to new position
- **Close Other Tabs**: Right-click tab → "Close Other Tabs"

### Privacy Mode
1. Click shield icon in toolbar
2. Or use shortcut `Cmd/Ctrl + Shift + P`
3. Browse privately - no history, cookies, or cache saved

## Troubleshooting Common Issues

### Fenrir Won't Start
```bash
# Check dependencies
cargo version

# Clean build
cargo clean && cargo build --release

# Check logs
export RUST_LOG=debug
./target/release/fenrir
```

### Website Display Issues
1. Try refreshing (`Cmd/Ctrl + R`)
2. Check **Settings** → **Advanced** → "Reset to Defaults"
3. Enable/disable hardware acceleration in **Settings** → **Performance**

### AI Features Not Working
1. Check internet connection (for model downloads)
2. Verify sufficient disk space (2GB+ recommended)
3. Check **Settings** → **AI Features** → "Status"

## Performance Tips

### For Better Performance
1. **Settings** → **Performance**
   - Enable hardware acceleration (if supported)
   - Adjust memory limits if needed
   - Limit background tabs

### For Lower Memory Usage
1. Use fewer extensions/tabs
2. Enable "Suspend background tabs"
3. Clear cache regularly

## Getting Help

### Quick Help Within Fenrir
1. Type `about:help` in URL bar
2. Visit `about:settings` for all settings
3. Check `about:memory` for resource usage

### Online Resources
- [Documentation](https://docs.fenrir-browser.org)
- [GitHub Issues](https://github.com/your-org/fenrir/issues)
- [Community Forum](https://community.fenrir-browser.org)

## Next Steps

### Explore Advanced Features
1. **Extensions**: Check available privacy extensions
2. **Custom CSS**: Modify browser appearance
3. **Advanced Settings**: `about:config` for power users

### Contribute to Fenrir
1. Star the project on GitHub
2. Report bugs or suggest features
3. Consider contributing code or documentation

## Example Workflow

### Daily Browsing Setup
```bash
# 1. Launch Fenrir
./target/release/fenrir

# 2. Set up essential bookmarks
# 3. Configure privacy settings
# 4. Enable local AI for summaries
# 5. Import passwords (if desired)
```

### Developer Setup
```bash
# 1. Enable developer tools
# 2. Set up custom user agent (if needed)
# 3. Configure network interception
# 4. Set up local MCP providers
```

## Command Line Options

### Useful Launch Options
```bash
# Start with specific URL
fenrir https://example.com

# Start in privacy mode
fenrir --private

# Start with specific profile
fenrir --profile "Work"

# Disable GPU acceleration (troubleshooting)
fenrir --disable-gpu

# Enable verbose logging
fenrir --log-level debug
```

### All Available Options
```bash
# View all options
fenrir --help
```

## Migration from Other Browsers

### Quick Migration Guide
1. **Export data** from current browser
2. **Import into Fenrir** (Settings → Import & Export)
3. **Test critical websites**
4. **Configure Fenrir-specific features** (AI, privacy)

### What Transfers Well
- Bookmarks
- History (optional)
- Passwords (via export/import)
- Search engines

### What Might Need Adjustment
- Extensions (Fenrir has different extension system)
- UI preferences
- Advanced settings

---

*Previous: [Installation Guide](installation.md) | Next: [Architecture Overview](../architecture/overview.md)*
