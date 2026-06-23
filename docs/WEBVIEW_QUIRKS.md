# WKWebView & wry Cross-Platform Quirks

## macOS WKWebView

### Crashes

| Quirk | Symptom | Fix |
|-------|---------|-----|
| `style.cursor = 'crosshair'` in synchronous event handler | Segfault (EXC_BAD_ACCESS) | Wrap in `requestAnimationFrame()` — see `setCanvasCursor()` |
| `window.prompt()` / `alert()` / `confirm()` | Silent failure / crash on macOS 14+ | Use custom modal dialogs — see `#text-dialog` pattern |
| `evaluate_script` during active touch/mouse event | Deadlock or crash | Defer via `requestAnimationFrame` or `DOMContentLoaded` |

### Autoplay

WKWebView blocks autoplay of audio-containing video without user gesture.
- Always add `playsinline muted autoplay` to `<video>` elements
- Call `.with_autoplay(true)` on `WebViewBuilder`

### Rendering

| Quirk | Workaround |
|-------|------------|
| `backdrop-filter` requires `-webkit-backdrop-filter` | Always use both properties |
| `object-fit: contain` on `<video>` may letterbox incorrectly | Set both `width` and `height` to 100% |
| Canvas `getContext('2d')` differs in sub-pixel rendering | Use `Math.round()` for coordinate alignment |

## Linux (WebKitGTK)

| Quirk | Workaround |
|-------|------------|
| Requires `libwebkit2gtk-4.1-dev` (not 4.0) | Verify package version in CI |
| No sandbox by default | Use Linux namespaces if embedding |
| CSP `media-src` restrictions may block localhost | Include `http://127.0.0.1:*` in CSP |

## Windows (WebView2)

| Quirk | Workaround |
|-------|------------|
| WebView2 runtime must be installed | Use Evergreen Bootstrapper or Fixed Version |
| `windows_subsystem = "windows"` hides console | Write logs to file instead |
| DPI scaling affects `LogicalSize` vs `PhysicalSize` | Always use `LogicalSize` for window creation |

## General wry/tao

- `window.drag_window()` on macOS must be called from a mouse-down event handler
- Resize events from JS → Rust should be throttled via `requestAnimationFrame`
- IPC `postMessage` is synchronous from JS but arrives asynchronously in Rust
- `WebViewBuilder::with_devtools(false)` for release builds
- The `fullscreen` feature must be enabled for macOS: `wry = { features = ["fullscreen"] }`
