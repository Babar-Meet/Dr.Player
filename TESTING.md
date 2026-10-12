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
7. [Resume position, window memory and frame stepping](#7-resume-position-window-memory-and-frame-stepping)
8. [Quick smoke-test checklist (2-minute run)](#quick-smoke-test-checklist-2-minute-run)

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
| 5 | Click the **+1F** step button. | Video time moves forward by one frame of the file and the player stays paused. The step is the frame interval measured from the video itself, so about 1/60 s (16.7 ms) on a 60 fps source, 1/30 s (33.3 ms) at 30 fps and 1/24 s (41.7 ms) at 24 fps. It is a time and not a frame boundary, so the engine decodes from whatever frame is nearest. See §7.5. No crash. |

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
| 8 | Hold a shape down with the **Hand** tool and scroll the mouse wheel over it. Then release and scroll again. | The held shape's stroke resizes one notch at a time, between 1 and 40 — wider than the 1 to 20 the size slider in 2.5 can reach, so a held shape can be drawn thicker than the toolbar can set. The resize is **not on the undo stack**: an undo taken after it steps back past the draw that made the shape, not past the resize. After the release the wheel does **nothing**, because nothing is held. |

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
| Frame step (±1F, the source's own frame interval — see §7.5) | ✓ | ✓ | ✓ |
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
(`src/main.rs:3736-3737`). That line is the only diagnostic this app emits, anywhere, on every
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

## 7. Resume position, window memory and frame stepping

Three features arrived together and none of them had a checklist here, so this section is the whole
test story for them. All three are about something surviving a launch, so leaving and relaunching the
app is part of nearly every step.

They share one state file: `state.json` stored directly in the executable's directory (portable persistence, with automatic migration from `%APPDATA%\Dr.Player\state.json` on Windows, `~/Library/Application Support/Dr.Player/state.json` on macOS, and `~/.config/Dr.Player/state.json` on Linux if an older state was present). It holds the window's remembered geometry, global hold-to-speed preference, and one playback position per file. The app reads it once, at launch, so it can be opened in a text editor between runs to set up an exact case.

Three facts the steps below lean on:

- **The time readout is whole seconds.** The formatter floors, so nothing on screen shows a fraction.
  Where a step needs a particular stored position, write it into the state file rather than trying to
  stop the video on the second.
- **A position is written when playback pauses, and by the ✕ button in the top bar**, which writes
  before it closes. Alt+F4, the taskbar close button, and a session that simply ends do **not** write
  one. So the way to leave a position behind is to press Space, or to close with ✕.
- **Escape does not touch the resume chip.** It asks the window to leave fullscreen and nothing else.
  §7.2 is where that is checked.

Fixture lengths and rates below were measured from the files in `TESTING_videos/`, not read off their
names. This reproduces them:

```
ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate \
        -show_entries format=duration -of default=nw=1 <file>
```

| File | Rate | Length | Offer band (5 s to 90%) |
|------|------|--------|-------------------------|
| `rabbit320.mp4`, `rabbit320.webm` | 30 fps | 7.800 s | 5 s – 7.02 s (narrow) |
| `sample_640x360.mkv` | 29.97 fps | 13.346 s | 5 s – 12.01 s |
| `sample_1280x720.mov` | 23.976 fps | 28.237 s | 5 s – 25.41 s (widest) |
| `flower.mp4`, `flower.webm` | 29.97 fps | 5.055 s | **empty** — never offered |
| `test-vp9.webm` | 24 fps | 2.000 s | **empty** — never offered |
| `elephants_dream_1024x576_30s.mp4` | 24 fps | 653.792 s | 5 s – 588.41 s |

### 7.1 Resume offer thresholds

| # | Step | Expected |
|---|------|----------|
| 1 | Launch a file that has no stored position. | Video plays from 0:00 and **no chip appears**. |
| 2 | On `rabbit320.mp4`, stop at about 00:00:03, close with ✕, relaunch. | **No chip.** Under `RESUME_MIN_SECONDS = 5` there is nothing worth resuming to, and four seconds of a file is noise over the opening. |
| 3 | On the same file, stop at about 00:00:06, close with ✕, relaunch. | Chip appears with **one** button on it, reading **Continue from 00:00:06**. There is no second control and nothing to refuse with. |
| 4 | Stop that file past 00:00:07.02 — 90 per cent of 7.8 s — close with ✕, relaunch. | **No chip.** Past `RESUME_END_FRACTION = 0.9` somebody opening it again is asking to watch it afresh. |
| 5 | Repeat steps 3 and 4 on `sample_1280x720.mov`, stopping at 20 s and at 26 s. | Chip at 20 s, no chip at 26 s. The band scales with the file; it is not a fixed number of seconds from the end. |
| 6 | Launch `flower.mp4` or `test-vp9.webm` with a stored position inside what would otherwise be the band. | **No chip, always.** Their 90 per cent mark (4.55 s and 1.80 s) is below the 5 s minimum, so no position on either file can ever be offered. This is the two rules meeting, and it is a refusal rather than a bug. |
| 7 | Hand-edit a stored position to a number past the file's duration, and separately to one for a file whose recorded byte size no longer matches the file on disk. Relaunch each time. | **No chip, and no error either time.** A position that does not belong to the file that is actually open is not offered. |
| 8 | Open the offer and watch that the video is **not** paused and the chip has **not** taken focus. | The video keeps playing. Pressing Space while the chip is up does not pause it. |

### 7.2 The one answer, what is not one, and what Escape is not

The chip carries a single button. Everything else that takes it off the screen is the same action as
the ten-second timeout: hide the chip, change nothing else. None of them clears the stored position
and none of them stops the file recording one, because a file reaching its end is the only thing that
does either.

| # | Step | Expected |
|---|------|----------|
| 1 | Relaunch onto a file with an offer. Click **Continue from …**. | The video jumps to the saved position and **keeps playing**. Nothing pauses. The position stays stored, because the video really is at that position. |
| 2 | Relaunch onto a file with an offer, and let the ten seconds run with no input at all (see §7.3). | The chip goes away. Playback is **left exactly where it was** — no seek, no pause — and nothing is cleared or written. |
| 3 | Continue as in step 1, watch on to about 00:00:20, pause, close with ✕, and relaunch. | The chip offers 00:00:20. Taking the offer is not refusing it, so this file keeps recording normally. |
| 4 | Relaunch onto a file with an offer and click anywhere on the picture while it is up. | The chip goes away, playback is **left exactly where it was** — it does not seek to 0:00 — and the stored position is still there. |
| 5 | Repeat step 4, then watch on to about 00:00:25, pause, close with ✕, and relaunch. | **The chip offers 00:00:25.** A click is not a refusal either: the file was still being watched, so where it was left is still worth having. |
| 6 | With the chip up, go fullscreen (F or F11), then press **Escape**. | **Only** fullscreen is left. **The chip is still on screen**, and the stored position is untouched. Escape does one thing in this app and that is it; it never touches the controls. |
| 7 | With the chip up and the video **not** fullscreen, press **Escape**. | **Nothing at all happens.** No chip dismissed, no playback touched, nothing written. |
| 8 | Tab to the Continue button on the chip and press **Enter**. | The same as clicking it: the video jumps to the saved position and keeps playing. **Space** on that focused button also activates it and does **not** pause the video — the chip stops keys so one press cannot do both. |
| 9 | Let a file play all the way to its end. | The stored position is cleared at the end, and this is the **only** thing that clears one: the button, the timeout and a click on the picture all leave it standing. |
| 10 | Turn **loop** on and watch to the end. | The file replays from 0, and **no position is written at all**, whether or not the chip was ever offered. |

### 7.3 The chip's own timeout

| # | Step | Expected |
|---|------|----------|
| 1 | Launch `sample_1280x720.mov` with a stored position of 20 s, so there is film left to watch while the chip is up. | Chip appears. |
| 2 | Touch nothing and count from the moment it appears. | It is gone on its own after **10 s** (`RESUME_TIMEOUT_MS`). |
| 3 | Let it time out, watch on to about 00:00:25, pause, close with ✕, and relaunch. | The chip offers 00:00:25. Timing out is not a dismissal: the video was still being watched, so where it was left is still worth having. |
| 4 | While the chip is up, move the mouse and press keys. | The chrome reappears as it always does. The chip's timer is a timeout and is **not** reset by this. |
| 5 | With the chip up, press nothing at all and let the ten seconds run, then close with ✕ straight away without pausing, and open `state.json` (next to `dr-player.exe`) in a text editor. | The entry for this file is **byte for byte the one you wrote before closing last time**: same `t`, same `at`. The timeout hid the chip and reached nothing else — it did not clear the entry and it did not rewrite it. |

### 7.4 Window size and position memory

| # | Step | Expected |
|---|------|----------|
| 1 | Resize the window to something ordinary, move it, wait about a second, then close with ✕. Relaunch. | Same size, same position. |
| 2 | Drag a resize corner down to a few hundred pixels across. | The window goes as small as you take it. **Nothing clamps a live resize**, and that is deliberate: a very small window is the picture-in-picture substitute. |
| 3 | Close at that small size and relaunch. | The window opens at the **default 1280×720**, not at the small size. A remembered size under 200×120 (`MIN_RESTORED_WIDTH` / `MIN_RESTORED_HEIGHT`) is refused, because a window that small could not hold the 48 px top bar and the 41 px bottom row without the two overlapping — and with no system chrome, those are the only way the window can be moved or resized. |
| 4 | Hand-edit `"window"` in the state file to a size that is not a size — `"w": 1e30`, a negative, or a missing side — and relaunch. | The app **opens at the default**, with no error and no dialog. A size read back from the file goes through the same `is_postable_size` test a live resize goes through. |
| 5 | Hand-edit `"window"` to a real but tiny size such as 100×80, and relaunch. | Default again, for the reason in step 3. |
| 6 | Move the window so the saved rectangle would sit entirely off any monitor — `"x": -5000, "y": -5000` — and relaunch. | The window opens **centred on the primary monitor at the same size**. The position is dropped; the size is kept. |
| 7 | Move the window so it hangs half off the right-hand edge, and relaunch. | **Restored where it was.** The guard is that the whole rectangle still fits a monitor that is connected now, not that every pixel is on screen. |
| 8 | Unplug or disable the monitor the window was left on, and relaunch on what is left. | It opens on a connected monitor rather than off-screen. |
| 9 | Go fullscreen, quit, and relaunch. | Opens **windowed**, at the remembered size. Fullscreen is never remembered. |
| 10 | Turn on fill mode (C), reshape the window, close, and relaunch. | The fill-mode shape is not remembered either. The remembered size is the last one the owner gave the window itself. |
| 11 | Replace the state file with the text `not json`, then with an empty file, then delete it, relaunching each time. | The app opens normally every time, remembering nothing. None of the three is a reason not to open, and none is anything the owner did wrong. |
| 12 | Make the config directory read-only, then launch and close. | It opens and closes normally. A write that fails is dropped; there is no dialog and no complaint. |

### 7.5 Frame stepping at more than one frame rate

The step is the file's own frame interval, measured from the video as it plays. That makes it
directly countable: **how many presses of `+1F` it takes to move the readout on by one second is the
frame rate**, because the readout advances once per `1/rate` presses. Count presses on a paused video,
where every press is one step.

| # | Step | Expected |
|---|------|----------|
| 1 | Open `sample_1280x720.mov` (23.976 fps), pause, and count presses of **+1F** until the readout ticks over by one second. | **24 presses.** 23.976 is within 2 per cent of 24, so it is read as exactly 1/24. |
| 2 | Repeat on `rabbit320.mp4` (30 fps). | **30 presses.** |
| 3 | Repeat on `flower.mp4` and on `sample_640x360.mkv` (29.97 fps). | **30 presses.** 29.97 is within 2 per cent of 30 and is read as exactly 1/30. |
| 4 | Repeat on `elephants_dream_1024x576_30s.mp4` (24 fps, and 653 s long, so it is the easiest one to count in). | **24 presses.** |
| 5 | Repeat using the `,` and `.` keys, and again on the two step buttons in the draw toolbar. | The same count every time. All four call sites move by the same interval. |
| 6 | Repeat on a 25, 50, 60 or 120 fps clip. | **25, 50, 60 and 120 presses** respectively. No fixture at these rates ships in `TESTING_videos/`; make one with `ffmpeg -i in.mp4 -r 50 -c:v libx264 -an out.mp4`, and confirm what you built by running the `ffprobe` line above against it. |
| 7 | Compare every count above against 60. | **60 at every rate is the bug this replaced**, not the expected result. If every file you open takes 60 presses per second, the measurement is not running and the step has fallen back to the old fixed 1/60 s. |
| 8 | Step backwards with `,` from 0:00, and forwards past the end of a short file. | Clamped at 0:00 and at the duration. Neither end throws and neither wraps. |
| 9 | Confirm the video is paused after each step and did not start itself again. | It stays paused. A step is a look at a still frame. |

Two honest limits on this section. It measures the **size of the step**, which is what the feature
sets; it does not measure whether the engine landed on a frame boundary, because `currentTime` is a
position in seconds and the engine decodes from whatever frame is nearest. And the fallback branch
cannot be reached from the UI on WebView2, WKWebView or WebKitGTK, since all three deliver
measurements; if you ever do land on an engine that does not, the symptom is step 7 — 60 presses
everywhere.

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
