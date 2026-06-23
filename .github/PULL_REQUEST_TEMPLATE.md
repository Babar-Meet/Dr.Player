## Dr.Player Code Review Checklist

### General
- [ ] Does `cargo build` succeed without warnings?
- [ ] Does `cargo clippy` pass with no warnings?
- [ ] Does `cargo test` pass?
- [ ] Has the change been tested on at least one other platform than the reviewer's?
- [ ] Is the diff under 400 lines? If not, can it be broken into smaller PRs?

### wry/tao Specific
- [ ] Any `evaluate_script` call — could the JS callback re-enter Rust?
- [ ] Any `postMessage` from JS — is the Rust handler idempotent?
- [ ] Any cursor mutation — wrapped in `requestAnimationFrame`? (macOS WKWebView crash mitigation)
- [ ] Any `window.prompt`, `alert`, or `confirm` calls — replaced with custom dialog?
- [ ] Any `mousedown`/`mouseup` on the WebView — are `preventDefault`/`stopPropagation` correct?
- [ ] Any resizing — throttled via `requestAnimationFrame`?

### macOS-Specific
- [ ] No synchronous `evaluate_script` in event callback that triggers DOM events
- [ ] All `alert`/`confirm`/`prompt` replaced with custom JS UI
- [ ] CSS uses `-webkit-` prefixes where needed (backdrop-filter, user-select)
- [ ] No reliance on `beforeunload`

### Cross-Platform
- [ ] Path handling uses `std::path::PathBuf`, not string concatenation
- [ ] `.gitattributes` normalizes line endings
- [ ] Window icon has fallback when `.ico` fails
- [ ] HiDPI and standard DPI both accounted for

### Security
- [ ] Local HTTP server binds only to `127.0.0.1`
- [ ] Random token ≥ 16 alphanumeric chars
- [ ] CSP restricts `media-src` and `connect-src` to `127.0.0.1:*`
- [ ] IPC handler does not eval or execute arbitrary code
