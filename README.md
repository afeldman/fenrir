<p align="center">
  <img src="docs/logo.png" alt="cloudlogin" width="320">
</p>

# Fenrir Browser

**A privacy-focused, EU-first web browser built on Servo with AI integration**

Fenrir is a modern web browser designed with privacy, security, and European values at its core. Built on the Servo browser engine with Rust, it offers a fast, secure browsing experience with integrated AI capabilities through MCP (Model Context Protocol).

## Features

### 🛡️ Privacy & Security
- **EU-First Design**: Built with European data protection standards in mind
- **Privacy by Default**: Minimal data collection, no telemetry
- **Secure Architecture**: Rust-based memory safety
- **Modern TLS**: rustls with AWS-LC-RS crypto provider

### 🚀 Performance
- **Servo Engine**: Next-generation browser engine written in Rust
- **Parallel Rendering**: Multi-core optimized layout and rendering
- **Hardware Acceleration**: GPU-accelerated graphics pipeline

### 🤖 AI Integration
- **Local AI Models**: Run models locally with Candle ML framework
- **MCP Support**: Model Context Protocol for AI tool integration
- **CPU/GPU/Metal**: Multiple acceleration backends
- **HuggingFace Integration**: Easy model downloading and management

### 🎨 Modern UI
- **egui Framework**: Immediate-mode GUI for responsive interface
- **Custom Toolbar**: Privacy-focused navigation controls
- **Resource Management**: Efficient handling of browser resources

## Architecture

Fenrir follows a modular architecture with these core components:

### Crates
- **`fenrir-core`**: Core types, traits, and shared utilities
- **`fenrir-servo`**: Servo browser engine integration and rendering
- **`fenrir-network`**: Network stack and bandwidth management
- **`fenrir-mcp`**: Model Context Protocol implementation
- **`fenrir-ai`**: Local AI model inference and management
- **`fenrir-app`**: Main application with UI and window management

### Dependencies
- **Servo**: Browser engine (git dependency, tag v0.0.5)
- **egui/winit**: UI framework and window management
- **tokio**: Async runtime
- **rustls**: TLS implementation
- **Candle**: HuggingFace Rust ML framework
- **rmcp**: Official MCP Rust SDK

## Getting Started

### Prerequisites

- **Rust**: 1.86.0 or later (specified in `rust-toolchain.toml`)
- **Cargo**: Rust package manager
- **System Dependencies**:
  - macOS: Xcode Command Line Tools
  - Linux: Development libraries (gcc, pkg-config, etc.)
  - Windows: Visual Studio Build Tools

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd fenrir
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run the browser**:
   ```bash
   cargo run --release
   ```

### Development

For development builds with debug symbols:
```bash
cargo build
cargo run
```

## Project Structure

```
fenrir/
├── Cargo.toml              # Workspace configuration
├── Cargo.lock              # Dependency lock file
├── rust-toolchain.toml     # Rust version specification
├── fenrir-app/             # Main application binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Entry point
│       ├── app.rs          # Browser application
│       ├── browser.rs      # Browser core logic
│       ├── input.rs        # Input handling
│       ├── resources.rs    # Resource management
│       ├── toolbar.rs      # UI toolbar
│       └── waker.rs        # Async waker implementation
├── crates/                 # Library crates
│   ├── fenrir-core/        # Core types and utilities
│   ├── fenrir-servo/       # Servo integration
│   ├── fenrir-network/     # Network stack
│   ├── fenrir-mcp/         # MCP protocol
│   └── fenrir-ai/          # AI capabilities
└── resources/              # Browser resources
    ├── about-memory.html
    ├── badcert.html
    ├── crash.html
    ├── debugger.js
    └── ... (other HTML/CSS/JS resources)
```

## AI Features

### Local Model Inference
Fenrir supports running AI models locally using the Candle framework:

```rust
// Example: Load and run a local model
use fenrir_ai::ModelRunner;

let runner = ModelRunner::new("model-path");
let result = runner.infer("Your prompt here").await?;
```

### MCP Integration
Connect to AI services through the Model Context Protocol:

```rust
use fenrir_mcp::MCPClient;

let client = MCPClient::connect("provider-url").await?;
let response = client.query("Your question").await?;
```

### Supported Backends
- **CPU**: Fallback for all systems
- **CUDA**: NVIDIA GPU acceleration
- **Metal**: Apple Silicon/GPU acceleration

## Privacy Features

### Data Protection
- No telemetry or usage data collection
- Local AI processing (optional cloud services)
- EU GDPR compliance considerations

### Security
- Memory-safe Rust implementation
- Modern TLS with rustls
- Regular dependency updates

## Contributing

We welcome contributions! Please see our contributing guidelines for details.

### Development Workflow
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

### Code Style
- Follow Rust formatting: `cargo fmt`
- Run clippy checks: `cargo clippy`
- Maintain documentation

## License

This project is licensed under the **EUPL-1.2** (European Union Public License).

See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Servo Project**: For the amazing browser engine
- **Rust Community**: For the excellent ecosystem
- **egui**: For the immediate-mode GUI framework
- **HuggingFace**: For the Candle ML framework

## Roadmap

### Short-term
- [ ] Basic browsing functionality
- [ ] Tab management
- [ ] Bookmark system
- [ ] Privacy settings UI

### Medium-term
- [ ] Extension support
- [ ] Password manager integration
- [ ] Enhanced AI features
- [ ] Sync capabilities

### Long-term
- [ ] Mobile versions
- [ ] Advanced privacy tools
- [ ] Decentralized features
- [ ] Plugin ecosystem

## Support

For issues, questions, or discussions:
- Check existing issues on GitHub
- Create a new issue for bugs
- Join our community discussions

---

## 📞 Support

- **Documentation**: [docs.fenrir.ai](https://github.com/afeldman/fenrir)
- **Issues**: [GitHub Issues](https://github.com/afeldman/fenrir/issues)
- **Discussions**: [GitHub Discussions](https://github.com/afeldman/fenrir/discussions)
- **Email**: anton.feldmann@gmail.com

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=afeldman/fenrir&type=Date)](https://star-history.com/#afeldman/fenrir&Date)

---

**Fenrir Browser** – Browsing with privacy, powered by Rust and AI.
