# Dr.Player — Manual QA Testing Guide

> **Target audience:** QA engineers & developers running manual regression tests.
> **Primary risk:** macOS (WKWebView) crashes from `window.prompt()`, nested `.click()` dispatch,
> synchronous cursor mutation during keyboard events, and DOM timing races.
> Build with `cargo build --release` and run `./target/release/dr-player <video-file>`.

---

## Table of Contents

1. [macOS-specific test plan](#1-macos-specific-test-plan)
2. [Draw mode — step-by-step](#2-draw-mode--step-by-step)
3. [Keyboard shortcut conflicts — step-by-step](#3-keyboard-shortcut-conflicts--step-by-step)
4. [Cross-platform test matrix](#4-cross-platform-test-matrix)
5. [Crash / error resilience tests](#5-crash--error-resilience-tests)
6. [Edge cases & regression checklist](#6-edge-cases--regression-checklist)

---

## 1. macOS-specific test plan

WKWebView on macOS is the most crash-prone platform. Run **all** of the following on macOS
before every release. If you only have time for one platform test, test macOS.

### 1.1 Prompt replacement (text dialog)

| # | Step | Expected |
|---|------|----------|
| 1 | Open a video. Press `D` (or click ✎) to enter draw mode. | Draw toolbar appears, video pauses. |
| 2 | Click the **Text** tool (or press `T`), then click anywhere on the canvas. | A styled dialog appears: "Enter text:" with an input field, OK and Cancel buttons. **No native `window.prompt()` alert.** |
| 3 | Type "Hello World" and press **Enter** (or click **OK**). | Dialog closes. Text "Hello World" is rendered on the canvas at the clicked position. |
| 4 | Click the canvas again with the Text tool, then press **Escape** (or click **Cancel**). | Dialog closes. No text is added. |
| 5 | Click the Text tool, then click the canvas. Before typing, click the **Pen** tool. | Dialog should remain open (it is a separate overlay). Close it by pressing Escape. The tool should switch to Pen. |
| 6 | Rapidly: Enter draw mode, click Text tool, click canvas, type, OK, click again, Escape. Do this 10 times. | No crash. Dialog always opens and closes cleanly. |

### 1.2 Cursor mutation deferral (rAF)

| # | Step | Expected |
|---|------|----------|
| 1 | Enter draw mode. Move mouse over canvas. | Cursor is crosshair (or grab for Hand tool). |
| 2 | Press tool shortcut keys rapidly: `p`, `l`, `a`, `r`, `c`, `h` in sequence. | Cursor changes smoothly. No crash. |
| 3 | Exit draw mode (`E` or click ✕ Exit). | Cursor returns to default. |
| 4 | Enter draw mode again immediately. Repeat 10 times. | No crash. Cursor is always correct. |

### 1.3 Click dispatch replacement

| # | Step | Expected |
|---|------|----------|
| 1 | Enter draw mode. Click a **color swatch**. | Color changes. No crash. |
| 2 | Click the **Pen** tool button in the toolbar (not the keyboard shortcut). | Tool switches. No crash. |
| 3 | Click **✕ Exit** button directly. | Exits draw mode cleanly. |
| 4 | Re-enter draw mode. Click the **-5s** seek button in the draw toolbar. | Video seeks backward 5 seconds. No crash. |
| 5 | Click the **+1F** step button. | Video advances 1 frame. No crash. |

### 1.4 Initialization script timing

| # | Step | Expected |
|---|------|----------|
| 1 | Launch the player with a video file from the command line. | Window opens. Video loads and starts playing (or shows first frame). Title bar shows the filename. |
| 2 | Close and re-launch 5 times with different video files. | No blank window. No "Error loading video" on valid files. |

---

## 2. Draw mode — step-by-step

### 2.1 Basic drawing operations

| # | Step | Expected |
|---|------|----------|
| 1 | Open any video. Press `D` (or click ✎). | Draw toolbar appears at bottom-center. Video pauses. |
| 2 | Verify toolbar has: Pen (P), Line (L), Arrow (A), Rectangle (R), Circle (C), Hand (H), color swatches, size slider, undo/redo, clear, ±5s, ±1F, Exit. | All buttons visible. |
| 3 | Select **Pen** (P). Click and drag on canvas. | A freehand line is drawn following the mouse. |
| 4 | Release mouse. | Drawing stops. Shape is committed. |
| 5 | Select **Line** (L). Click and drag. | A straight line appears from start to current position. |
| 6 | Release. | Line is committed. |
| 7 | Select **Rectangle** (R). Click and drag diagonally. | Rectangle outline appears, growing/shrinking with mouse. |
| 8 | Release. | Rectangle is committed. |
| 9 | Select **Circle** (C). Click and drag. | Circle appears with center at click point, radius following mouse. |
| 10 | Release. | Circle is committed. |
| 11 | Select **Arrow** (A). Click and drag. | Arrow line with arrowhead at endpoint appears. |
| 12 | Release. | Arrow is committed. |

### 2.2 Undo / Redo

| # | Step | Expected |
|---|------|----------|
| 1 | Draw 3 shapes (e.g., line, rect, circle). | All visible. |
| 2 | Click **Undo** (↩) or press mouse button 3 (back). | Last shape (circle) disappears. |
| 3 | Click **Undo** again. | Rectangle disappears. |
| 4 | Click **Redo** (↪) or press mouse button 4 (forward). | Rectangle reappears. |
| 5 | Draw a new shape. | Redo stack is cleared. Cannot redo the earlier undone shape. |
| 6 | Undo until empty. | All shapes removed. Undo button does nothing on empty stack. |
| 7 | Redo until empty. | All shapes restored. Redo button does nothing on empty stack. |

### 2.3 Hand tool — select & move

| # | Step | Expected |
|---|------|----------|
| 1 | Draw a shape. Select **Hand** (H). | Cursor changes to grab. |
| 2 | Click on the shape. | Shape is selected. Cursor changes to grabbing. |
| 3 | Drag the shape. | Shape moves with mouse. |
| 4 | Release. | Shape stays at new position. |
| 5 | Click on empty area. | Nothing happens (no shape selected). |
| 6 | While a shape is selected, press **Delete** or **Backspace**. | Shape is removed. |

### 2.4 Text tool

| # | Step | Expected |
|---|------|----------|
| 1 | Select **Text** tool (click data-tool="text" button or... note: no keyboard shortcut; click the toolbar button). | Cursor changes to crosshair. |
| 2 | Click on canvas. | Dialog opens with text input. |
| 3 | Type text, press Enter. | Text appears at clicked position. |
| 4 | In Hand mode, click on the text. | Text is selected. |
| 5 | Scroll mouse wheel. | Text font size changes (larger/smaller). |
| 6 | Drag the text. | Text moves. |

### 2.5 Tool switching mid-draw

| # | Step | Expected |
|---|------|----------|
| 1 | Select **Line**. Click and start dragging a line. **While still dragging**, press `R`. | Drawing state resets. Any incomplete line (where start = end) is discarded. Tool switches to Rectangle. |
| 2 | Repeat with Pen: start drawing, press another tool key mid-stroke. | Incomplete single-point pen stroke is discarded. Tool switches. |
| 3 | Start drawing a line, release (complete). Then switch tool. | The completed line is retained. |
| 4 | Start a Pen stroke, draw a few points, release. Then switch tool. | The multi-point pen stroke is retained. |

### 2.6 Size & color

| # | Step | Expected |
|---|------|----------|
| 1 | Click a color swatch (e.g., Green). | Green becomes active color. Swatch is highlighted. |
| 2 | Draw a line. | Line is green. |
| 3 | Use size slider to increase to 10. | Subsequent draws are thicker. |
| 4 | Press `3` on keyboard (in draw mode). | Size becomes 8 (= 2 + 3*2). Slider updates. |

### 2.7 Drag toolbar

| # | Step | Expected |
|---|------|----------|
| 1 | Hover over the left edge (⠿ handle) of the draw toolbar. | Cursor changes to grab. |
| 2 | Click and drag the handle. | Toolbar follows the mouse. |
| 3 | Release. | Toolbar stays at the new position. |

---

## 3. Keyboard shortcut conflicts — step-by-step

### 3.1 Global shortcuts bail out in draw mode

| # | Step | Expected |
|---|------|----------|
| 1 | Play a video. Press **Space**. | Video toggles play/pause. |
| 2 | Enter draw mode. Press **Space**. | **Nothing happens.** Video does not toggle. |
| 3 | Exit draw mode. Press **Space**. | Video toggles again. |
| 4 | In draw mode, press **Arrow Left/Right**. | **Nothing happens.** No seeking. |
| 5 | In draw mode, press **Arrow Up/Down**. | **Nothing happens.** No volume change. |
| 6 | In draw mode, press **L**. | Tool switches to Line. **Loop does not toggle.** |
| 7 | Exit draw mode. Press **L**. | Loop toggles (button highlights). |

### 3.2 Draw mode shortcuts only fire in draw mode

| # | Step | Expected |
|---|------|----------|
| 1 | While NOT in draw mode, press `p`, `l`, `a`, `r`, `c`, `h`, `e`. | Nothing happens. (Note: `l` toggles loop — see 3.1/6.) |
| 2 | Enter draw mode. Press `p`. | Tool = Pen. |
| 3 | Press `l`. | Tool = Line. |
| 4 | Press `a`. | Tool = Arrow. |
| 5 | Press `r`. | Tool = Rectangle. |
| 6 | Press `c`. | Tool = Circle. |
| 7 | Press `h`. | Tool = Hand. |
| 8 | Press `e`. | Exits draw mode. |
| 9 | Press `1` through `9` in draw mode. | Size changes accordingly. |
| 10 | Press `0` in draw mode. | Size = 2. |

### 3.3 Escape behavior

| # | Step | Expected |
|---|------|----------|
| 1 | Go fullscreen (F11 or F). Press **Escape**. | Exits fullscreen. |
| 2 | Enter draw mode. Press **Escape**. | **Nothing happens.** Does not exit fullscreen. Does not exit draw mode. (Use `E` or the Exit button to exit draw mode.) |
| 3 | Exit draw mode while fullscreen. Press **Escape**. | Exits fullscreen normally. |

### 3.4 Modifier keys in draw mode

| # | Step | Expected |
|---|------|----------|
| 1 | In draw mode, select a shape with Hand tool. Press **Delete**. | Selected shape is removed. |
| 2 | In draw mode, press **Ctrl+Backspace** (or **Cmd+Backspace** on macOS). | All shapes are cleared. |
| 3 | In draw mode, press mouse button 3 (back) on canvas while NOT drawing. | Undoes last shape. |
| 4 | In draw mode, press mouse button 4 (forward) on canvas while NOT drawing. | Redoes last undone shape. |
| 5 | In draw mode, **start drawing** (click and drag). While dragging, press mouse button 3. | **Nothing happens.** Undo is blocked while drawing. |

---

## 4. Cross-platform test matrix

Test every cell in this matrix before a release.

| Feature | Windows (WebView2) | macOS (WKWebView) | Linux (WebKitGTK) |
|---------|-------------------|-------------------|-------------------|
| Launch & load video | Stress-test with 20 files | Stress-test with 20 files | Verify works |
| Play/Pause (Space, button) | ✓ | ✓ | ✓ |
| Seek (±5s buttons, arrows) | ✓ | ✓ | ✓ |
| Frame step (±1F) | ✓ | ✓ | ✓ |
| Volume (slider, arrows, M) | ✓ | ✓ | ✓ |
| Fullscreen (button, F, F11, Escape) | ✓ | **Must test** - WKWebView fullscreen transitions | ✓ |
| Window drag (title bar) | ✓ | ✓ | ✓ |
| Window resize (edges) | ✓ | ✓ | ✓ |
| Minimize / Close | ✓ | ✓ | ✓ |
| **Draw mode: enter/exit** (button, D, E) | ✓ | **Must test** - crash risk | ✓ |
| **Draw mode: tool switch mid-draw** | ✓ | **Must test** - crash risk | ✓ |
| **Draw mode: cursor changes** | ✓ | **Must test** - crash risk (rAF fix) | ✓ |
| **Text dialog** (type, Enter, Escape) | ✓ | **Must test** - crash risk (prompt replacement) | ✓ |
| **Mouse buttons 3/4 for undo/redo** | ✓ | ✓ | ✓ |
| Undo/redo buttons | ✓ | ✓ | ✓ |
| Clear canvas | ✓ | ✓ | ✓ |
| Keyboard shortcut isolation | ✓ | **Must test** - ensure no cross-talk | ✓ |
| Loop toggle (L) | ✓ | ✓ | ✓ |
| Toolbar drag | ✓ | ✓ | ✓ |
| Rapid mode switching (draw ↔ play 20×) | ✓ | **Must test** | ✓ |
| **Error resilience** (see §5) | ✓ | **Must test** | ✓ |
| Window blur while drawing | ✓ | ✓ | ✓ |

### Repetition stress test (macOS critical)

Run this sequence **20 times** without restarting the app:

1. Play video → pause → enter draw mode
2. Draw a few shapes (mix of tools)
3. Switch tools mid-draw twice
4. Undo twice, redo once
5. Enter text via dialog, OK and Cancel
6. Exit draw mode
7. Toggle fullscreen
8. Seek around

If there is no crash after 20 iterations, the macOS fixes are stable.

---

## 5. Crash & error resilience tests

### 5.1 Global error handlers

| # | Step | Expected |
|---|------|----------|
| 1 | Open DevTools console (if available) or enable logging. | |
| 2 | In draw mode, evaluate in console: `throw new Error("test crash")` | Error is logged as `GLOBAL_ERROR` but does NOT crash the app. |
| 3 | Evaluate: `Promise.reject("test rejection")` | Warning logged as `UNHANDLED_PROMISE` but app continues. |

### 5.2 Rust panic hook

| # | Step | Expected |
|---|------|----------|
| 1 | If you can inject a Rust panic (e.g., via an invalid IPC message), verify it prints "Dr.Player internal error: ..." to stderr and the app does not immediately crash. | Graceful message to console. |
| 2 | Under normal use, verify no panics occur. | |

### 5.3 Invalid inputs

| # | Step | Expected |
|---|------|----------|
| 1 | In draw mode, call `switchTool("")` via console. | No crash. Tool is set to empty string (unexpected but not crashing). |
| 2 | Set size slider to 0 (hack via console: `document.getElementById('csize').value = 0`). Draw. | Size 0 still works (min should be 1). |
| 3 | Resize the window while drawing. | Canvas resizes. Shapes are re-rendered. No crash. |
| 4 | Hide the HUD (◎ button) then enter draw mode. | Draw mode still works. |
| 5 | Rapidly click the Draw button (✎) on/off as fast as possible. | No crash. |

---

## 6. Edge cases & regression checklist

Run these after every code change to catch regressions fast.

- [ ] **Single-point click (no drag)** with every tool — does it create a zero-size shape that switchTool cleans up?
- [ ] **Pen with 1 point** — the code duplicates the point to draw a dot; verify it renders.
- [ ] **Canvas mousedown while video is loading** (no duration) — no crash.
- [ ] **Blur event** while drawing — `window.blur` resets drawing state. Verify this doesn't leave dangling state.
- [ ] **Text dialog + draw mode conflict** — verify clicking canvas while dialog is open does NOT start a new draw.
- [ ] **Loop off by default** — verify the loop button shows "Off" state, and the `ended` event does NOT replay.
- [ ] **Loop on** — toggle on, let video end. Verify it restarts from 0.
- [ ] **Volume = 0** — icon changes to mute. Click again to unmute at previous level.
- [ ] **Mouse leave canvas during draw** — `mouseleave` handler sets `drawing = false`. Verify stroke is not extended when mouse re-enters.
- [ ] **Resize corner cursors** — verify the correct resize cursors appear at window edges (nwse, nesw, ns, ew).
- [ ] **Undo/redo after clear** — clear canvas, then undo. Shapes should reappear.
- [ ] **Ctrl+Backspace in draw mode** — clears all shapes. Verify it does NOT navigate back in browser.
- [ ] **Fullscreen toggle in draw mode** — should be blocked (draw mode hides fullscreen). Verify closing draw mode then pressing F11 works.

---

## Quick smoke-test checklist (2-minute run)

For a fast regression check between builds:

```
[ ] Launch — video plays
[ ] Enter draw mode (D) — toolbar visible, video pauses
[ ] Draw a line (L), a rect (R), a circle (C)
[ ] Switch to Pen mid-line (P) — incomplete line cleaned up
[ ] Undo (↩) — shape removed
[ ] Redo (↪) — shape back
[ ] Text dialog — type and OK, then Cancel
[ ] Exit draw mode (E)
[ ] Press Space — play/pause works
[ ] Press L — loop toggles
[ ] Fullscreen (F or F11)
[ ] Escape from fullscreen
[ ] Close window (✕)
```

If all pass, do the full matrix above. 🟢
