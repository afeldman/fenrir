# Web Compatibility

This document tracks **web compatibility issues affecting the Fenrir browser**.

Fenrir uses the **Servo rendering engine**, therefore many compatibility issues originate from missing or incomplete web platform features in Servo.

The goal of this document is to:

* identify missing Web APIs
* track failing modern websites
* prioritize upstream Servo contributions

---

# Compatibility Philosophy

Fenrir aims to support **modern websites without custom hacks**.

Compatibility improvements should be implemented by:

1. implementing missing Web APIs
2. improving Servo standards compliance
3. contributing fixes upstream

Fenrir should avoid site-specific patches whenever possible.

---

# Compatibility Layers

Modern websites depend on several browser capabilities:

HTML parsing
DOM APIs
CSS features
JavaScript runtime
Web APIs
network behavior

Servo already implements most HTML and CSS features, but **several modern Web APIs are incomplete or missing**.

---

# Priority Compatibility Targets

These websites are used as **real-world compatibility benchmarks**.

| Website     | Status        | Notes                         |
| ----------- | ------------- | ----------------------------- |
| DuckDuckGo  | ⚠️ partial    | requires IntersectionObserver |
| GitHub      | ⚠️ partial    | dynamic content loading       |
| Wikipedia   | ✅ works       | mostly static                 |
| Hacker News | ✅ works       | simple HTML                   |
| Google Docs | ❌ unsupported | complex JS platform           |
| YouTube     | ❌ unsupported | heavy JS + media              |

The goal is to gradually increase compatibility.

---

# Major Missing APIs

Several APIs are critical for modern web frameworks.

---

# IntersectionObserver

Specification:

https://w3c.github.io/IntersectionObserver/

Purpose:

Detect when elements enter or leave the viewport.

Used for:

lazy loading
virtualized lists
scroll-based animations

Frameworks depending on it:

React
Next.js
Vue

Servo status:

partially implemented

Tracking issue:

https://github.com/servo/servo/issues/35767

Key missing feature:

pending initial intersectionobserver targets

Impact:

Without IntersectionObserver many modern pages fail to load correctly.

Priority:

HIGH

---

# ResizeObserver

Specification:

https://drafts.csswg.org/resize-observer/

Purpose:

Observe element size changes.

Used for:

responsive layouts
dynamic UI components

Servo status:

missing

Impact:

Moderate compatibility issues.

Priority:

MEDIUM

---

# MutationObserver

Specification:

https://dom.spec.whatwg.org/

Purpose:

Observe DOM changes.

Used heavily by:

React
Angular
dynamic UI frameworks

Servo status:

partially implemented

Priority:

MEDIUM

---

# Navigation API

Specification:

https://wicg.github.io/navigation-api/

Purpose:

modern navigation handling.

Used by:

Next.js
modern SPAs

Servo status:

not implemented

Priority:

LOW

---

# View Transitions API

Specification:

https://developer.mozilla.org/en-US/docs/Web/API/View_Transitions_API

Purpose:

smooth page transitions.

Servo status:

not implemented

Priority:

LOW

---

# Framework Compatibility

Modern frameworks rely on specific browser APIs.

---

## React

React depends on:

MutationObserver
IntersectionObserver
modern DOM APIs

Servo status:

partial compatibility.

---

## Next.js

Next.js requires:

IntersectionObserver
Navigation API
modern fetch APIs

Without IntersectionObserver the framework may crash.

---

## Vue

Vue compatibility is mostly good but depends on:

MutationObserver
DOM APIs

---

# Debugging Compatibility Issues

Typical debugging workflow:

1. load site in Fenrir
2. open developer console
3. inspect JavaScript errors
4. identify missing API

Example error:

ReferenceError: IntersectionObserver is not defined

Next step:

check Servo implementation status.

---

# Servo Contribution Workflow

When a compatibility issue is discovered:

1. confirm missing API
2. check existing Servo issue
3. implement missing feature in Servo fork
4. run Web Platform Tests
5. submit upstream PR

---

# Web Platform Tests

Servo relies on **WPT** to ensure standards compliance.

Run tests:

```bash
./mach test-wpt
```

Run specific tests:

```bash
./mach test-wpt -f IntersectionObserver
```

Passing WPT tests is required before submitting PRs.

---

# Compatibility Tracking

Compatibility issues should be tracked in a table.

Example:

| Feature              | Status  | Servo Issue | Priority |
| -------------------- | ------- | ----------- | -------- |
| IntersectionObserver | partial | #35767      | HIGH     |
| ResizeObserver       | missing | none        | MEDIUM   |
| MutationObserver     | partial | existing    | MEDIUM   |

---

# Fenrir Testing Strategy

Fenrir should maintain a set of **test websites**.

Example list:

lite.duckduckgo.com
github.com
news.ycombinator.com
wikipedia.org

These sites help verify real-world compatibility.

---

# Long Term Goal

The goal is for Fenrir to run **modern websites without compatibility patches**.

This will be achieved by:

improving Servo standards compliance
implementing missing Web APIs
contributing fixes upstream

Improving Servo benefits the entire Rust browser ecosystem.

---

Related documents:

docs/browser-internals.md
docs/rendering-pipeline.md
docs/servo-integration.md
docs/browser-process-model.md
