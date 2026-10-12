# WKWebView & wry Cross-Platform Quirks

Every row below is checked against the pinned dependency versions and against `src/main.rs` as it
stands. Where the app does not implement a mitigation, that is said: a quirk row is not a promise
about the code.

## macOS WKWebView

### Crashes

| Quirk | Symptom | Fix |
|-------|---------|-----|
| `drawCanvas.style.cursor = ...` from a synchronous handler | Segfault (EXC_BAD_ACCESS) | Wrap in `requestAnimationFrame()` — see `setCanvasCursor()` (`src/main.rs:1575-1579`), which covers every canvas cursor write |
| `document.body.style.cursor = ...` from a synchronous handler | Same hazard, and the app still does it | **Unmitigated.** `applyBodyCursor()` (`src/main.rs:1588`) assigns it synchronously at `1592` and `1605`, and runs from the document `mousemove` handler (`1845`), from `showControls()`/`hideControls()` (`1613`, `1619`) via `keydown -> showUI()` (`1765`), and from the shortcuts panel (`2613`, `2629`). No reason is recorded for why this is safe; TESTING.md 1.1 is where a macOS run would find out |
| `window.prompt()` / `alert()` / `confirm()` | Silent failure / crash on macOS 14+ | Use custom modal dialogs — see `#text-dialog` pattern (`src/main.rs:1485-1495`) |
| `evaluate_script` during an active user gesture | Deadlock or crash | **Neither call is deferred.** `src/main.rs:3211` (fullscreen state push) is entered straight from the Rust event loop on a `fullscreen` message (`3880`), and that message is posted by `requestFullscreen()` (`2575`), reached from the fullscreen button's `onclick` (`2578`) and from the `F`/`F11` keydown arms via `.click()` (`2736-2737`). So it is in flight during a user gesture. The update path (`3888`) is guarded by a different mechanism: an initialisation script with a pending flag (`3800`), not by `requestAnimationFrame` or `DOMContentLoaded` |

### Autoplay

WKWebView blocks autoplay of audio-containing video without a user gesture. What the app actually does:

- `<video id="v" playsinline></video>` — `playsinline` is the only attribute on the element (`src/main.rs:1126`); there is no `muted` or `autoplay` attribute
- The script sets `vid.muted = false` and `vid.volume = 1.0` (`1516-1517`) and calls `vid.play()` (`1518`), so audible autoplay is tried first on purpose. Only the rejected promise's `catch` (`1522-1525`) sets `vid.muted = true` and retries
- `.with_autoplay(true)` on the builder (`3805`)
- `--autoplay-policy=no-user-gesture-required` is added as an extra browser argument, **on Windows only** (`3807-3810`)

### Rendering

| Quirk | Workaround |
|-------|------------|
| `backdrop-filter` requires `-webkit-backdrop-filter` | **Not applicable here.** The app uses neither property; it reaches for opaque fills (`src/main.rs:61`, `background: #16161a`) because a translucent wash on this chrome has nothing to read against (`src/main.rs:58`) |
| `object-fit: contain` on `<video>` may letterbox incorrectly | `width: 100%` and `height: 100%` are both set with `object-fit: contain` (`src/main.rs:22-27`). Per the comment at `28-30`, `contain` is the only fit the file declares in every mode, and Fill mode gives the window the video's own shape instead, so there is no empty space for `contain` to letterbox |
| Canvas `getContext('2d')` differs in sub-pixel rendering | **No workaround in this app.** The 2D context is `src/main.rs:2746` and the drawing path (`2768-2809`, `drawShape`) rounds nothing. The only two `Math.round()` calls are window dimensions in `resizeShape()` (`1677`) and playback-rate quantisation in `stepRate()` (`2172`) |

## Linux (WebKitGTK)

