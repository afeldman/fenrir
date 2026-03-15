# Fenrir API Architecture

This document describes the **external API architecture of the Fenrir browser**.

The API system allows external tools, AI systems, and plugins to interact with Fenrir in a controlled way.

---

# API Goals

The Fenrir API is designed to provide:

automation
tool integration
AI interoperability
extension support

The API must remain:

secure
stable
versioned

---

# API Architecture Overview

Fenrir exposes APIs through multiple layers.

```
External Client
      │
      ▼
Fenrir API Layer
      │
      ▼
Core Browser Services
```

Supported clients include:

MCP tools
CLI tools
browser extensions
automation agents

---

# API Types

Fenrir provides several API types.

Browser Control API
Navigation API
Bookmark API
AI API
Wallet API

---

# Example API Calls

Example navigation command:

```json
{
  "command": "navigate",
  "url": "https://example.com"
}
```

Example AI call:

```json
{
  "command": "summarize_page"
}
```

---

# MCP Integration

Fenrir APIs are accessible through MCP tools.

Example MCP tool:

```
navigate_page(url)
```

This allows AI agents to control the browser.

---

# Internal API Layer

Internal APIs are implemented in:

fenrir-core

Responsibilities:

command validation
permission checks
event dispatch

---

# Permission Model

Each API call must pass permission checks.

Examples:

navigation → allowed
file system → restricted
wallet access → user approval

---

# API Versioning

Fenrir APIs are versioned.

Example:

```
/api/v1/navigate
/api/v1/bookmarks
```

This ensures backward compatibility.

---

# Event Streaming

Clients can subscribe to browser events.

Examples:

tab_created
page_loaded
workspace_changed

Example:

```
subscribe: page_loaded
```

---

# Automation

The API allows automation tools to control Fenrir.

Example tasks:

open pages
extract data
run AI analysis

Automation is especially useful for AI agents.

---

# Summary

The Fenrir API provides a secure interface for interacting with the browser.

It enables:

automation
AI integration
external tools
