# Browser Internals

This document explains **how modern browsers work internally** and how Fenrir maps onto those concepts.

It serves as a reference for contributors working on Fenrir.

Modern browsers such as Chrome, Firefox and Safari all share similar architectural principles.

---

# Core Browser Components

A modern browser typically consists of the following subsystems:

User Interface
Browser Process
Rendering Engine
JavaScript Engine
Networking Stack
GPU Renderer
Storage System

Fenrir follows the same conceptual model.

---

# Browser Process

The browser process is responsible for:

window management
navigation
security policy
process orchestration

In Fenrir this role is implemented by:

fenrir-app
fenrir-core

The browser process initializes all subsystems.

---

# Rendering Engine

The rendering engine is responsible for displaying web content.

Responsibilities include:

HTML parsing
CSS styling
layout computation
painting

Fenrir uses:

Servo

Servo internally includes:

HTML parser
CSS engine
layout engine
DOM implementation

---

# DOM (Document Object Model)

The DOM is a structured representation of the HTML document.

Example HTML:

<html>
  <body>
    <h1>Hello</h1>
  </body>
</html>

Becomes:

Document
└── html
└── body
└── h1

DOM nodes contain:

attributes
text content
event listeners

Servo builds the DOM while parsing HTML.

---

# CSS Style System

CSS rules are applied to DOM nodes.

Example:

h1 {
color: red;
}

The browser must:

match selectors
compute style rules
inherit properties

Servo uses a **parallel CSS engine** derived from Mozilla's style system.

---

# Layout Engine

The layout engine determines **where elements appear on screen**.

This stage calculates:

element sizes
positions
flow rules

Example:

block layout
flexbox
grid layout

Output:

Layout Tree

The layout tree contains only **visible renderable nodes**.

---

# Display List

After layout the browser produces a **display list**.

The display list describes drawing commands.

Examples:

draw text
draw rectangle
draw image

This list is independent from the GPU implementation.

---

# GPU Rendering

Modern browsers render using the GPU.

Servo uses:

WebRender

WebRender converts display lists into GPU commands.

Pipeline:

Display List
↓
Scene Graph
↓
GPU Commands
↓
wgpu
↓
Graphics API

Supported APIs:

Vulkan
Metal
DirectX12
OpenGL (fallback)

---

# JavaScript Engine

JavaScript engines execute web scripts.

Typical responsibilities:

parse JavaScript
execute bytecode
manage garbage collection

Examples of JS engines:

V8 (Chrome)
SpiderMonkey (Firefox)
JavaScriptCore (Safari)

Servo currently uses:

SpiderMonkey

Fenrir may experiment with alternative engines in the future.

---

# Networking

Browsers include a full networking stack.

Responsibilities:

DNS resolution
HTTP requests
TLS encryption
caching

Fenrir implements networking in:

fenrir-network

Libraries used:

reqwest
rustls
hickory-dns

---

# Event System

Browsers react to events.

Examples:

mouse click
keyboard input
scroll
DOM mutation

Event flow:

Input
↓
UI framework
↓
browser engine
↓
DOM event dispatch

---

# Storage Systems

Browsers support multiple storage mechanisms.

Examples:

cookies
localStorage
IndexedDB
cache storage

Fenrir currently uses:

redb

Future features may include:

encrypted storage
syncable state

---

# Multi Process Architecture

Modern browsers isolate content into processes.

Examples:

Chrome

Browser Process
Renderer Process
GPU Process
Network Process

Servo currently runs in a single process model.

Future Fenrir versions may implement stronger sandboxing.

---

# Security Model

Browsers must enforce strict security rules.

Examples:

same origin policy
content security policy
sandboxed iframes

Fenrir plans to implement additional security layers:

fenrir-security
fenrir-privacy

---

# Summary

A browser transforms web pages through several stages:

HTML
↓
DOM
↓
CSS styling
↓
layout
↓
display list
↓
GPU rendering

Fenrir relies on **Servo for rendering** while implementing its own infrastructure for networking, AI, and privacy.

This architecture allows Fenrir to focus on innovation without reimplementing the full browser stack.
