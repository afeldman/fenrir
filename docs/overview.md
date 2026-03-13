# Fenrir Browser Overview

## What is Fenrir Browser?

Fenrir Browser is a privacy-focused, EU-first web browser built on the Servo browser engine with Rust. It combines modern web browsing capabilities with integrated AI features while maintaining strong privacy and security standards.

## Core Philosophy

### 🛡️ Privacy First
- **EU-First Design**: Built with European data protection standards (GDPR) in mind
- **Minimal Data Collection**: No telemetry, no usage tracking by default
- **Local Processing**: AI features can run entirely locally without cloud dependency

### 🚀 Performance & Modernity
- **Rust-Based**: Memory-safe implementation for security and performance
- **Servo Engine**: Next-generation browser engine with parallel rendering
- **Hardware Acceleration**: GPU-accelerated graphics pipeline

### 🤖 AI Integration
- **Local AI Models**: Run models locally using Candle ML framework
- **MCP Support**: Model Context Protocol for extensible AI tool integration
- **Multiple Backends**: CPU, CUDA (NVIDIA), and Metal (Apple) support

## Key Features

### Browsing Experience
- Modern web standards support
- Tabbed browsing interface
- Bookmark management
- Privacy-focused navigation controls
- Customizable toolbar

### Privacy & Security
- Memory-safe architecture (Rust)
- Modern TLS implementation (rustls)
- No telemetry or analytics
- Local AI processing option
- Regular security updates

### AI Capabilities
- Local model inference with Candle
- MCP protocol for AI tool integration
- HuggingFace model integration
- CPU/GPU acceleration options
- Privacy-preserving AI features

### Development Features
- Modular architecture
- Comprehensive API
- Extensible design
- Well-documented codebase
- Active development community

## Target Audience

### End Users
- Privacy-conscious individuals
- Users in EU/EEA regions
- Those wanting local AI capabilities
- People seeking alternatives to mainstream browsers

### Developers
- Rust developers interested in browser technology
- AI/ML developers wanting browser integration
- Privacy technology enthusiasts
- Open source contributors

### Enterprises
- Organizations needing GDPR-compliant browsers
- Companies requiring local AI processing
- Institutions with strict privacy requirements

## Technology Stack

### Core Technologies
- **Rust**: Primary programming language
- **Servo**: Browser rendering engine
- **egui/winit**: UI framework
- **tokio**: Async runtime
- **rustls**: TLS implementation

### AI/ML Stack
- **Candle**: HuggingFace Rust ML framework
- **MCP**: Model Context Protocol
- **HuggingFace**: Model repository integration

### Development Tools
- **Cargo**: Rust package manager
- **Git**: Version control
- **CI/CD**: GitHub Actions (planned)
- **Documentation**: mdBook, Rustdoc

## Project Status

### Current Version: 0.1.0 (Alpha)

**Implemented Features:**
- Basic browser rendering with Servo
- Modular crate architecture
- Core types and error handling
- Network stack foundation
- MCP protocol implementation
- AI model runner foundation

**In Development:**
- Complete UI implementation
- Tab management
- Bookmark system
- Enhanced privacy features
- Performance optimizations

**Planned Features:**
- Extension system
- Password manager
- Sync capabilities
- Mobile versions
- Advanced privacy tools

## Why Choose Fenrir?

### For Privacy
- Built with privacy as a core design principle
- EU GDPR compliance considerations
- No hidden data collection
- Transparent about data processing

### For Performance
- Rust's memory safety and performance
- Servo's parallel rendering engine
- Hardware-accelerated graphics
- Efficient resource usage

### For AI Integration
- Local AI processing option
- Open AI protocol (MCP) support
- Multiple acceleration backends
- Privacy-preserving AI features

### For Developers
- Clean, modular architecture
- Comprehensive documentation
- Active development community
- Extensible design

## Getting Involved

### Using Fenrir
1. Check [Installation Guide](installation.md) for setup instructions
2. Review [Configuration Guide](configuration/guide.md) for customization
3. Explore [AI Features](ai/integration.md) for AI capabilities

### Contributing
1. Read [Contributing Guide](contributing/guide.md)
2. Set up [Development Environment](development/setup.md)
3. Check [Open Issues](https://github.com/your-org/fenrir/issues)

### Reporting Issues
- Use GitHub Issues for bug reports
- Include system information and steps to reproduce
- Check existing issues before creating new ones

## License

Fenrir Browser is licensed under the **European Union Public License 1.2 (EUPL-1.2)**.

This license ensures:
- Copyleft protection for derivatives
- Compatibility with other open source licenses
- Legal clarity for EU-based projects
- Strong software freedom protections

## Community

### Communication Channels
- **GitHub**: Primary development and issue tracking
- **Discord/Matrix**: Community discussions (planned)
- **Documentation**: This documentation site

### Code of Conduct
We are committed to providing a friendly, safe, and welcoming environment for all. Please read our Code of Conduct before participating.

## Acknowledgments

Fenrir Browser builds upon the work of many open source projects:

- **Servo Project**: For the amazing browser engine
- **Rust Community**: For the excellent ecosystem and tools
- **egui**: For the immediate-mode GUI framework
- **HuggingFace**: For the Candle ML framework
- **All Contributors**: For their time and expertise

---

*Next: [Installation Guide](installation.md) →*
