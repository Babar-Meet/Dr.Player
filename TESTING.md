# Dr.Player — Manual QA Testing Guide

> **Target audience:** QA engineers & developers running manual regression tests.
> **Primary risk:** macOS (WKWebView) crashes from nested `.click()` dispatch, synchronous cursor
> mutation during keyboard events, and DOM timing races.
> Build with `cargo build --release` and run `./target/release/dr-player <video-file>`.

---

## Table of Contents

1. [macOS-specific test plan](#1-macos-specific-test-plan)
2. [Draw mode — step-by-step](#2-draw-mode--step-by-step)
3. [Keyboard shortcut conflicts — step-by-step](#3-keyboard-shortcut-conflicts--step-by-step)
4. [Cross-platform test matrix](#4-cross-platform-test-matrix)
5. [Crash / error resilience tests](#5-crash--error-resilience-tests)
6. [Edge cases & regression checklist](#6-edge-cases--regression-checklist)
7. [Quick smoke-test checklist (2-minute run)](#quick-smoke-test-checklist-2-minute-run)

---

## 1. macOS-specific test plan

WKWebView on macOS is the most crash-prone platform. Run **all** of the following on macOS
before every release. If you only have time for one platform test, test macOS.

### 1.1 Cursor mutation deferral (rAF)

| # | Step | Expected |
|---|------|----------|
| 1 | Enter draw mode. Move mouse over canvas. | Cursor is crosshair (or grab for Hand tool). |
| 2 | Press tool shortcut keys rapidly: `p`, `l`, `a`, `r`, `c`, `h` in sequence. | Cursor changes smoothly. No crash. |
| 3 | Exit draw mode (`Escape` or click ✕ Exit). | Cursor returns to default. |
| 4 | Enter draw mode again immediately. Repeat 10 times. | No crash. Cursor is always correct. |

### 1.2 Click dispatch replacement — draw toolbar

The draw toolbar dispatches real mouse clicks end to end. The player shortcuts do not: `.`, `,`,
`F`/`F11`, `M` and `L` still call `.click()` on the HUD buttons. Click the buttons themselves in
the steps below, so what is tested is the path that changed.

| # | Step | Expected |
|---|------|----------|
| 1 | Enter draw mode. Click a **color swatch**. | Color changes. No crash. |
| 2 | Click the **Pen** tool button in the toolbar (not the keyboard shortcut). | Tool switches. No crash. |
| 3 | Click **✕ Exit** button directly. | Exits draw mode cleanly. |
| 4 | Re-enter draw mode. Click the **-5s** seek button in the draw toolbar. | Video seeks backward 5 seconds. No crash. |
| 5 | Click the **+1F** step button. | Video time moves forward by exactly 1/60 s (16.7 ms) and the player stays paused. The step is a fixed time, not a frame: nothing reads the source frame rate, so on a 30 fps source it is half a frame and on a 24 fps source two fifths of one. No crash. |

### 1.3 Initialization script timing

| # | Step | Expected |
|---|------|----------|
| 1 | Launch the player with a video file from the command line. | Window opens. Video loads and starts playing (or shows first frame). Title bar shows the filename. |
| 2 | Close and re-launch 5 times with different video files. | No blank window. No "Error loading video" on valid files. |

---

## 2. Draw mode — step-by-step

### 2.1 Basic drawing operations

| # | Step | Expected |
|---|------|----------|
| 1 | Open any video. Click **✎** in the top bar. | Draw toolbar appears at bottom-center. Video pauses. |
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

A shape is only "selected" while a mouse button is held down on it. The selection is dropped on
mouse-up and on mouse-leave, so there is no persistent selection state to act on afterwards.

| # | Step | Expected |
|---|------|----------|
| 1 | Draw a shape. Select **Hand** (H). | Cursor changes to grab. |
| 2 | Press and hold the left mouse button on the shape. | Cursor changes to grabbing. The shape is picked up, and only for as long as the button is held. |
| 3 | Drag the shape. | Shape moves with mouse. |
| 4 | Release. | Shape stays at new position. Nothing stays selected. |
| 5 | Click on empty area. | Nothing happens. |
| 6 | Press and hold the left mouse button on the shape, then press **Delete** before letting go. | That one shape is removed, and the removal is undoable. Letting go first removes the selection, and a later **Delete** then does nothing at all. |
| 7 | Press **Backspace** (no modifier held). | **Every** shape is cleared, not just one. Undo (↩) brings them all back. |

### 2.4 Tool switching mid-draw

| # | Step | Expected |
|---|------|----------|
| 1 | Select **Line**. Click and start dragging a line. **While still dragging**, press `R`. | Drawing state resets. Any incomplete line (where start = end) is discarded. Tool switches to Rectangle. |
| 2 | Repeat with Pen: start drawing, press another tool key mid-stroke. | Incomplete single-point pen stroke is discarded. Tool switches. |
| 3 | Start drawing a line, release (complete). Then switch tool. | The completed line is retained. |
| 4 | Start a Pen stroke, draw a few points, release. Then switch tool. | The multi-point pen stroke is retained. |

### 2.5 Size & color

| # | Step | Expected |
|---|------|----------|
| 1 | Click a color swatch (e.g., Green). | Green becomes active color. Swatch is highlighted. |
| 2 | Draw a line. | Line is green. |
| 3 | Use size slider to increase to 10. | Subsequent draws are thicker. |
| 4 | Press `3` on keyboard (in draw mode). | Size becomes 8 (= 2 + 3*2). Slider updates. |

### 2.6 Drag toolbar

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
| 1 | While NOT in draw mode, press `p`, `l`, `a`, `r`, `c`, `h`. | Nothing switches tool. Outside draw mode `l` toggles loop (see 3.1/7), `c` reshapes the window and `h` hides the controls. |
| 2 | Enter draw mode. Press `p`. | Tool = Pen. |
| 3 | Press `l`. | Tool = Line. |
| 4 | Press `a`. | Tool = Arrow. |
| 5 | Press `r`. | Tool = Rectangle. |
| 6 | Press `c`. | Tool = Circle. |
| 7 | Press `h`. | Tool = Hand. |
| 8 | Press `1` through `9` in draw mode. | Size changes accordingly. |
| 9 | Press `0` in draw mode. | Size = 2. |

### 3.3 Escape behavior

| # | Step | Expected |
|---|------|----------|
| 1 | Go fullscreen (F11 or F). Press **Escape**. | Exits fullscreen. |
| 2 | Enter draw mode. Press **Escape**. | Exits draw mode. The player handler returns early while the draw bar is open, so this does not also leave fullscreen. |
| 3 | Exit draw mode while fullscreen. Press **Escape**. | Exits fullscreen normally. |

### 3.4 Modifier keys in draw mode

| # | Step | Expected |
|---|------|----------|
| 1 | In draw mode, select **Hand** (H), press and hold the left mouse button on a shape, then press **Delete** before letting go. | The held shape is removed. **Do not release first:** the selection is dropped on mouse-up, so a **Delete** pressed after releasing does nothing. |
| 2 | In draw mode, press **Backspace**, with no modifier held. | All shapes are cleared, and the clearing is undoable. |
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
| Frame step (±1F, fixed 1/60 s — not the source frame rate) | ✓ | ✓ | ✓ |
| Volume (slider, arrows, M) | ✓ | ✓ | ✓ |
| Fullscreen (button, F, F11, Escape) | ✓ | **Must test** - WKWebView fullscreen transitions | ✓ |
| Window drag (title bar) | ✓ | ✓ | ✓ |
| Window resize (edges) | ✓ | ✓ | ✓ |
| Minimize / Close | ✓ | ✓ | ✓ |
| **Draw mode: enter/exit** (✎ button, Escape or ✕ Exit) | ✓ | **Must test** - crash risk | ✓ |
| **Draw mode: tool switch mid-draw** | ✓ | **Must test** - crash risk | ✓ |
| **Draw mode: cursor changes** | ✓ | **Must test** - crash risk (rAF fix) | ✓ |
| **Mouse buttons 3/4 for undo/redo** | ✓ | ✓ | ✓ |
| Undo/redo buttons | ✓ | ✓ | ✓ |
| Clear canvas | ✓ | ✓ | ✓ |
| Keyboard shortcut isolation | ✓ | **Must test** - ensure no cross-talk | ✓ |
| Loop toggle (L) | ✓ | ✓ | ✓ |
| Toolbar drag | ✓ | ✓ | ✓ |
| Rapid mode switching (draw ↔ play 20×) | ✓ | **Must test** | ✓ |
| **Error resilience** (see §5) | ✓ | **Must test** | ✓ |
| Window blur while drawing | ✓ | ✓ | ✓ |

### 4.1 Repetition stress test (macOS critical)

Run this sequence **20 times** without restarting the app:

1. Play video → pause → enter draw mode
2. Draw a few shapes (mix of tools)
3. Switch tools mid-draw twice
4. Undo twice, redo once
5. Exit draw mode with Escape
6. Toggle fullscreen
7. Seek around

If there is no crash after 20 iterations, the macOS fixes are stable.

---

## 5. Crash & error resilience tests

### 5.1 Rust panic hook

What the code does: the panic hook prints `Dr.Player internal error: {info}` to stderr
(`src/main.rs:3081-3082`). That line is the only diagnostic this app emits, anywhere, on every
platform. Nothing else logs, so its absence is a result and not a fault.

Two honest limits on seeing it. The binary is a GUI-subsystem process
(`#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]`, `src/main.rs:1`), so it gets no
console **window** of its own, and it writes no log file. A launch by double-clicking the exe therefore
shows nothing at all, whatever the app is doing, and there is no file to open afterwards. The stderr
handle itself is still valid, though: a GUI-subsystem process started from a console inherits that
console's handles, so a launch from a console can read what the hook printed.

| # | Step | Expected |
|---|------|----------|
| 1 | Use the app normally through one full draw-mode pass and the smoke checklist. | No visible crash, hang, or lost window. |
| 2 | From `cmd.exe`, run `target\release\dr-player.exe <video-file>`. Watch the terminal, or put `2> err.txt` on the end of the same command line to capture stderr into a file instead of watching it scroll. | stderr stays empty. An empty `err.txt` means no panic was caught on that run. Do not report the hook as broken on the basis of an empty file. |
| 3 | Repeat step 2 from `cmd.exe` while working the app through the 4.1 sequence, 20 iterations of draw, undo, redo, tool switch, fullscreen and seek, again with `2> err.txt`. | `err.txt` is empty, meaning no panic was caught across the run. A line starting `Dr.Player internal error:` means a panic was caught, and the text after it names the source location. |

Run step 2 and step 3 from `cmd.exe`, not from PowerShell. A `2>` redirection in PowerShell can
produce an empty file for a GUI-subsystem process, which then reads as "no panic" when it means "not
captured". Watching the terminal without a redirection works in either shell.

The hook only adds that line to stderr. It does not change what a panic does to the process, so a
caught panic still ends the run and the window still disappears: the message and the disappearance are
one event, not two faults.

### 5.2 Invalid inputs

| # | Step | Expected |
|---|------|----------|
| 1 | Resize the window while drawing. | Canvas resizes. Shapes are re-rendered. No crash. |
| 2 | Hide the HUD with the **eye icon** (title bar, left of the fill button) then enter draw mode. | Draw mode still works. |
| 3 | Rapidly click the Draw button (✎) on/off as fast as possible. | No crash. |

---

## 6. Edge cases & regression checklist

Run these after every code change to catch regressions fast.

- [ ] **Single-point click (no drag)** with every tool — does it create a zero-size shape that switchTool cleans up?
- [ ] **Pen with 1 point** — the code duplicates the point to draw a dot; verify it renders.
- [ ] **Canvas mousedown while video is loading** (no duration) — no crash.
- [ ] **Blur event** while drawing — `window.blur` resets drawing state. Verify this doesn't leave dangling state.
- [ ] **Loop off by default** — verify the loop button shows "Off" state, and the `ended` event does NOT replay.
- [ ] **Loop on** — toggle on, let video end. Verify it restarts from 0.
- [ ] **Volume = 0** — icon changes to mute. Click the icon again: volume returns to **50%**, not to the level you had before, because driving the slider (or `ArrowDown`) to 0 overwrites the stored level with 0. Muting with the icon or `M` instead leaves the stored level alone, so that path *does* return to the previous level.
- [ ] **Mouse leave canvas during draw** — `mouseleave` handler sets `drawing = false`. Verify stroke is not extended when mouse re-enters.
- [ ] **Resize corner cursors** — verify the correct resize cursors appear at window edges (nwse, nesw, ns, ew).
- [ ] **Undo/redo after clear** — clear canvas, then undo. Shapes should reappear.
- [ ] **Backspace in draw mode**, no modifier held — clears all shapes and stays undoable.
- [ ] **Fullscreen toggle in draw mode** — should be blocked (draw mode hides fullscreen). Verify closing draw mode then pressing F11 works.

---

## Quick smoke-test checklist (2-minute run)

For a fast regression check between builds:

```
[ ] Launch — video plays
[ ] Enter draw mode (✎ button) — toolbar visible, video pauses
[ ] Draw a line (L), a rect (R), a circle (C)
[ ] Start a line (L), drag, then press R before releasing — tool switches, the line is kept
[ ] Select Pen (P), press and hold on the canvas without moving, then press L before releasing — the one-point stroke is discarded
[ ] Undo (↩) — shape removed
[ ] Redo (↪) — shape back
[ ] Exit draw mode (Escape or ✕ Exit)
[ ] Press Space — play/pause works
[ ] Press L — loop toggles
[ ] Fullscreen (F or F11)
[ ] Escape from fullscreen
[ ] Close window (✕)
```

If all pass, do the full matrix above. 🟢
