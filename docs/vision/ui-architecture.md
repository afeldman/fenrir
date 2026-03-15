# Fenrir UI Architecture

This document describes the architecture of the **Fenrir browser user interface**.

The UI is designed around the concept of **AI-assisted workspaces** called **Arbeitsbereiche**.

Workspaces allow users to organize browsing activities into logical groups that can be automatically managed by AI.

---

# UI Design Goals

The Fenrir UI focuses on:

clarity
productivity
AI-assisted organization
minimal distraction

Key idea:

The browser should adapt to how the user works.

---

# UI Technology Stack

Fenrir uses native Rust UI technologies.

Frameworks:

winit
egui

Reasons:

full control over rendering
GPU compatibility with WebRender
minimal dependencies

The UI is fully native and does not use embedded webviews.

---

# Workspace Concept

The central UI abstraction is a **Workspace (Arbeitsbereich)**.

A workspace groups:

tabs
bookmarks
history
AI context

Example:

```text
Workspace: Research
 ├─ Tab: AI Paper
 ├─ Tab: GitHub Repo
 └─ Bookmark: Transformer Tutorial
```

Workspaces allow the browser to organize user activities.

---

# AI Workspace Organization

Fenrir can automatically organize workspaces using AI.

Example workflow:

```
Open multiple tabs
      ↓
AI detects topic similarity
      ↓
Tabs grouped into workspace
```

Example:

Tabs about:

Rust
Servo
WebRender

→ grouped into **"Browser Development" workspace**

---

# Workspace Structure

Internal model:

```rust
struct Workspace {
    id: WorkspaceId,
    name: String,
    tabs: Vec<Tab>,
    bookmarks: Vec<Bookmark>,
    context_tags: Vec<String>
}
```

Workspaces store browsing context.

---

# Workspace AI Context

Each workspace maintains an **AI context**.

Context includes:

recent pages
topics
entities

This allows the assistant to understand user intent.

Example:

User asks:

"Summarize my research."

AI summarizes the workspace.

---

# Tab Model

Tabs belong to workspaces.

Example:

```rust
struct Tab {
    id: TabId,
    url: Url,
    title: String,
    renderer: WebView
}
```

Tabs map directly to Servo WebViews.

---

# Workspace Navigation

The UI provides:

workspace switcher
tab overview
AI workspace suggestions

Example:

User opens many pages about crypto.

Fenrir suggests:

"Create workspace: Web3"

---

# UI Components

Major UI components include:

workspace sidebar
tab bar
browser viewport
AI assistant panel

---

# AI Assistant Panel

The assistant panel allows users to interact with the browser AI.

Capabilities:

page summarization
workspace insights
bookmark recommendations

Example:

User:

"Organize these tabs."

AI groups them into workspaces.

---

# State Management

UI state is managed through:

fenrir-core

Shared state structure:

```
BrowserState
 ├─ Workspaces
 ├─ Active Workspace
 └─ Tabs
```

---

# Future UI Features

Future improvements include:

AI tab grouping
semantic search across tabs
workspace history
visual knowledge maps

---

# Summary

Fenrir UI is built around **AI-assisted workspaces**.

Workspaces organize browsing into meaningful contexts.

AI helps users manage information without manual sorting.
