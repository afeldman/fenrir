# Fenrir Browser Architecture

This document describes the **high level architecture of the Fenrir browser**.

Fenrir is a privacy-focused browser built in Rust that embeds the **Servo rendering engine** while providing its own infrastructure for networking, security, AI and user interface.

This document explains the major components and how they interact.

More detailed documents are available in `docs/`.

---

# System Overview

Fenrir follows a **modular architecture** built around a set of independent crates.

High-level architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                        Fenrir Browser                       │
├─────────────────────────────────────────────────────────────┤
│                         UI Layer                            │
│                  (winit + egui interface)                   │
├─────────────────────────────────────────────────────────────┤
│                        Fenrir Engine                        │
│                                                             │
│   Core System   │   Network Stack   │   AI Runtime          │
│   fenrir-core   │   fenrir-network  │   fenrir-ai           │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│                    Servo Rendering Engine                   │
│                     (HTML / CSS / Layout)                   │
├─────────────────────────────────────────────────────────────┤
│                         WebRender                           │
│                      GPU Rendering                          │
├─────────────────────────────────────────────────────────────┤
│                Operating System & Hardware                  │
└─────────────────────────────────────────────────────────────┘
```

Fenrir focuses on building **browser infrastructure** while Servo provides the **web rendering engine**.

---

# Core Architectural Principles

## Privacy by Design

Fenrir is designed to minimize data exposure.

Principles include:

* local AI inference whenever possible
* minimal telemetry
* transparent data flow
* strict network controls

Sensitive data remains on the user's device.

---

## Modular Architecture

Each subsystem is implemented as a separate crate.

Benefits:

* clear boundaries
* easier testing
* independent development
* replaceable implementations

Subsystems communicate through well-defined interfaces.

---

## Rust-Native Stack

Fenrir avoids legacy C/C++ dependencies whenever possible.

Core technologies include:

Rust
tokio async runtime
rustls TLS stack
wgpu graphics abstraction

This ensures:

memory safety
modern concurrency
cross-platform portability.

---

## Security First

Security is integrated at every layer.

Techniques include:

strict origin policies
sandboxing
secure networking
cryptographic best practices

Rust's type system prevents entire classes of vulnerabilities.

---

# Major Components

Fenrir consists of several core subsystems.

---

# UI Layer

Crate: **fenrir-app**

Purpose:

User interface and window management.

Responsibilities:

window creation
input handling
toolbar and navigation controls
display of rendered content

Technologies used:

winit
egui

The UI layer interacts with the browser engine through event messages.

---

# Core System

Crate: **fenrir-core**

Purpose:

Provide shared types and infrastructure for the entire browser.

Responsibilities:

global browser state
tab management
event system
configuration access
plugin interfaces

Key types include:

BrowserState
Tab
Event
FenrirError

The core system coordinates all other subsystems.

---

# Browser Engine Integration

Crate: **fenrir-servo**

Purpose:

Embed the Servo rendering engine.

Responsibilities:

initialize Servo
create WebView instances
handle navigation
forward input events
process rendering callbacks

Each browser tab corresponds to a Servo WebView.

Servo provides:

HTML parsing
DOM construction
CSS styling
layout computation
display list generation.

---

# Network Stack

Crate: **fenrir-network**

Purpose:

Provide HTTP networking and request management.

Responsibilities:

DNS resolution
HTTP/HTTPS requests
request interception
proxy support
traffic management

Libraries used:

reqwest
rustls
hickory-dns

The network layer can intercept requests for privacy filtering.

---

# AI System

Crate: **fenrir-ai**

Purpose:

Local AI model execution and integration.

Responsibilities:

model loading
inference execution
prompt processing
AI feature integration

Inference framework:

Candle

Typical uses include:

page summarization
bookmark classification
assistant features.

AI runs locally whenever possible.

---

# MCP System

Crate: **fenrir-mcp**

Purpose:

Implementation of the Model Context Protocol.

Responsibilities:

AI tool routing
provider management
protocol communication

This allows Fenrir to interact with external AI systems in a standardized way.

---

# Data Flow

## Page Load Sequence

```
User input
   ↓
UI Layer
   ↓
URL parsing
   ↓
Network request
   ↓
HTTP response
   ↓
Servo rendering
   ↓
GPU rendering
   ↓
Display in window
```

---

## AI Request Flow

```
User request
   ↓
UI layer
   ↓
AI subsystem
   ↓
Local model inference
   ↓
Result processing
   ↓
UI display
```

---

# Communication Patterns

Fenrir uses several communication mechanisms.

### Event Bus

Components publish browser events.

Example:

PageLoaded
NavigationStarted
TabCreated

Components subscribe to events they care about.

---

### Async Tasks

Network requests and AI operations run asynchronously using **tokio**.

Example:

HTTP requests
model inference
background processing.

---

### Shared State

Global browser state is protected using:

Arc<RwLock<T>>

This allows safe concurrent access across tasks.

---

# Dependency Structure

Internal crate relationships:

```
fenrir-app
 ├─ fenrir-core
 ├─ fenrir-servo
 ├─ fenrir-network
 ├─ fenrir-ai
 └─ fenrir-mcp

fenrir-ai
 └─ fenrir-mcp

fenrir-mcp
 └─ fenrir-core
```

Servo is included as an external dependency.

---

# Configuration System

Fenrir uses layered configuration.

Sources:

1 default values
2 environment variables
3 config file
4 command line arguments
5 runtime settings

Typical config location:

```
~/.config/fenrir/config.toml
```

---

# Security Architecture

Fenrir uses **defense in depth**.

Layers include:

memory safety (Rust)
secure TLS stack
origin-based permissions
request validation

Future security modules:

fenrir-security
fenrir-privacy.

---

# Performance Architecture

Fenrir relies on asynchronous execution.

Main strategies:

parallel tasks
GPU rendering
network connection pooling
render caching.

Servo and WebRender perform most rendering optimizations.

---

# Monitoring and Diagnostics

Fenrir uses structured logging.

Logging framework:

tracing

Logs include:

network activity
render events
errors
performance metrics.

Debug tools include:

developer console
network inspector
debug panel.

---

# Future Architecture

Future improvements include:

process isolation per tab
privacy filtering engine
encrypted sync
plugin architecture.

Fenrir aims to remain modular and extensible.

---

# Related Documents

More detailed information is available in:

**Implementiert (`docs/current/`):**
- docs/current/browser-internals.md
- docs/current/rendering-pipeline.md
- docs/current/servo-integration.md
- docs/current/network-architecture.md
- docs/current/web-compatibility.md
- docs/current/browser-process-model.md

**In Entwicklung (`docs/active/`):**
- docs/active/ai-inference-architecture.md
- docs/active/mcp-providers.md
- docs/active/security-architecture.md
- docs/active/performance-architecture.md

**Geplant (`docs/vision/`):**
- docs/vision/ui-architecture.md
- docs/vision/extension-architecture.md
- docs/vision/wallet-architecture.md

Roadmap: ROADMAP.md
