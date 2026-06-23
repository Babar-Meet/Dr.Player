Dr.Player v0.2.0

macOS Crash Fixes (Critical)
- WKWebView crash on cursor mutation - drawCanvas.style.cursor now deferred via requestAnimationFrame
- Nested DOM event dispatch - replaced .click() calls with direct switchTool() function
- window.prompt() crash - replaced with custom HTML text dialog (prompt() not implemented in wry's WKWebView)
- Autoplay blocked on macOS - added playsinline muted autoplay to video element + .with_autoplay(true)
- Fullscreen crash - added features = ["fullscreen"] to wry dependency (required on macOS)
- evaluate_script race - switched to with_initialization_script + DOMContentLoaded listener

State Machine & Input Fixes
- Tool switching mid-draw now cleans up incomplete shapes
- Tool click handler delegates to switchTool() - no duplicated logic
- Mouse button 3/4 (back/forward) no longer triggers undo during active drawing
- Global keyboard shortcuts bail out when draw mode is active - prevents KeyL/KeyM conflicts
- Escape exits draw mode; e key removed to prevent accidental close
- Removed dead variables (drawStartX, drawStartY)
- Fixed dual mouseup handler state bypass

Error Resilience
- Global JS error handlers (window.onerror, unhandledrejection)
- Rust panic hook for graceful crash handling
- Removed double-throttled resize cooldown

Testing Infrastructure
- 73 regression tests in ui/tests/player.test.js (Vitest + jsdom)
- Manual QA guide in TESTING.md
- 13 test video files in TESTING_videos/ (MP4, WebM, AVI, MOV, MKV)
- Code review checklist at .github/PULL_REQUEST_TEMPLATE.md
- Platform quirks docs at docs/WEBVIEW_QUIRKS.md
- Pre-commit hook at .githooks/pre-commit