| Quirk | Workaround |
|-------|------------|
| Building needs the GTK 3 generation of WebKitGTK | Install `libwebkit2gtk-4.1-dev`, **not** the 4.0 or the 6.0 package. The pinned chain is `wry 0.39.5` (`Cargo.lock:3514-3515`) → `webkit2gtk` / `webkit2gtk-sys 2.0.1` (`Cargo.lock:3110-3111`, `3134-3135`). `webkit2gtk-sys 2.0.1` declares `[package.metadata.system-deps.webkit2gtk_4_1] name = "webkit2gtk-4.1"`, so its build script's `system_deps::Config::new().probe()` asks pkg-config for `webkit2gtk-4.1` — which is exactly the `libwebkit2gtk-4.1-dev` package — and the same crate depends on `gtk-sys 0.18` (GTK 3) and `soup3-sys`. The 6.0 series is the GTK 4 generation, the one 4.1 was obsoleted by, and 4.0 is older still; neither is what this binding probes, so installing either satisfies nothing here and the build fails in `probe()` |
| — | There is no CI to check the package version in. This repository has no `.github/workflows` and no azure/gitlab/travis/appveyor/Jenkinsfile/circleci config. To check locally: `cargo metadata --format-version 1`, or read the `webkit2gtk-sys` entry in `Cargo.lock`, or just build and read the `system_deps` error, which names the `webkit2gtk-4.1` pkg-config module it could not find |
| CSP `media-src` restrictions may block localhost | Include `http://127.0.0.1:*` in the CSP — the policy at `src/main.rs:8` does, for `media-src` and `connect-src` |
| No sandbox by default | **Not applicable here.** The app is a standalone window: no sandbox configuration, no namespaces and no embedding path anywhere in `src/main.rs`. Namespace advice is only relevant if the webview is embedded in another app |

## Windows (WebView2)

| Quirk | Workaround |
|-------|------------|
| WebView2 runtime must be installed | **The installer does nothing about it.** `installer/DrPlayer.nsi` ships no Evergreen Bootstrapper, no fixed-version payload and no runtime presence check; its only WebView2 mentions are the uninstaller's profile deletion, a comment block (`397-410`) and the `RMDir /r /REBOOTOK "$INSTDIR\${APP_EXE}.WebView2"` directive under it (`411`). The app assumes a preinstalled Evergreen runtime |
| `windows_subsystem = "windows"` hides console (`src/main.rs:1`) | **There is no log file and no replacement.** The app's only diagnostic is the panic hook's `eprintln!("Dr.Player internal error: {}", info)` (`3710`). The attribute means the process gets no console *window* of its own, so a launch by double-clicking the exe shows nothing, but a process started from `cmd.exe` inherits that console's handles and the message is visible there, capturable with a `2>` redirection. No log file exists to catch it anywhere else: there is no `File::create`, no `OpenOptions` and no `writeln!` anywhere in `main.rs`. TESTING.md 5.1 covers the panic hook |
| DPI scaling affects `LogicalSize` vs `PhysicalSize` | Always use `LogicalSize` for window creation — `with_inner_size(tao::dpi::LogicalSize::new(inner_width, inner_height))` (`src/main.rs:3775`) and `let _ = window.set_inner_size(tao::dpi::LogicalSize::new(w, h));` (`3915`). The inner size there is the remembered size when one was read back from the state file and passed the checks above it, and the 1280×720 default otherwise. No `PhysicalSize` and no `ScaleFactorChanged` arm in the file |

## General wry/tao

- `window.drag_window()` on macOS must be called from a mouse-down handler. All three posts are from `mousedown`: `src/main.rs:1772`, `1776`, `1898`; the Rust arm is `3884-3885`
- Resize events from JS → Rust should be throttled via `requestAnimationFrame`. The drag path is (`src/main.rs:1833-1839`, behind a `resizePending` guard); `snapWindowToVideo()` posts directly at `1691` and does not, which is correct for a one-shot snap but means the throttle is not universal
- IPC `postMessage` is fire-and-forget on the JS side and arrives asynchronously in Rust: the handler forwards to `proxy.send_event` (`src/main.rs:3815-3817`) and the message is consumed in `event_loop.run` (`3840`)
- DevTools are off in **every** build, not just release ones: `.with_devtools(false)` is called unconditionally at `src/main.rs:3804`, with no `cfg!(debug_assertions)` anywhere. wry's `devtools` feature is also not enabled (`Cargo.toml:11` lists only `fullscreen`), so devtools are unavailable regardless
- The `fullscreen` feature (`Cargo.toml:11`) is enabled but inert at the pinned version: wry 0.39.5 defines `fullscreen = []` with no dependencies. Fullscreen in this app comes from tao's window (`src/main.rs:3874-3883`), never from a wry webview call, so no macOS requirement can be substantiated from that feature