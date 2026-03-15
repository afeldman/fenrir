# Servo Integration Architecture

This document describes how the **Fenrir browser integrates the Servo rendering engine**.

Fenrir embeds Servo as a rendering engine while providing its own infrastructure for:

* UI
* networking
* security
* AI
* browser state

The goal is to reuse Servo’s rendering pipeline without modifying Servo internals.

---

# Architectural Overview

Fenrir follows an **embedder architecture**.

Fenrir acts as the **browser shell**, while Servo handles rendering.

High level structure:

Fenrir UI
↓
Fenrir Engine
↓
Servo Embedder Layer
↓
Servo Engine
↓
WebRender
↓
wgpu
↓
GPU

Fenrir never modifies Servo directly.

All integration happens through the **fenrir-servo crate**.

---

# Fenrir Components Involved

Servo integration touches several crates.

fenrir-servo
fenrir-core
fenrir-network
fenrir-security (future)
fenrir-ui

Each has a specific role.

---

# fenrir-servo

The **fenrir-servo crate** is the bridge between Fenrir and Servo.

Responsibilities:

* initialize Servo
* create WebView instances
* manage navigation
* pass input events
* receive rendering events

The rest of Fenrir should not depend on Servo directly.

---

# Servo Embedder Model

Servo is designed to be embedded.

The embedder is responsible for:

* window creation
* event loop
* input handling
* navigation control
* permission decisions

Fenrir implements this role.

Core types used:

Servo
WebView
WebViewDelegate
EmbedderCallbacks

---

# WebView Concept

A **WebView** represents a rendered web page.

Conceptually it corresponds to a browser tab.

Fenrir maps:

Browser Tab → Servo WebView

Example:

Tab 1 → WebView 1
Tab 2 → WebView 2

WebViews are managed by the browser engine.

---

# Initialization Flow

When Fenrir starts:

1. load configuration
2. initialize logging
3. initialize networking
4. initialize Servo
5. create UI window

Example startup flow:

main()

↓

create FenrirEngine

↓

initialize Servo

↓

create first WebView

↓

navigate to start page

---

# Navigation Flow

Navigation follows this sequence.

User enters URL

↓

UI sends navigation event

↓

FenrirEngine processes request

↓

fenrir-network resolves URL

↓

Servo WebView loads page

↓

rendering pipeline starts

---

# Rendering Pipeline

Servo handles the complete rendering pipeline.

HTML

↓

DOM construction

↓

CSS style computation

↓

layout

↓

display list generation

↓

WebRender

↓

wgpu

↓

GPU frame

Fenrir does not interfere with these steps.

---

# Input Handling

Input originates from the OS.

Flow:

OS

↓

winit event

↓

Fenrir UI

↓

fenrir-servo

↓

Servo WebView

↓

DOM event dispatch

Examples:

mouse click
keyboard input
scroll events

---

# Event Handling

Servo generates events for the embedder.

Examples:

navigation started
page title changed
load finished
console messages

fenrir-servo converts these into Fenrir events.

Example:

Servo event

↓

fenrir-servo adapter

↓

Fenrir event bus

↓

UI update

---

# Permission System

Servo occasionally requests permission decisions.

Examples:

geolocation
notifications
file access
navigation

Fenrir will implement permission checks in:

fenrir-security

Flow:

Servo PermissionRequest

↓

fenrir-servo adapter

↓

fenrir-security decision

↓

grant or deny

↓

result returned to Servo

---

# Networking

Servo can perform network requests internally.

Fenrir may optionally intercept requests through:

fenrir-network

Possible responsibilities:

DNS resolution
HTTPS enforcement
request filtering
proxy support

This layer allows Fenrir to implement:

privacy protections
request auditing

---

# Tab Management

Each browser tab corresponds to a WebView.

Fenrir maintains a tab registry.

Example structure:

BrowserState

contains:

Vec<Tab>

Tab

contains:

WebView
URL
navigation state
history

---

# Window Integration

Fenrir uses:

winit + egui

Responsibilities:

window management
input events
drawing UI

Servo rendering output is presented inside the browser viewport.

---

# Security Model

Servo provides some isolation internally.

Fenrir adds additional layers.

Planned security layers:

fenrir-security
origin policies
URL validation
permission prompts

Security decisions must always be made by Fenrir.

Servo should never access sensitive system APIs directly.

---

# Error Handling

Servo errors are mapped into Fenrir error types.

Example:

ServoError

↓

FenrirError

This prevents Servo errors from leaking across crate boundaries.

---

# Debugging

Useful debugging tools include:

tracing logs
Servo debug flags
WebRender debug overlays

Fenrir debug panel exposes:

navigation events
render events
input events

---

# Future Improvements

Possible future improvements include:

process isolation per tab
stronger sandboxing
custom network interception
privacy enforcement

However Servo will remain the primary rendering engine.

---

# Design Principles

Servo integration follows strict rules.

Servo must remain an external dependency.

Fenrir must not modify Servo internals.

All Servo interaction must go through:

fenrir-servo

This ensures maintainability and easy Servo upgrades.

---

# Summary

Fenrir acts as a **browser shell** embedding Servo.

Fenrir provides:

UI
networking
security
AI
browser state

Servo provides:

HTML parsing
CSS styling
layout
DOM
GPU rendering

This separation allows Fenrir to innovate without reimplementing a full browser engine.
