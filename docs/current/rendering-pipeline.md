# Fenrir Rendering Pipeline

This document describes the **rendering pipeline used by the Fenrir browser**.

Fenrir relies on the **Servo rendering engine** and **WebRender GPU renderer**.

---

# Overview

Rendering transforms a web page into pixels on the screen.

Fenrir rendering pipeline:

HTML
↓
Servo DOM
↓
CSS Style System
↓
Layout Engine
↓
Display List
↓
WebRender
↓
wgpu
↓
GPU

---

# Step 1: HTML Fetch

The process begins with a URL request.

fenrir-network performs:

DNS lookup
HTTPS request
response streaming

The HTML document is passed to Servo.

---

# Step 2: HTML Parsing

Servo parses the HTML document.

Example input:

<html>
<body>
<h1>Hello</h1>
</body>
</html>

Output:

DOM tree.

Parsing happens incrementally while the document downloads.

---

# Step 3: DOM Construction

Servo constructs a DOM tree representing the document.

Each node corresponds to:

element
text node
attribute

Example structure:

Document
└── html
└── body
└── h1

---

# Step 4: CSS Processing

CSS rules are parsed and matched against DOM nodes.

Example:

h1 {
font-size: 32px;
}

Servo computes **computed styles** for each node.

This includes:

font
color
positioning
display rules

---

# Step 5: Layout

The layout engine computes geometry.

Outputs:

position
width
height

Layout supports:

block layout
flexbox
grid

Result:

Layout tree.

Only visible nodes appear in the layout tree.

---

# Step 6: Display List Generation

Servo converts the layout tree into a **display list**.

Display list entries include:

draw rectangle
draw text
draw image
draw border

This list is independent of GPU APIs.

---

# Step 7: Scene Building

WebRender converts the display list into a **scene graph**.

The scene graph organizes render primitives.

Examples:

text primitives
image primitives
clip regions

---

# Step 8: GPU Commands

WebRender generates GPU draw commands.

These commands are sent to:

wgpu

wgpu abstracts the graphics backend.

Supported backends:

Vulkan
Metal
DirectX12
OpenGL

---

# Step 9: Frame Rendering

The GPU executes the draw commands.

Output:

Frame buffer

This frame is presented to the window.

The UI framework (winit + egui) displays the final frame.

---

# Frame Lifecycle

Each frame goes through:

layout
display list generation
scene building
GPU rendering

Fenrir updates frames when:

user input occurs
layout changes
animations run
scroll events occur

---

# Input Handling

User input enters through:

winit

Events include:

mouse movement
mouse click
keyboard input
scroll wheel

Events are forwarded to Servo.

Servo dispatches them as DOM events.

---

# Performance Considerations

Servo and WebRender are designed for GPU efficiency.

Key techniques:

GPU batching
parallel style computation
incremental layout
texture caching

These reduce CPU overhead and improve frame rate.

---

# Future Rendering Improvements

Possible future work includes:

custom GPU compositor
AI-assisted rendering optimization
GPU-driven layout experiments

However Fenrir will continue to rely on Servo as the primary rendering engine.

---

# Debugging Rendering

Common debugging tools include:

log tracing
Servo debug flags
WebRender debug overlays

The Fenrir debug panel (F12) exposes:

render events
input events
blit status

This helps diagnose rendering problems.

---

# Summary

Fenrir rendering pipeline:

Network fetch
↓
Servo HTML parser
↓
DOM construction
↓
CSS style computation
↓
layout
↓
display list
↓
WebRender
↓
wgpu
↓
GPU frame

This architecture allows Fenrir to leverage a modern GPU-based renderer while keeping the browser infrastructure written in Rust.
