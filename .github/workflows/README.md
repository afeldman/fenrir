# GitHub Actions Workflows

This repository uses GitHub Actions for continuous integration and deployment. The following workflows are configured:

## Workflows

### 1. Release Build (`release.yml`)
**Trigger:** When a tag matching `vX.Y.Z` is pushed (e.g., `v1.0.0`, `v2.3.1`)

**Jobs:**
- Builds release binaries for:
  - Linux (x86_64-unknown-linux-gnu)
  - Windows (x86_64-pc-windows-msvc)
  - macOS (x86_64-apple-darwin)
- Creates compressed archives for each platform
- Creates a GitHub Release with all artifacts
- Includes release notes template

### 2. Develop Branch CI (`develop.yml`)
**Trigger:** On push to `develop` branch or pull requests targeting `develop`

**Jobs (sequential):**
1. **Linting:** Runs `cargo fmt` and `cargo clippy`
2. **Tests:** Runs all tests including integration tests
3. **Build Check:** Builds in debug and release modes

### 3. Main Branch CI (`main.yml`)
**Trigger:** On push to `main` branch or pull requests targeting `main`

**Jobs (sequential):**
1. **Linting:** Runs `cargo fmt` and `cargo clippy`
2. **Tests:** Runs unit, integration, and documentation tests
3. **Documentation:** Builds Rust documentation and checks links
4. **Build Check:** Builds in debug/release modes and checks for unused dependencies

## Usage

### Creating a Release
1. Update version in `Cargo.toml` (if needed)
2. Create and push a tag:
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```
3. The workflow will automatically:
   - Build binaries for all platforms
   - Create a GitHub Release
   - Upload artifacts

### Development Workflow
- **Feature branches:** Create PRs to `develop` branch
- **Release preparation:** Merge `develop` into `main` when ready
- **Hotfixes:** Create PRs directly to `main` branch

## Requirements

### System Dependencies
The workflows install the following system dependencies on Ubuntu:
- `pkg-config`
- `libssl-dev`
- `libx11-dev`
- `libxcb1-dev`
- `libxcb-render0-dev`
- `libxcb-shape0-dev`
- `libxcb-xfixes0-dev`

### Rust Toolchain
- Stable Rust toolchain
- Components: `rustfmt`, `clippy`
- Target platforms: Linux, Windows, macOS

## Cache Configuration
The workflows use GitHub Actions cache to speed up builds by caching:
- Cargo registry
- Cargo git dependencies
- Build target directory

## Artifacts
- **Release workflow:** Uploads platform-specific binaries as artifacts
- **Main workflow:** Uploads generated documentation as artifact

## Customization

### Adding New Platforms
Edit `release.yml` and add to the `matrix.target` array.

### Adding New Checks
Edit the respective workflow files to add new steps (e.g., security scanning, code coverage).

### Modifying Dependencies
Update the system dependencies installation step in each workflow if additional dependencies are needed.
