# Installation Guide

This guide covers how to install Fenrir Browser on different platforms and configurations.

## System Requirements

### Minimum Requirements
- **CPU**: x86_64 or ARM64 processor
- **RAM**: 4 GB minimum, 8 GB recommended
- **Storage**: 2 GB free disk space
- **OS**: See platform-specific requirements below

### Recommended Requirements
- **CPU**: Multi-core processor (4+ cores)
- **RAM**: 16 GB for optimal performance with AI features
- **Storage**: 10 GB for models and cache
- **GPU**: Dedicated GPU for hardware acceleration (optional but recommended)

## Platform-Specific Installation

### macOS

#### Prerequisites
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Method 1: From Source (Recommended)
```bash
# Clone the repository
git clone https://github.com/your-org/fenrir.git
cd fenrir

# Build in release mode
cargo build --release

# Run Fenrir
./target/release/fenrir
```

#### Method 2: Using Cargo Install
```bash
# Install directly via Cargo (when available)
cargo install fenrir-browser

# Run
fenrir
```

#### Method 3: Homebrew (Future)
```bash
# Once available in Homebrew
brew install fenrir-browser
```

### Linux

#### Prerequisites (Ubuntu/Debian)
```bash
# Install build dependencies
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    libssl-dev \
    pkg-config \
    libglib2.0-dev \
    libgtk-3-dev \
    libx11-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev
```

#### Prerequisites (Fedora/RHEL)
```bash
# Install build dependencies
sudo dnf install -y \
    gcc \
    gcc-c++ \
    curl \
    openssl-devel \
    pkgconfig \
    glib2-devel \
    gtk3-devel \
    libxcb-devel
```

#### Installation
```bash
# Clone and build
git clone https://github.com/your-org/fenrir.git
cd fenrir
cargo build --release

# Optional: Install to system
sudo cp target/release/fenrir /usr/local/bin/
```

### Windows

#### Prerequisites
1. Install [Rust](https://rustup.rs/) using rustup-init.exe
2. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
   - Select "Desktop development with C++" workload
3. Install [LLVM](https://releases.llvm.org/download.html) (optional, for LTO)

#### Installation
```powershell
# Clone the repository
git clone https://github.com/your-org/fenrir.git
cd fenrir

# Build
cargo build --release

# Run
.\target\release\fenrir.exe
```

## Docker Installation

### Quick Start with Docker
```bash
# Pull the Docker image (when available)
docker pull ghcr.io/your-org/fenrir:latest

# Run Fenrir in Docker
docker run -it \
  --rm \
  -e DISPLAY=$DISPLAY \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  ghcr.io/your-org/fenrir:latest
```

### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'
services:
  fenrir:
    image: ghcr.io/your-org/fenrir:latest
    environment:
      - DISPLAY=${DISPLAY}
    volumes:
      - /tmp/.X11-unix:/tmp/.X11-unix
      - ./data:/data
    devices:
      - /dev/dri:/dev/dri  # GPU acceleration
```

## Building from Source

### Clone the Repository
```bash
git clone https://github.com/your-org/fenrir.git
cd fenrir
```

### Update Submodules (if any)
```bash
git submodule update --init --recursive
```

### Build Options

#### Development Build (Debug Symbols)
```bash
cargo build
```

#### Release Build (Optimized)
```bash
cargo build --release
```

#### Specific Feature Builds
```bash
# Build with AI features
cargo build --release --features "ai"

# Build without AI features
cargo build --release --no-default-features

# Build with specific backend
cargo build --release --features "cuda"  # NVIDIA CUDA
cargo build --release --features "metal" # Apple Metal
```

### Install System-Wide
```bash
# Install to cargo bin directory
cargo install --path .

# Or copy manually
sudo cp target/release/fenrir /usr/local/bin/
```

## Post-Installation Setup

### First Run Configuration
1. **Launch Fenrir** for the first time
2. **Privacy Settings**: Review and configure privacy options
3. **AI Setup**: Configure local AI models if desired
4. **Import Data**: Import bookmarks/history from other browsers (optional)

### Configuration Files
Fenrir stores configuration in platform-specific locations:

- **Linux**: `~/.config/fenrir/`
- **macOS**: `~/Library/Application Support/Fenrir/`
- **Windows**: `%APPDATA%\Fenrir\`

### Environment Variables
```bash
# Set Rust logging level
export RUST_LOG=info

# Enable backtraces on panic
export RUST_BACKTRACE=1

# Set custom config directory
export FENRIR_CONFIG_DIR=~/.config/my-fenrir

# Set cache directory
export FENRIR_CACHE_DIR=~/.cache/fenrir
```

## Verifying Installation

### Check Version
```bash
fenrir --version
```

### Verify Dependencies
```bash
# Check Rust version
rustc --version

# Check Cargo version
cargo --version

# Check build tools
make --version  # or equivalent
```

### Run Tests
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration

# Run specific test suite
cargo test --package fenrir-core
```

## Troubleshooting Installation

### Common Issues

#### Build Failures
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustup update
```

#### Missing Dependencies
```bash
# On Ubuntu/Debian
sudo apt install -y libssl-dev pkg-config

# On macOS
brew install openssl pkg-config
```

#### Permission Issues
```bash
# Fix cargo permissions
sudo chown -R $(whoami) ~/.cargo

# Fix installation directory permissions
sudo chmod +x /usr/local/bin/fenrir
```

#### GPU Acceleration Issues
```bash
# Check GPU availability
nvidia-smi  # NVIDIA
rocminfo    # AMD
```

### Getting Help
1. Check the [Troubleshooting Guide](../troubleshooting/common.md)
2. Search [GitHub Issues](https://github.com/your-org/fenrir/issues)
3. Ask in community channels

## Updating Fenrir

### From Source
```bash
# Pull latest changes
git pull origin main

# Rebuild
cargo build --release

# Reinstall if needed
cargo install --path . --force
```

### Using Package Manager
```bash
# Update via Cargo
cargo install fenrir-browser --force

# Update via Homebrew (when available)
brew upgrade fenrir-browser
```

### Docker Updates
```bash
# Pull latest image
docker pull ghcr.io/your-org/fenrir:latest

# Restart container
docker-compose down && docker-compose up -d
```

## Uninstallation

### Remove from System
```bash
# Remove binary
sudo rm /usr/local/bin/fenrir

# Remove configuration
rm -rf ~/.config/fenrir/
rm -rf ~/.cache/fenrir/

# On macOS
rm -rf ~/Library/Application\ Support/Fenrir/

# On Windows (PowerShell)
Remove-Item -Path "$env:APPDATA\Fenrir" -Recurse -Force
```

### Clean Cargo Installation
```bash
# Uninstall via Cargo
cargo uninstall fenrir-browser

# Clean build artifacts
cd /path/to/fenrir
cargo clean
```

## Next Steps

After successful installation:

1. **Configure Privacy Settings**: Review [Privacy Configuration](../privacy/design.md)
2. **Set Up AI Features**: See [AI Integration Guide](../ai/integration.md)
3. **Customize UI**: Check [UI Customization](../ui/theming.md)
4. **Explore Features**: Try the [Quick Start Guide](quick-start.md)

---

*Previous: [Overview](overview.md) | Next: [Quick Start Guide](quick-start.md)*
