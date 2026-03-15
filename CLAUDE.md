# CLAUDE.md

This file provides guidance to Claude Code when working inside the **Fenrir browser** project.

Fenrir is a privacy-first, Rust-native browser using the **Servo rendering engine** and **local AI inference**.

The browser is developed as part of the larger **browser monorepo**, but this document only applies to the **fenrir/** directory.

---

# Project Status

Phase: **1–2 active development**

Current capabilities:

* Browser launches
* Pages load over HTTP
* Servo rendering works
* Logging infrastructure operational
* Basic toolbar implemented
* Debug panel available

Some modern sites fail due to missing Web APIs in Servo (e.g. IntersectionObserver).

---

# High Level Architecture

Fenrir is structured as a modular Rust workspace.

Core layers:

Fenrir App
↓
UI (winit + egui)
↓
Fenrir Engine
↓
Servo Rendering
↓
WebRender
↓
wgpu → GPU

Fenrir focuses on privacy, local AI, and Rust-native infrastructure.

---

# Workspace Layout

fenrir/

fenrir-app/
Binary entry point

crates/

fenrir-core
Core types, traits, event bus, shared state

fenrir-config
Settings, profiles, feature flags

fenrir-network
HTTP stack, DNS, proxy, interceptors

fenrir-servo
Servo wrapper and WebView integration

fenrir-ai
Local AI inference runtime

fenrir-mcp
MCP host + server

fenrir-log
Logging infrastructure with rolling logs

fenrir-error-management
Centralized error handling

fenrir-i18n
Internationalization system

Future crates planned:

fenrir-crypto
fenrir-security
fenrir-privacy
fenrir-vpn
fenrir-identity
fenrir-passwords
fenrir-wallet
fenrir-web3
fenrir-bookmarks
fenrir-mail
fenrir-ui
fenrir-sync
fenrir-update
fenrir-zerotrust
fenrir-compute

---

# Rendering Stack

Fenrir uses Servo via git dependency.

Servo provides:

* HTML parsing
* CSS styling
* layout engine
* DOM implementation
* WebRender GPU pipeline

Fenrir does **not modify Servo internally**.

Integration happens through:

fenrir-servo crate.

Important:

The local `servo/` directory in the monorepo exists **only for upstream contributions**, not for building Fenrir.

---

# UI Layer

Fenrir uses:

winit + egui

Reasons:

* full control over rendering
* minimal dependencies
* no webview wrappers
* easier GPU integration

The UI currently provides:

* toolbar
* URL bar
* debug panel (F12)
* event logging

---

# AI Integration

Fenrir runs **local AI inference** using Candle.

Default model:

noeum/noeum-1-nano

Properties:

Architecture: Ramo (MoE + GQA + YaRN-RoPE)
Parameters: 0.6B total / ~0.2B active
License: Apache 2.0
Origin: Austria

Model is used for:

* bookmark classification
* page summarization
* extension APIs
* assistant features

Inference implementation lives in:

crates/fenrir-ai/

No cloud fallback is allowed.

All AI must run locally.

---

# Networking

Networking is implemented in:

fenrir-network

Main libraries:

reqwest
rustls
hickory-dns

Capabilities:

* HTTP client
* DNS resolution
* HTTPS interception
* DoH support
* proxy support

All network access **must go through fenrir-network**.

---

# Logging

Logging is implemented via:

tracing
logroller

Logs are written to:

~/Library/Application Support/fenrir/logs/

Features:

* rolling logs
* debug panel integration
* structured tracing

---

# Error Handling

Fenrir uses:

thiserror
anyhow

All crates must propagate errors using structured error types.

Production code must not use:

unwrap()
expect()

---

# Security Rules (never violate)

1. Crypto must only exist in **fenrir-crypto**
2. Network access must only go through **fenrir-network**
3. Private keys must never leave memory unencrypted
4. Secrets must implement **zeroize**
5. No OpenSSL allowed — use **rustls**
6. All external URLs must be validated by **fenrir-security**
7. Sync must never transmit:

   * private keys
   * master passwords
   * VPN credentials
8. All cryptographic operations must be auditable
9. No C dependencies if Rust alternatives exist
10. All data collection must support GDPR audit entries

---

# Crypto Policy

Allowed primitives:

ChaCha20-Poly1305
Ed25519
Argon2id
BLAKE3

Key storage:

keyring-rs using OS keychain.

Crypto must never be implemented outside the crypto crate.

---

# Storage

Local database:

redb

Used for:

* browser state
* bookmarks
* metadata
* AI embeddings (future)

---

# P2P and Sync

Future sync layer uses:

iroh

Goals:

* peer-to-peer sync
* optional self-hosted nodes
* encrypted device federation

---

# Development Workflow

Rust toolchain:

1.91.0

Common commands:

cargo build

cargo run -p fenrir-app

cargo nextest run

Format and lint:

cargo fmt
cargo clippy

---

# Goose Automation

Some tasks are executed via Goose agents.

Maximum:

2 concurrent processes.

Example run:

~/.local/bin/goose run --no-session --with-builtin developer --max-turns 60 
-i .goose/task_tests.md > /tmp/goose_tests.log 2>&1 &

Task queue is defined in `.goose/`.

---

# Current Known Limitations

Modern websites may fail due to missing Servo APIs.

Example:

DuckDuckGo main page requires IntersectionObserver.

Workaround:

lite.duckduckgo.com

Upstream solution:

Complete IntersectionObserver support in Servo.

---

# Design Philosophy

Fenrir follows these principles:

Rust-first
privacy-first
local AI by default
minimal dependencies
no vendor lock-in
EU regulatory compliance

Avoid unnecessary complexity.

Prefer simple, auditable Rust code.

---

# Things Claude must never do

Do not modify Servo internals inside Fenrir.

Do not introduce OpenSSL.

Do not add large frameworks.

Do not introduce cloud AI APIs.

Do not add unsafe code unless absolutely necessary.

---

# Future Goals

Complete browser stack including:

* AI assistant
* privacy engine
* tracker blocking
* encrypted sync
* Web3 integration
* local compute marketplace

Fenrir aims to become a **sovereign EU browser platform** built entirely in Rust.

---

License: EUPL-1.2
