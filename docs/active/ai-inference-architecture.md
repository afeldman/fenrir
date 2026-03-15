# AI Browser Architecture

This document describes how **artificial intelligence is integrated into the Fenrir browser**.

Unlike traditional browsers where AI features are optional add-ons, Fenrir integrates AI as a **core subsystem**.

The AI system supports:

page understanding
bookmark intelligence
user assistance
automation
privacy analysis

All AI functionality is designed to run **locally whenever possible**.

---

# Architecture Overview

Fenrir integrates AI through the **fenrir-ai crate**.

High level architecture:

```id="a5s2rx"
User Interaction
        │
        ▼
Browser UI
        │
        ▼
Fenrir Engine
        │
        ▼
AI Runtime (fenrir-ai)
        │
 ┌──────┼───────────────┐
 ▼      ▼               ▼
Page AI  Bookmark AI   Agent System
        │
        ▼
Local AI Model (noeum-1-nano)
        │
        ▼
Inference Engine (Candle)
```

AI operates as a shared service used by multiple browser subsystems.

---

# Core AI Components

Fenrir AI architecture consists of several major components.

---

# AI Runtime

Location:

crates/fenrir-ai

Purpose:

Provide a unified runtime for AI tasks inside the browser.

Responsibilities:

model loading
prompt processing
inference execution
result caching

The runtime exposes AI capabilities to other browser subsystems.

---

# Local Model Execution

Fenrir executes AI models locally using:

Candle

Benefits:

no cloud dependency
low latency
privacy protection

All inference happens on the user's machine.

---

# Default Model

Fenrir uses:

noeum/noeum-1-nano

Model characteristics:

0.6B parameters
~0.2B active parameters
Mixture-of-Experts architecture

Advantages:

fast inference
low memory usage
good reasoning performance

---

# Thinking Mode

The model supports an optional reasoning mode.

Example:

```
/think
```

When enabled, the model performs deeper reasoning before producing a final answer.

This is useful for:

bookmark classification
page summarization
complex queries

---

# AI Subsystems

The AI runtime supports several subsystems.

---

# Page Understanding

The browser analyzes the current page.

Tasks include:

content summarization
topic detection
entity extraction

Example workflow:

```id="9r3rdb"
Page Loaded
     ↓
DOM Analysis
     ↓
Text Extraction
     ↓
AI Summarization
     ↓
User Display
```

This allows the browser to provide contextual information.

---

# Bookmark Intelligence

Bookmarks are automatically enriched with AI metadata.

Capabilities:

automatic tagging
topic classification
summary generation

This system is shared with the **GMarks architecture**.

---

# AI Assistant

Fenrir provides an integrated assistant.

Capabilities:

explain page content
answer questions about pages
summarize articles
help navigate complex websites

Example interaction:

User:

"Summarize this article."

AI:

Generates a summary from page content.

---

# Agent-Based Browsing

Future versions of Fenrir may include **AI agents**.

Agents can:

navigate websites
fill forms
perform repetitive browsing tasks

Example:

```id="5uvr8j"
User Request
   ↓
AI Agent
   ↓
Browser Control
   ↓
Automated Navigation
```

Agents operate with strict permission controls.

---

# Privacy Analysis

AI can analyze websites for privacy risks.

Examples:

tracker detection
cookie analysis
permission warnings

Example pipeline:

```id="n9c9m3"
Page Load
   ↓
Script Analysis
   ↓
AI Evaluation
   ↓
Privacy Warning
```

---

# MCP Integration

Fenrir supports the **Model Context Protocol (MCP)**.

MCP allows AI models to use external tools.

Example tools:

web search
document processing
browser automation

Architecture:

```id="63g2tm"
AI Runtime
     │
     ▼
MCP Client
     │
     ▼
External Tools
```

MCP allows controlled integration with external services.

---

# AI Task Scheduling

AI operations run asynchronously.

Typical tasks include:

bookmark classification
page summarization
assistant queries

Tasks run on background threads to avoid blocking the UI.

---

# Resource Management

AI models require memory and compute resources.

Fenrir implements safeguards:

model lazy loading
inference rate limiting
background execution

This ensures smooth browsing performance.

---

# Security Model

AI components must follow strict security rules.

Requirements:

no execution of arbitrary scripts
no remote model downloads without user consent
no transmission of page content without permission

AI must never bypass browser security policies.

---

# Future AI Capabilities

Possible future developments include:

semantic browsing
AI-driven search
knowledge graph navigation
personal knowledge assistants

These features will build upon the existing AI runtime.

---

# Design Principles

Fenrir AI architecture follows several principles.

Local First

AI runs locally whenever possible.

Privacy Preserving

User data stays on the device.

Modular Design

AI components remain independent of core browser logic.

Extensible

New models and AI capabilities can be added without changing the browser engine.

---

# Summary

Fenrir integrates AI as a **core browser capability**.

The AI runtime powers:

page understanding
bookmark intelligence
browser assistance
automation

By combining a local model (`noeum-1-nano`) with the Candle inference engine, Fenrir provides powerful AI features while preserving user privacy.

---

Related documents:

docs/ai-bookmark-architecture.md
docs/browser-internals.md
docs/web-compatibility.md
