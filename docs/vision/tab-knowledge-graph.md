# Tab Knowledge Graph Architecture

This document describes the **Tab Knowledge Graph system** used in the Fenrir browser.

The goal of the knowledge graph is to transform browsing data into a structured knowledge system that can be used by AI and workspace management.

Instead of treating tabs as isolated entities, Fenrir represents browsing activity as a **graph of connected knowledge nodes**.

---

# Concept Overview

Traditional browsers treat tabs as independent units.

Fenrir introduces a **knowledge graph model**.

Graph structure:

```
Tab
 │
 ├─ Page Content
 │
 ├─ Workspace
 │
 ├─ Bookmark
 │
 └─ Entities
```

Each browsing activity becomes a node in the graph.

The AI system can reason about relationships between nodes.

---

# Graph Model

The knowledge graph consists of nodes and edges.

Nodes represent entities.

Edges represent relationships.

Example graph:

```
Workspace
   │
   ├── Tab
   │     │
   │     └── Web Page
   │            │
   │            ├── Topic
   │            ├── Entities
   │            └── Links
   │
   └── Bookmark
```

---

# Node Types

The knowledge graph supports multiple node types.

Tab
Workspace
Bookmark
Page
Topic
Entity

Example entity types:

Person
Organization
Technology
Concept

---

# Example Knowledge Graph

Example browsing session:

User visits:

Servo documentation
WebRender repository
Rust GPU article

Graph representation:

```
Workspace: Browser Development
   │
   ├── Tab: Servo Docs
   │       └── Topic: Browser Engines
   │
   ├── Tab: WebRender Repo
   │       └── Topic: GPU Rendering
   │
   └── Tab: Rust GPU Article
           └── Topic: Graphics Programming
```

AI can detect relationships between these pages.

---

# Data Model

Example node structure:

```rust
struct GraphNode {
    id: NodeId,
    node_type: NodeType,
    metadata: Metadata,
}
```

Edge structure:

```rust
struct GraphEdge {
    source: NodeId,
    target: NodeId,
    relation: RelationType,
}
```

---

# Relation Types

Common relation types include:

belongs_to_workspace
related_topic
duplicate_page
references
similar_content

Example:

```
Tab A ── related_topic ── Tab B
```

---

# Page Analysis Pipeline

When a page loads, the system analyzes its content.

Pipeline:

```
Page Load
   ↓
Text Extraction
   ↓
Entity Extraction
   ↓
Topic Detection
   ↓
Graph Update
```

The graph is continuously updated during browsing.

---

# Entity Extraction

AI extracts important entities from pages.

Examples:

programming languages
companies
technologies

Example:

Page about:

Rust async runtime

Entities extracted:

Rust
Tokio
async programming

These entities become graph nodes.

---

# Topic Detection

AI identifies page topics.

Example:

Page topic:

Browser Engine Development

Graph update:

```
Tab → Topic: Browser Development
```

Topics help group related browsing activities.

---

# Workspace Integration

Workspaces are represented as graph nodes.

Example:

```
Workspace
   │
   ├── Tab
   ├── Bookmark
   └── Topic
```

This allows AI to understand the context of a workspace.

---

# Bookmark Integration

Bookmarks also become graph nodes.

Example:

```
Bookmark
   │
   └── Topic
```

This allows bookmarks to participate in the knowledge graph.

---

# Semantic Similarity

AI can compute similarity between pages.

Method:

embedding vectors.

Example:

```
Tab A → embedding
Tab B → embedding

cosine similarity
```

If similarity is high, an edge is created.

---

# Graph Storage

The graph is stored locally.

Possible storage:

```
redb
```

Graph data includes:

nodes
edges
embeddings

This allows fast local queries.

---

# Graph Queries

AI and UI can query the graph.

Example queries:

Find related tabs
Find related bookmarks
Find previous research on a topic

Example:

```
query:
related_tabs("Rust async")
```

Result:

Tabs related to Rust async programming.

---

# AI Use Cases

The knowledge graph enables many AI features.

Examples:

workspace recommendations
tab grouping
research summaries
semantic search

Example:

User asks:

"What have I researched about WebRender?"

AI queries the graph.

---

# Visualization

Future versions may visualize the knowledge graph.

Possible UI:

```
Workspace Graph View
```

Nodes represent:

tabs
topics
entities

Edges show relationships.

This helps users explore their knowledge.

---

# Privacy Model

All graph data is stored locally.

No browsing knowledge is transmitted externally unless explicitly enabled.

This ensures strong privacy guarantees.

---

# Future Improvements

Possible future features include:

knowledge graph visualization
automatic research summaries
semantic browsing history
cross-workspace insights

The knowledge graph will become the foundation for AI-powered browsing.

---

# Summary

The Fenrir Tab Knowledge Graph transforms browsing activity into structured knowledge.

The system connects:

tabs
workspaces
bookmarks
topics
entities

This allows AI to understand user activity and provide intelligent assistance.

---

Related documents:

workspace-ai-architecture.md
ai-browser-architecture.md
ai-bookmark-architecture.md
