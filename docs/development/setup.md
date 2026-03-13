# Development Setup

This guide covers setting up a development environment for Fenrir Browser, including tools, dependencies, and workflow.

## Prerequisites

### Required Tools

#### 1. Rust Toolchain
```bash
# Install rustup if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install specific Rust version (from rust-toolchain.toml)
rustup install 1.86.0
rustup default 1.86.0

# Verify installation
rustc --version
cargo --version
```

#### 2. Build Essentials
**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew packages
brew install cmake pkg-config
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    libglib2.0-dev \
    libgtk-3-dev \
    libx11-dev \
    libxcb1-dev \
    libxcb-render0-dev \
    libxcb-shape0-dev \
    libxcb-xfixes0-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install -y \
    gcc \
    gcc-c++ \
    cmake \
    pkgconfig \
    openssl-devel \
    glib2-devel \
    gtk3-devel \
    libxcb-devel
```

**Windows:**
1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. Select "Desktop development with C++" workload
3. Install [LLVM](https://releases.llvm.org/download.html) (optional)

### 3. Git
```bash
# Install Git
# macOS: brew install git
# Ubuntu: sudo apt install git
# Windows: https://git-scm.com/download/win

# Configure Git
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

## Project Setup

### Clone the Repository
```bash
# Clone the main repository
git clone https://github.com/your-org/fenrir.git
cd fenrir

# Initialize submodules (if any)
git submodule update --init --recursive
```

### Verify Project Structure
```bash
# Check project structure
tree -L 2

# Expected structure:
# fenrir/
# ├── Cargo.toml
# ├── Cargo.lock
# ├── rust-toolchain.toml
# ├── fenrir-app/
# ├── crates/
# │   ├── fenrir-core/
# │   ├── fenrir-servo/
# │   ├── fenrir-network/
# │   ├── fenrir-mcp/
# │   └── fenrir-ai/
# └── resources/
```

### Build Dependencies
```bash
# Build all dependencies (first build may take a while)
cargo build

# Or build specific crate
cargo build -p fenrir-core
cargo build -p fenrir-app
```

## Development Tools

### Recommended IDE/Editor

#### Visual Studio Code
```bash
# Install VS Code extensions
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension bungcip.better-toml
code --install-extension vadimcn.vscode-lldb
```

#### IntelliJ IDEA / CLion
- Install Rust plugin
- Configure Rust toolchain
- Enable Cargo features

### Essential Development Tools

#### 1. Rust Analyzer
```bash
# Ensure rust-analyzer is installed
rustup component add rust-analyzer
```

#### 2. Cargo Watch
```bash
# Install cargo-watch for automatic rebuilding
cargo install cargo-watch
```

#### 3. Cargo Edit
```bash
# Install cargo-edit for dependency management
cargo install cargo-edit
```

#### 4. Cargo Nextest
```bash
# Install nextest for better test runner
cargo install cargo-nextest
```

#### 5. Cargo Audit
```bash
# Install cargo-audit for security audits
cargo install cargo-audit
```

## Development Workflow

### Building the Project

#### Development Build
```bash
# Build with debug symbols (faster compile)
cargo build

# Build specific crate
cargo build -p fenrir-app

# Build with specific features
cargo build --features "ai"
```

#### Release Build
```bash
# Build optimized version
cargo build --release

# Build with LTO (slower compile, faster runtime)
cargo build --release --config 'profile.release.lto="fat"'
```

#### Clean Build
```bash
# Clean all build artifacts
cargo clean

# Clean specific crate
cargo clean -p fenrir-core
```

### Running the Browser

#### Development Mode
```bash
# Run with debug symbols
cargo run

# Run with specific arguments
cargo run -- https://example.com

# Run with environment variables
RUST_LOG=debug cargo run
```

#### Release Mode
```bash
# Run optimized version
cargo run --release

# Run from built binary
./target/release/fenrir
```

### Testing

#### Run All Tests
```bash
# Run all tests
cargo test

# Run tests with nextest (faster)
cargo nextest run

# Run tests with specific filter
cargo test test_network
```

#### Test Specific Crate
```bash
# Test core crate
cargo test -p fenrir-core

# Test with verbose output
cargo test -p fenrir-core -- --nocapture
```

#### Integration Tests
```bash
# Run integration tests
cargo test --test integration

# Run UI tests
cargo test --test ui
```

### Code Quality

#### Formatting
```bash
# Format all code
cargo fmt

# Check formatting without applying
cargo fmt -- --check
```

#### Linting
```bash
# Run clippy linter
cargo clippy

# Run clippy with all warnings
cargo clippy -- -W clippy::pedantic -W clippy::nursery

# Fix clippy suggestions
cargo clippy --fix
```

#### Security Audit
```bash
# Check for vulnerable dependencies
cargo audit

# Update dependencies
cargo update
```

## Debugging

