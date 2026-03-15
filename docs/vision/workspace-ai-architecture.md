# Workspace AI Architecture

This document describes how **AI manages workspaces in the Fenrir browser**.

Workspaces ("Arbeitsbereiche") are the central concept of the Fenrir user interface.

They allow users to organize browsing activities into meaningful contexts.

AI assists users by automatically:

grouping tabs
detecting topics
suggesting workspaces
maintaining workspace context

---

# Workspace Concept

A workspace represents a **logical browsing context**.

Examples:

Research
Work Project
Shopping
Crypto

Each workspace contains:

tabs
bookmarks
history
AI context

Example:

```text
Workspace: Browser Development
 ├ Servo documentation
 ├ WebRender repository
 ├ Rust async article
```

---

# Workspace Data Model

Example structure:

```rust
struct Workspace {
    id: WorkspaceId,
    name: String,
    tabs: Vec<Tab>,
    bookmarks: Vec<Bookmark>,
    tags: Vec<String>,
    ai_context: WorkspaceContext
}
```

The workspace context stores semantic information used by the AI.

---

# Workspace Context Model

Workspace context is continuously updated by the AI.

Example:

```rust
struct WorkspaceContext {
    topics: Vec<String>,
    entities: Vec<String>,
    recent_pages: Vec<Url>,
    embeddings: Vec<Vector>
}
```

This allows AI to understand the purpose of a workspace.

---

# Tab Analysis Pipeline

When a new tab is opened the browser analyzes the page.

Process:

```text
Tab Opened
    ↓
Metadata Extraction
    ↓
Content Extraction
    ↓
AI Topic Detection
    ↓
Workspace Classification
```

The AI decides whether the tab belongs to an existing workspace.

---

# Metadata Extraction

Basic information is extracted from the page.

Sources include:

page title
meta description
OpenGraph tags
URL structure

Example:

```text
https://github.com/servo/servo
```

Possible tags:

Rust
Browser Engine
Open Source

---

# Content Extraction

Text is extracted from the page.

Sources:

DOM text nodes
article content
headings

This text is used for AI analysis.

---

# AI Topic Detection

The AI model analyzes page content.

Model used:

noeum-1-nano

Example prompt:

```
Determine the main topic of this webpage.
Return tags and category.
```

Example output:

```json
{
  "category": "software development",
  "tags": ["rust", "browser engine"]
}
```

---

# Workspace Classification

The AI compares the new tab with existing workspaces.

Method:

semantic similarity between embeddings.

Example:

Workspace topics:

Rust
WebRender
Servo

New tab topic:

Rust GPU rendering

Result:

Tab added to **Browser Development workspace**.

---

# Workspace Suggestion System

If no workspace matches, AI may suggest a new one.

Example:

User opens several pages about:

Ethereum
MetaMask
Solidity

AI suggests:

```
Create workspace: Web3 Development
```

---

# Workspace Evolution

Workspaces evolve over time.

Example:

Research workspace may gradually specialize.

Example progression:

```
AI Research
 → LLM Architectures
 → Mixture-of-Experts
```

AI can update workspace tags automatically.

---

# Cross Workspace Insights

AI can detect relationships between workspaces.

Example:

```
Workspace A: Rust Development
Workspace B: Browser Development
```

AI may suggest merging or linking them.

---

# Workspace Memory

Workspaces act as **long-term knowledge containers**.

Stored data:

topics
summaries
key pages

This allows AI to answer questions like:

```
What have I researched about Rust async?
```

---

# Workspace AI Tasks

AI performs several background tasks:

tab classification
topic extraction
workspace summarization
bookmark suggestions

These tasks run asynchronously.

---

# Workspace Assistant

The AI assistant can interact with workspaces.

Example commands:

```
Summarize this workspace
Group these tabs
Find related articles
```

Example result:

AI generates workspace overview.

---

# Privacy Model

Workspace AI operates locally whenever possible.

Processing uses:

local model inference

User data is not transmitted externally without permission.

---

# Future Workspace Features

Future capabilities include:

knowledge graph visualization
semantic search across workspaces
AI-generated research summaries
workspace timelines

These features transform workspaces into **personal knowledge hubs**.

---

# Summary

Workspaces are the core organizational structure of Fenrir.

AI helps manage them by:

analyzing page content
grouping tabs
detecting topics
suggesting workspace structures

This allows the browser to become a **knowledge workspace instead of just a tab container**.
