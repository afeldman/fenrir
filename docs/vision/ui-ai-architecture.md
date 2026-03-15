# Fenrir UI + AI Architecture

This document describes the architecture of the **Fenrir user interface and AI interaction layer**.

Fenrir integrates AI directly into the browser UI instead of treating it as a separate feature.

The UI combines:

native Rust UI
AI-driven interaction
workspace organization
LLM-powered commands

---

# Design Philosophy

Traditional browsers:

UI + optional AI chatbot.

Fenrir:

AI **integrated into the UI layer**.

The browser becomes an **interactive knowledge workspace**.

---

# High Level Architecture

```text
User
 │
 ▼
Fenrir UI (egui)
 │
 ▼
UI State Layer
 │
 ▼
AI Interaction Layer
 │
 ├── LLM Runtime (fenrir-ai)
 ├── MCP Tool System
 └── Browser APIs
```

AI is able to interact with browser functionality through controlled APIs.

---

# UI Layer

The UI is implemented using:

winit
egui

Major UI regions:

Workspace Sidebar
Tab Bar
Browser Viewport
AI Panel

Example layout:

```text
┌────────────────────────────────────────────┐
│ Workspace │ Tabs │ URL Bar                │
├────────────────────────────────────────────┤
│                                            │
│            Browser Viewport                │
│                                            │
├───────────────────────────────┬────────────┤
│ Workspace Panel               │ AI Panel   │
└───────────────────────────────┴────────────┘
```

---

# Workspace System

The central UI abstraction is the **Workspace**.

A workspace groups related browsing activities.

Example:

```
Workspace: Rust Development
 ├ Servo docs
 ├ GitHub issues
 ├ WebRender documentation
```

Workspaces store:

tabs
bookmarks
AI context

---

# AI Workspace Intelligence

AI analyzes tabs and suggests workspace groupings.

Example:

```
User opens:
Rust docs
Servo repo
WebRender blog

AI suggests:
Workspace: Browser Engine Development
```

The AI system uses:

page content
page titles
semantic embeddings

---

# AI Command Interface

Fenrir supports **LLM commands** similar to Velo.

Example commands:

```
/summarize
/extract-links
/group-tabs
/find-related
```

Example usage:

User:

"Group my tabs into topics."

AI:

Creates workspaces automatically.

---

# LLM Interaction Model

User requests are processed as structured AI tasks.

Flow:

```
User Prompt
     ↓
Prompt Parser
     ↓
AI Runtime
     ↓
MCP Tool Calls
     ↓
Browser Action
```

Example:

User:

"Summarize this page."

Flow:

```
extract_page_text
      ↓
AI summarization
      ↓
display result
```

---

# MCP Tool Integration

The AI UI connects to MCP tools.

Examples:

navigate_page
extract_page_text
search_bookmarks
organize_tabs

Example:

```
AI → organize_tabs()
```

The MCP system ensures that AI actions are safe and controlled.

---

# Context System

AI interactions require browser context.

Context includes:

current page content
workspace history
recent tabs

Context model:

```
AIContext
 ├ Current Page
 ├ Workspace State
 └ User History
```

This allows AI to reason about the browsing session.

---

# AI Panel

The AI panel provides an interactive assistant.

Capabilities:

page summarization
research help
tab organization
bookmark suggestions

Example interaction:

User:

"What are the key ideas on this page?"

AI:

Returns summary + highlights.

---

# Inline AI Actions

AI actions can appear directly in the UI.

Examples:

"Summarize article" button
"Group tabs" suggestion
"Create workspace"

These suggestions appear contextually.

---

# Automation

Future versions may support AI automation.

Example:

```
User:
"Research Rust async runtimes"
```

AI performs:

```
search web
open tabs
summarize results
create workspace
```

This creates a research workspace automatically.

---

# Privacy Model

AI processing should occur locally whenever possible.

Default model:

```
noeum-1-nano
```

Advantages:

local inference
low latency
privacy protection

External AI services require explicit user consent.

---

# UI State Management

UI state is managed in:

```
fenrir-core
```

Example state model:

```
BrowserState
 ├ Workspaces
 ├ Tabs
 ├ ActiveTab
 └ AIContext
```

The UI reacts to state changes.

---

# Future Features

Planned features include:

semantic tab search
AI workspace recommendations
knowledge graph visualization
AI browsing agents

These features will expand the AI-driven browsing experience.

---

# Summary

Fenrir combines:

native Rust UI
AI-driven workflows
workspace-based browsing
LLM command interface

This creates a browser designed for **AI-assisted knowledge work**.