### Logging Configuration
```bash
# Set log level via environment variable
export RUST_LOG=debug

# More specific logging
export RUST_LOG=fenrir_core=debug,fenrir_network=info

# Log to file
export RUST_LOG=debug
export RUST_LOG_FILE=/tmp/fenrir.log
```

### Debug Build with Symbols
```bash
# Build with debug symbols
cargo build

# Run with debugger
lldb target/debug/fenrir
```

### Profiling

#### CPU Profiling
```bash
# Install profiling tools
# macOS: Instruments (Xcode)
# Linux: perf, valgrind
# Windows: Windows Performance Toolkit

# Example with perf (Linux)
perf record ./target/release/fenrir
perf report
```

#### Memory Profiling
```bash
# Install memory profiler
cargo install cargo-valgrind

# Run with valgrind
cargo valgrind run
```

## Development Environment Configuration

### Editor Configuration

#### VS Code Settings
```json
{
    "rust-analyzer.check.command": "clippy",
    "rust-analyzer.cargo.features": ["ai"],
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": "explicit"
    }
}
```

#### Git Hooks
```bash
# Install pre-commit hook
cp .githooks/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit

# Or use cargo-husky
cargo install cargo-husky
cargo husky install
```

### Environment Variables

#### Development-Specific Variables
```bash
# Add to ~/.bashrc or ~/.zshrc
export FENRIR_DEV_MODE=1
export FENRIR_LOG_LEVEL=debug
export RUST_BACKTRACE=1

# For Servo development
export SERVO_DEBUG=1
```

#### Cargo Configuration
```toml
# ~/.cargo/config.toml
[build]
rustflags = ["-C", "target-cpu=native"]

[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

## Working with Dependencies

### Adding Dependencies
```bash
# Add a new dependency
cargo add serde

# Add with specific features
cargo add tokio --features full

# Add development dependency
cargo add --dev mockito
```

### Updating Dependencies
```bash
# Update all dependencies
cargo update

# Update specific crate
cargo update -p tokio

# Check for outdated crates
cargo outdated
```

### Vendor Dependencies
```bash
# Vendor dependencies (for offline development)
cargo vendor

# Use vendored dependencies
cargo build --offline
```

## Working with Servo

### Servo Dependencies
```bash
# Servo is included as git dependency
# Check current version in Cargo.toml
grep servo Cargo.toml

# Update Servo version
# 1. Update tag in Cargo.toml
# 2. Run cargo update
cargo update -p servo
```

### Servo Development Notes
- Servo requires specific Rust version (check rust-toolchain.toml)
- Large dependency tree - first build takes time
- May require additional system dependencies

## Continuous Integration

### Local CI Simulation
```bash
# Run tests like CI
cargo test --all-features

# Run clippy like CI
cargo clippy --all-features -- -D warnings

# Run formatting check
cargo fmt -- --check
```

### GitHub Actions
```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: 1.86.0
      - run: cargo test --all-features
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo fmt -- --check
```

## Troubleshooting Development Issues

### Common Problems

#### Build Failures
```bash
# Clean and rebuild
cargo clean
cargo build

# Check Rust version
rustup update

# Check system dependencies
pkg-config --version
cmake --version
```

#### Linking Errors
```bash
# Check linker configuration
echo $LD_LIBRARY_PATH

# On macOS, check Xcode
xcode-select -p

# On Windows, check Visual Studio
where cl.exe
```

#### Test Failures
```bash
# Run single test
cargo test test_name -- --nocapture

# Run with more output
cargo test -- --nocapture

# Check test environment
env | grep FENRIR
```

### Getting Help

#### Debug Information
```bash
# Collect debug info
cargo version
rustc --version --verbose
uname -a

# Check system info
pkg-config --list-all | head -20
```

#### Ask for Help
1. Check existing GitHub issues
2. Search project documentation
3. Ask in community channels
4. Create minimal reproduction case

## Next Steps

After setting up your development environment:

1. **Explore the codebase**: Read [Architecture Overview](../architecture/overview.md)
2. **Run examples**: Try the example applications
3. **Pick an issue**: Start with "good first issue" labeled tasks
4. **Write tests**: Add tests for new features
5. **Submit PR**: Follow [Contributing Guide](../contributing/guide.md)

## Useful Commands Reference

### Development Commands
```bash
# Build and run
cargo run

# Watch for changes and rebuild
cargo watch -x run

# Run tests continuously
cargo watch -x test

# Check for warnings
cargo check

# Generate documentation
cargo doc --open
```

### Debugging Commands
```bash
# Run with debugger
cargo run -- --debug

# Profile CPU usage
cargo flamegraph

# Check memory usage
cargo +nightly valgrind --tool=memcheck
```

### Maintenance Commands
```bash
# Update all dependencies
cargo update

# Audit security
cargo audit

# Check license compliance
cargo deny check

# Generate dependency graph
cargo depgraph | dot -Tpng > deps.png
```

---

*Next: [Building from Source](building.md)*
