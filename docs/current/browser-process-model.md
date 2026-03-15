# Browser Process Model

This document describes the **process architecture of the Fenrir browser**.

The process model defines how different parts of the browser are isolated for:

security
stability
performance

Fenrir currently runs in a **single-process architecture** but is designed to evolve into a **multi-process browser model**.

---

# Process Model Overview

Modern browsers separate responsibilities into multiple processes.

Typical model:

Browser Process
Renderer Processes
GPU Process
Network Process

Fenrir initially starts with a simplified architecture.

---

# Current Architecture

In early development Fenrir runs primarily in a **single process**.

Structure:

```id="x1a7g4"
Fenrir Process
 ├─ UI Layer
 ├─ Browser Engine
 ├─ Network Stack
 ├─ AI Runtime
 └─ Servo Renderer
```

Advantages:

simpler debugging
lower complexity
faster development iteration

However long term this model is not sufficient for strong security isolation.

---

# Planned Multi-Process Model

Future versions of Fenrir will adopt a **multi-process architecture**.

```id="5d6qjp"
┌─────────────────────────────┐
│        Browser Process      │
│                             │
│  UI │ Tab Management │ AI   │
└──────────────┬──────────────┘
               │
        ┌──────▼──────┐
        │ Renderer    │
        │ Process     │
        │ (Servo)     │
        └──────┬──────┘
               │
       ┌───────▼────────┐
       │ GPU Process     │
       │ WebRender       │
       └───────┬────────┘
               │
       ┌───────▼────────┐
       │ Network Process │
       └─────────────────┘
```

This architecture isolates untrusted content from critical browser systems.

---

# Browser Process

The **browser process** is the central controller.

Responsibilities:

UI rendering
tab management
navigation decisions
security policies
AI runtime

Main components:

fenrir-app
fenrir-core
fenrir-ai

The browser process should remain **trusted code only**.

---

# Renderer Process

The renderer process executes web content.

Responsibilities:

HTML parsing
DOM creation
CSS styling
layout computation
JavaScript execution

In Fenrir this is handled by:

Servo

Each tab may run in its own renderer process in future versions.

Benefits:

site isolation
crash containment
improved security

---

# GPU Process

The GPU process handles rendering operations.

Responsibilities:

WebRender execution
GPU resource management
frame composition

Separating GPU work prevents browser crashes from graphics driver issues.

---

# Network Process

The network process handles all network traffic.

Responsibilities:

DNS resolution
HTTP requests
TLS handling
request filtering

Fenrir uses:

reqwest
rustls
hickory-dns

Isolating networking improves security and allows better traffic control.

---

# AI Runtime Process

Fenrir includes a local AI runtime.

Responsibilities:

model loading
AI inference
assistant tasks

The AI runtime may remain in the browser process initially.

Future versions may isolate heavy AI workloads into a separate process.

---

# Interprocess Communication

Processes communicate using **message passing**.

Possible technologies:

IPC channels
shared memory
tokio channels

Typical message flow:

```id="wq9rfa"
UI Event
   ↓
Browser Process
   ↓
Renderer Process
   ↓
Render Update
   ↓
GPU Process
```

---

# Crash Isolation

A multi-process architecture prevents browser-wide crashes.

Examples:

Renderer crash → tab reload
GPU crash → renderer restart
network crash → connection reset

The browser process remains alive.

---

# Site Isolation

Future Fenrir versions may implement **site isolation**.

Example:

```id="g3snkt"
Tab 1 (example.com) → Renderer Process A
Tab 2 (news.com) → Renderer Process B
```

Benefits:

security isolation
memory protection
reduced attack surface

---

# Resource Management

The browser process monitors resource usage.

Responsibilities:

process lifecycle
memory limits
CPU scheduling

Inactive renderer processes may be suspended.

---

# Future Improvements

Possible improvements include:

per-tab renderer processes
AI worker processes
sandboxed GPU pipeline
improved IPC layer

These changes will increase security and stability.

---

# Summary

Fenrir begins with a **single-process architecture** for simplicity.

The long-term architecture evolves toward:

browser process
renderer processes
GPU process
network process

This model balances **security, performance, and maintainability**.

---

Related documents:

docs/security-architecture.md
docs/browser-internals.md
docs/rendering-pipeline.md
