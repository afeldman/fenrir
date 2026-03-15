# docs/current — Implementierte Architektur

Diese Dokumente beschreiben **was heute tatsächlich existiert und läuft**.

| Dokument | Inhalt | Status |
|---|---|---|
| [browser-internals.md](browser-internals.md) | Kernkomponenten: UI, Engine, DOM, Storage | ✅ Implementiert |
| [rendering-pipeline.md](rendering-pipeline.md) | HTML → DOM → Layout → GPU über Servo + WebRender | ✅ Implementiert |
| [servo-integration.md](servo-integration.md) | Servo-Wrapper, WebView, FenrirHost-Delegate | ✅ Implementiert |
| [network-architecture.md](network-architecture.md) | HTTP-Stack, DNS, TLS (rustls), Interceptor | ✅ Implementiert |
| [web-compatibility.md](web-compatibility.md) | Bekannte Servo-Kompatibilitätsprobleme, Missing APIs | ✅ Aktives Issue-Tracking |
| [browser-process-model.md](browser-process-model.md) | Aktuell: Single-Process; Multi-Process: Phase 4 | ✅ Ist-Stand |
