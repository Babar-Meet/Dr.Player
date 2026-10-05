## Dr.Player Code Review Checklist

**Nothing in this checklist is automated.** There is no CI in this repository: `.github/` holds only
this template, there is no `.github/workflows`, and no azure/gitlab/travis/appveyor/Jenkinsfile/circleci
config anywhere in the tree. `.githooks/pre-commit` exists but is not installed (`git config --get
core.hooksPath` is empty) and, as written, would reject every commit; treat every box below as
something a human did by hand.

The shipped `.githooks/pre-commit` attempts three things and nothing else: a tab-character check over
`src/`, a `prompt|alert|confirm` grep over `src/main.rs`, and `cargo check`. It runs no clippy, no
build-warning check, no tests, and none of the wry/tao, macOS, cross-platform or security items below.
So every box here is manual-only, including the ones a hook happens to overlap.

### General
- [ ] Does `cargo build` succeed without warnings? (manual)
- [ ] Does `cargo clippy --all-targets -- -D warnings` pass? (manual)
      The gate is `--all-targets` with `-D warnings`; bare `cargo clippy` lints less.
      **This does not pass today**: `cargo clippy --all-targets` emits
      `warning: this let-binding has unit value --> src\main.rs:3211:33` (`clippy::let_unit_value`),
      so under `-D warnings` it is an error. Either the warning is fixed or the box is unticked.
- [ ] Worked the manual guide in `TESTING.md` for the behaviour this change touches: section 1 for
      macOS, section 2 for draw mode, section 3 for shortcut conflicts, section 4 for the
      cross-platform matrix, section 5 for crash resilience, plus the 2-minute smoke list at the end.
      There is no `cargo test` to run — no `#[test]`, no `#[cfg(test)]`, no `tests/` directory, no
      dev-dependencies; `cargo test` reports `0 passed` and says nothing about the change. (manual)
- [ ] If you changed anything platform-shaped, note that only Windows is a shipped target: the installer
      is NSIS, `windows_subsystem` is set on Windows only, and the non-Windows
      `open_in_default_browser` is a compile-time fallback (`src/main.rs:3068-3076`). There is no
      macOS or Linux build job, no CI and no cross-compilation setup, so cross-platform testing is
      whatever the author could run locally. Say which platform you actually ran on. (manual)
- [ ] Is the diff under 400 lines? If not, can it be broken into smaller PRs? (manual)

### wry/tao Specific
- [ ] Any `evaluate_script` call — could the JS callback re-enter Rust? (manual)
- [ ] Any `postMessage` from JS — is the Rust handler idempotent?
      Idempotency is required of the absolute arms: `resize:w:h` (`src/main.rs:3206-3214`) sets a size,
      and `close`, `minimize`, `drag_window`, `update_available`, `open_release_page` (`3184-3205`)
      act absolutely, so a duplicate does the same thing twice. The `fullscreen` arm (`3188-3194`) is a
      **toggle** and is meant to be: a duplicated message flips it back. Do not "fix" that. (manual)
- [ ] Any canvas cursor mutation — wrapped in `requestAnimationFrame`? (`setCanvasCursor()`,
      `src/main.rs:1508-1512`, is the compliant writer). **Body cursor writes are not wrapped**:
      `applyBodyCursor()` assigns `document.body.style.cursor` synchronously and is called from
      `mousemove`, from `showControls()`/`hideControls()`, and from the shortcuts panel. Whether that
      is safe on macOS is untested — TESTING.md 1.1 is the place it would show. (manual)
- [ ] Any `window.prompt`, `alert`, or `confirm` calls — replaced with custom dialog? (manual)
- [ ] Any `mousedown`/`mouseup` on the WebView — are `preventDefault`/`stopPropagation` correct?
      The rule is written out at `src/main.rs:1802-1812`. (manual)
- [ ] Any resizing — throttled via `requestAnimationFrame`? The drag path is (`1761-1767`); the snap
      post at `1624` is a deliberate one-shot and is not throttled. (manual)

### macOS-Specific
- [ ] No synchronous `evaluate_script` in event callback that triggers DOM events. Both existing calls
      are from the Rust event loop (`2898`, `3202`), though `2898` is entered while a user gesture is
      still in flight. (manual)
- [ ] All `alert`/`confirm`/`prompt` replaced with custom JS UI (manual)
- [ ] CSS uses `-webkit-` prefixes where needed (`user-select` at `src/main.rs:17-18` is the live
      example; `backdrop-filter` is named in older checklists but the app does not use it) (manual)
- [ ] No reliance on `beforeunload` (manual)

### Cross-Platform
- [ ] Path handling uses `std::path::PathBuf`, not string concatenation (manual)
- [ ] `.gitattributes` normalizes line endings — **the repository has no `.gitattributes`.** This can
      only be answered "no" until someone adds one, so treat it as a proposal: if your change needs
      line-ending normalisation, add the file in the same PR. (manual)
- [ ] Window icon has fallback when `.ico` fails — there is no fallback asset. `load_icon()`
      (`src/main.rs:2878-2883`) degrades to `Option::None` and `with_window_icon` accepts it, so a decode
      failure means no icon rather than a different one. (manual)
- [ ] HiDPI and standard DPI both accounted for: `LogicalSize` throughout (`src/main.rs:3124`, `3211`),
      no `PhysicalSize`, no `ScaleFactorChanged` arm. (manual)

### Security
The first four and the last item describe the app as it stands today; they are here so a change that
breaks one is caught by a human. The CSP item is a review prompt, not a pass.

- [ ] Local HTTP server binds only to `127.0.0.1` (`src/main.rs:3102`)
- [ ] Random token ≥ 16 alphanumeric chars (`src/main.rs:3095-3099`)
- [ ] CSP at `src/main.rs:8` reviewed in full, not just the loopback part. What it is:
      `default-src 'self' 'unsafe-inline' http://127.0.0.1:*; media-src http://127.0.0.1:*;
      connect-src http://127.0.0.1:*; script-src 'unsafe-inline';`. So `media-src` and
      `connect-src` are loopback-only (correct), but the policy also carries
      `script-src 'unsafe-inline'` and an `'unsafe-inline'` inside `default-src` — under CSP2 that
      `'unsafe-inline'` in `default-src` is ignored, so inline script is permitted by `script-src`, not
      by `default-src`. There is no `object-src`, no `base-uri` and no `frame-ancestors`.
- [ ] IPC handler does not eval or execute arbitrary code: the handler is a string comparison chain
      (`src/main.rs:3184-3214`) and the only `evaluate_script` calls (`2898`, `3202`) format a boolean
      literal and a fixed string
- [ ] The page is handed **no URL**. It receives only the message `open_release_page`; the shell opens
      the compile-time constant `RELEASES_PAGE` (`src/main.rs:2903-2905`, `3203-3205`). Any change that
      gives the page a URL, or that makes that address anything but a constant, is a finding
- [ ] The bearer token is placed in the page URL and handed to the webview through an initialisation
      script (`src/main.rs:3130-3133`). Keep it out of anything the page can read back and out of the
      IPC message bodies