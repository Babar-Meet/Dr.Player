#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

const HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Dr.Player</title>
<meta http-equiv="Content-Security-Policy" content="default-src 'self' 'unsafe-inline' http://127.0.0.1:*; media-src http://127.0.0.1:*; connect-src http://127.0.0.1:*; script-src 'unsafe-inline';">
<style>
*, *::before, *::after { margin:0; padding:0; box-sizing:border-box; }
html, body {
    width: 100vw; height: 100vh;
    background: #000;
    overflow: hidden;
    font-family: 'Segoe UI', system-ui, -apple-system, sans-serif;
    color: #fff;
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
}

video {
    position: absolute;
    inset: 0;
    width: 100%; height: 100%;
    object-fit: contain;
}
/* contain above is the only fit this file declares, in every mode. Fill mode never touches
   it: it gives the window the video's own shape instead, so there is no empty space left
   for contain to letterbox and nothing is cropped or stretched to get there. */

/* ==================== TOP BAR ==================== */
#topbar {
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    z-index: 20;
    opacity: 0;
    transition: opacity 0.25s;
    pointer-events: none;
}
#topbar.show {
    opacity: 1;
    pointer-events: auto;
}

.drag-handle {
    position: absolute;
    inset: 0;
    cursor: grab;
    z-index: 1;
}

.title-capsule {
    background: rgba(255,255,255,0.12);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    border: 1px solid rgba(255,255,255,0.15);
    border-radius: 10px;
    padding: 6px 16px;
    font-size: 14px;
    font-weight: 500;
    color: rgba(255,255,255,0.95);
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
    max-width: 60%;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    position: relative;
    z-index: 2;
    pointer-events: auto;
}

.winctrl {
    display: flex;
    gap: 8px;
    z-index: 2;
    pointer-events: auto;
}
.wbtn {
    width: 28px; height: 28px;
    border-radius: 50%;
    background: rgba(255,255,255,0.1);
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.12);
    color: rgba(255,255,255,0.8);
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s;
    box-shadow: 0 2px 6px rgba(0,0,0,0.2);
}
.wbtn:hover {
    background: rgba(255,255,255,0.22);
    color: #fff;
}
.wbtn.close:hover {
    background: #e81123;
    border-color: #e81123;
    color: #fff;
}

/* ==================== BOTTOM HUD – SINGLE ROW ==================== */
#hud {
    position: absolute;
    inset: 0;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.25s;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    padding: 0 24px 18px;
    z-index: 10;
}
#hud.show {
    opacity: 1;
    pointer-events: auto;
}

.bar {
    background: rgba(255,255,255,0.08);
    backdrop-filter: blur(18px);
    -webkit-backdrop-filter: blur(18px);
    border: 1px solid rgba(255,255,255,0.12);
    border-radius: 12px;
    padding: 8px 14px;
    box-shadow: 0 8px 24px rgba(0,0,0,0.5);
}

/* Left to right, in the markup: play/pause, the two seek steps, the two frame steps, the elapsed
   time, the seek bar, the remaining time, volume, loop, fullscreen. Play/pause leads because every
   desktop player put side by side puts it left of the timeline and none puts it after, and the
   steps sit next to it. The row is plain flex and uses no order, so this markup is the visual order
   and the focus order as well; a CSS order would put one of the three out of step with the others. */
.controls-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
}

.seek-time {
    font-family: 'SF Mono', 'Consolas', monospace;
    font-size: 12px;
    color: rgba(255,255,255,0.9);
    min-width: 62px;
    text-align: center;
    white-space: nowrap;
}

.seekbar-container {
    flex: 1;
    height: 24px;
    display: flex;
    align-items: center;
    cursor: pointer;
    position: relative;
}
.seek-track {
    width: 100%;
    height: 4px;
    background: rgba(255,255,255,0.2);
    border-radius: 2px;
    position: relative;
    transition: height 0.12s;
}
.seekbar-container:hover .seek-track {
    height: 6px;
}
.seek-fill {
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 0%;
    background: #e00;
    border-radius: 2px;
    pointer-events: none;
}
.seek-thumb {
    position: absolute;
    top: 50%;
    transform: translate(-50%, -50%);
    width: 12px; height: 12px;
    background: #fff;
    border-radius: 50%;
    left: 0%;
    opacity: 0;
    transition: opacity 0.1s;
    pointer-events: none;
}
.seekbar-container:hover .seek-thumb {
    opacity: 1;
}

.btn {
    background: rgba(255,255,255,0.1);
    border: 1px solid rgba(255,255,255,0.15);
    color: rgba(255,255,255,0.9);
    border-radius: 7px;
    padding: 4px 8px;
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s;
    display: flex;
    align-items: center;
    justify-content: center;
    white-space: nowrap;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
    min-width: 34px;
    height: 28px;
}
.btn:hover {
    background: rgba(255,255,255,0.2);
    border-color: rgba(255,255,255,0.25);
    color: #fff;
    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}
.btn:active {
    transform: scale(0.95);
    background: rgba(255,255,255,0.14);
}

.vol-container {
    display: flex;
    align-items: center;
    gap: 5px;
    position: relative;
}
.vol-icon {
    font-size: 15px;
    opacity: 0.8;
    cursor: pointer;
    transition: opacity 0.1s;
    z-index: 1;
}
.vol-icon:hover { opacity: 1; }

.vol-slider {
    width: 60px;
    height: 3px;
    background: rgba(255,255,255,0.2);
    border-radius: 2px;
    cursor: pointer;
    position: relative;
    overflow: hidden;
    transition: height 0.12s;
}
.vol-slider:hover { height: 5px; }

.vol-fill {
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 100%;
    background: #fff;
    border-radius: 2px;
    pointer-events: none;
}

.fs-btn svg {
    width: 14px; height: 14px;
    fill: currentColor;
    display: block;
}

.btn-loop {
    display: flex;
    align-items: center;
    justify-content: center;
}
.btn-loop .loop-svg {
    width: 14px; height: 14px;
    display: block;
}
.btn-loop.active {
    background: rgba(255,80,80,0.3);
    border-color: rgba(255,100,100,0.5);
    color: #fff;
    box-shadow: 0 0 8px rgba(255,80,80,0.3);
}

/* ==================== DRAWING OVERLAY ==================== */
#stage {
    position: absolute;
    inset: 0;
}
#draw {
    position: absolute;
    inset: 0;
    width: 100%; height: 100%;
    pointer-events: none;
    z-index: 5;
    touch-action: none;
}
#draw.active {
    pointer-events: auto;
    cursor: crosshair;
}
body.drawmode #topbar,
body.drawmode #hud {
    display: none !important;
}
#drawbar {
    position: fixed;
    bottom: 24px; left: 50%; transform: translateX(-50%);
    display: none;
    flex-direction: row;
    align-items: stretch;
    z-index: 30;
    background: rgba(0,0,0,0.82);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 12px;
    padding: 5px 8px 5px 3px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.6);
    pointer-events: auto;
    gap: 0;
}
#drawbar.open {
    display: flex;
}
.drawbar-body {
    display: flex;
    flex-direction: column;
    gap: 3px;
}
.drow {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
}
.dbtn svg {
    width: 14px;
    height: 14px;
    display: block;
}
.dhandle svg {
    width: 20px;
    height: 42px;
    display: block;
}
.dbtn .dkey {
    font-size: 9px;
    opacity: 0.55;
    margin-left: 2px;
}
.dbtn {
    background: rgba(255,255,255,0.1);
    border: 1px solid rgba(255,255,255,0.12);
    color: rgba(255,255,255,0.85);
    border-radius: 6px;
    padding: 4px 6px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.12s;
    white-space: nowrap;
    line-height: 1;
    font-family: inherit;
}
.dbtn:hover {
    background: rgba(255,255,255,0.2);
    color: #fff;
}
.dbtn.active {
    background: rgba(255,80,80,0.35);
    border-color: rgba(255,100,100,0.5);
    color: #fff;
}
.dsep {
    width: 1px;
    height: 20px;
    background: rgba(255,255,255,0.15);
    margin: 0 4px;
    flex-shrink: 0;
}
.dhandle {
    cursor: grab;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4px 10px;
    background: rgba(255,255,255,0.05);
    border: none;
    border-right: 1px solid rgba(255,255,255,0.08);
    color: rgba(255,255,255,0.3);
    border-radius: 10px 4px 4px 10px;
    flex-shrink: 0;
    transition: all 0.15s;
    margin-right: 5px;
    min-width: 20px;
}
.dhandle:hover {
    color: rgba(255,255,255,0.7);
    background: rgba(255,255,255,0.1);
}
.dhandle:active {
    cursor: grabbing;
    background: rgba(255,255,255,0.15);
}
.cswatch {
    width: 18px; height: 18px;
    border-radius: 50%;
    border: 2px solid rgba(255,255,255,0.2);
    cursor: pointer;
    transition: transform 0.1s, border-color 0.1s;
    flex-shrink: 0;
}
.cswatch:hover {
    transform: scale(1.2);
}
.cswatch.sel {
    border-color: #fff;
    transform: scale(1.15);
}
#cpicker {
    width: 22px; height: 22px;
    padding: 0;
    border: 2px solid rgba(255,255,255,0.2);
    border-radius: 50%;
    cursor: pointer;
    background: none;
    flex-shrink: 0;
}
#cpicker::-webkit-color-swatch-wrapper { padding: 0; }
#cpicker::-webkit-color-swatch { border: none; border-radius: 50%; }
#csize {
    width: 50px;
    height: 4px;
    appearance: none;
    -webkit-appearance: none;
    background: rgba(255,255,255,0.2);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
}
#csize::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 12px; height: 12px;
    border-radius: 50%;
    background: #fff;
    border: none;
    cursor: pointer;
}

/* ==================== COMPACT LAYOUT FOR SMALL WINDOWS ==================== */
/* The window has no minimum size any more, so both bars have to survive being squeezed. Measured
   off the rules above, and in the row's own order, the bottom row's own minimum width is: play at
   34, the four text buttons at about 35 to 39px each (their own text plus 8px of padding a side,
   past the 34px floor), both time readouts at their 62px floor, volume at about 83 (15px icon, 5px
   gap, 60px slider), loop and fullscreen at 34, so about 457px of items; then ten 10px gaps; then
   48px of #hud padding, a 1px border and 14px of padding a side on .bar: about 635px. Nothing clips
   the row, so under that it spills past the window and its right-hand buttons stop being clickable.
   640 is that figure rounded up. Below it the row keeps #bplay, #tc and the seek bar, the top bar
   keeps #bdraw, #bcls and the drag handle, and a mouse-only viewer is still able to pause. */
@media (max-width: 640px) {
    /* Hidden, and still reachable: the two seek steps and the two frame steps are the arrow keys
       and comma and period, volume is the up and down arrows and M, loop is L, fullscreen is F or
       F11, and each of those runs off the keyboard whatever its button looks like. The hide and
       fill buttons are the bare H and C. Minimize has nowhere useful to go on a window this size.
       The caption goes as well, since it is the one element here that ever crowded the window
       buttons. The drag handle stays: what is left of the top bar is the pen, the cross and the
       handle, so the window is still moved by dragging it with the controls up, and no keypress
       stands between the owner and the window's position. */
    #bseek-back, #bseek-fwd, #bprev, #bnext,
    #tr, .vol-container, #bloop, #bfs,
    .title-capsule, #bhide, #bfill, #bmin { display: none; }
    /* The handle is absolute with inset: 0, so it fills the 48px top bar already and costs the
       compact row nothing: out of flow, and in the other bar besides. What it does cost is the
       resize margin, 8px off every edge, which is a tenth of an 80px window, and a press there
       belongs to the edge handler and not to a move. Inset by that margin on three sides, leaving
       a handle (window less 16) by 40px: 64 by 40 at an 80px window, of which the pen and the
       cross take the middle 28, so the strip under them is (window less 16) by 10 and wears the
       handle's own grab cursor. The bottom stays at 0 rather than 8 because handing those 8px back
       leaves a 2px strip, and the only window that gains is one under 56px tall. */
    .drag-handle { inset: 8px 8px 0 8px; }
    /* Hiding the caption leaves no flex gap behind it, so .winctrl is the top bar's only in-flow
       child and space-between holds it against the right padding, 16px clear of the resize
       margin. In the row the padding that came off goes to the seek bar: flex: 1 is already
       1 1 0%, so it grows into the freed width, and its automatic minimum size is the one thing
       that could stop it ever reaching zero. The time readout stays, it costs 62px and it is the
       only thing left that says where in the video you are. */
    #hud { padding-left: 16px; padding-right: 16px; }
    .bar { padding-left: 6px; padding-right: 6px; }
    .seekbar-container { min-width: 0; }
}
/* Under this the compact row runs out too: 46px of padding and border, 96px of button and time and
   two 10px gaps put the seek bar's zero at 162px of window. Taking the row's own padding, its gap
   and the time font down gives 42px back, which holds a seek bar a person can hit down to a 120px
   window and leaves each side of the row at least 15px inside the window, clear of the 8px resize
   margin. */
@media (max-width: 220px) {
    #hud { padding-left: 10px; padding-right: 10px; }
    .bar { padding-left: 4px; padding-right: 4px; }
    .controls-row { gap: 6px; }
    #tc { font-size: 10px; min-width: 0; }
}
/* A 48px top bar and a 46px bar whose top edge sits 64px up the window meet at 112px of height.
   Only the bottom bar gives ground: its own padding halves and the gap under it trims to 12px,
   which keeps the row's controls clear of the top bar's buttons down to an 83px window and leaves
   the seek strip 19px off the bottom edge, still outside the resize margin. The top bar keeps its
   48px so its buttons never reach into the margin at the top edge, and the seek bar is never
   hidden for height. Longhands only, so this composes with the two width rules above. */
@media (max-height: 112px) {
    .bar { padding-top: 4px; padding-bottom: 4px; }
    #hud { padding-bottom: 12px; }
}
</style>
</head>
<body>
<div id="stage">
  <video id="v" playsinline></video>
  <canvas id="draw"></canvas>
</div>

<!-- Top bar -->
<div id="topbar">
    <div class="drag-handle" id="drag-handle"></div>
    <div class="title-capsule" id="title">Dr.Player</div>
    <div class="winctrl">
        <button class="wbtn" id="bhide" title="Toggle on-screen controls">◎</button>
        <button class="wbtn" id="bfill" title="Fill: shape the window to the video (C)">▢</button>
        <button class="wbtn" id="bdraw" title="Draw on video">✎</button>
        <button class="wbtn" id="bmin" title="Minimize">—</button>
        <button class="wbtn close" id="bcls" title="Close">✕</button>
    </div>
</div>

<!-- Bottom HUD – single row -->
<div id="hud">
    <div class="bar">
        <div class="controls-row">
            <button class="btn" id="bplay" title="Play/Pause">▶</button>
            <button class="btn" id="bseek-back">-5s</button>
            <button class="btn" id="bseek-fwd">+5s</button>
            <button class="btn" id="bprev">-1F</button>
            <button class="btn" id="bnext">+1F</button>

            <span class="seek-time" id="tc">00:00:00</span>

            <div class="seekbar-container" id="seekbar-container">
                <div class="seek-track">
                    <div class="seek-fill" id="seek-fill"></div>
                    <div class="seek-thumb" id="seek-thumb"></div>
                </div>
            </div>

            <span class="seek-time" id="tr">-00:00:00</span>

            <div class="vol-container">
                <span class="vol-icon" id="vol-icon">🔊</span>
                <div class="vol-slider" id="vol-slider">
                    <div class="vol-fill" id="vol-fill"></div>
                </div>
            </div>

            <button class="btn btn-loop" id="bloop" title="Loop: Off">
                <svg viewBox="0 0 14 14" class="loop-svg" id="loop-off" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M2 4h9v7H2V7"/>
                    <path d="M0.5 8.5L2 7l1.5 1.5"/>
                </svg>
                <svg viewBox="0 0 14 14" class="loop-svg" id="loop-on" style="display:none" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M2 4h9v7H2V4"/>
                    <path d="M9.5 2.5L11 4l-1.5 1.5"/>
                    <path d="M3.5 9.5L2 11l1.5 1.5"/>
                </svg>
            </button>
            <button class="btn fs-btn" id="bfs" title="Fullscreen">
                <svg viewBox="0 0 14 14">
                    <path d="M1 1h4v1.5H2.5V5H1V1z
                             M8 1h4v4h-1.5V2.5H8V1z
                             M1 9h1.5v2.5H5V13H1V9z
                             M12 9v4H9v-1.5h2.5V9H12z"/>
                </svg>
            </button>
        </div>
    </div>
</div>

<!-- Drawing toolbar -->
<div id="drawbar">
  <button class="dhandle" title="Drag to move toolbar">
    <svg viewBox="0 0 20 42" fill="currentColor"><circle cx="5" cy="7" r="1.8"/><circle cx="15" cy="7" r="1.8"/><circle cx="5" cy="21" r="1.8"/><circle cx="15" cy="21" r="1.8"/><circle cx="5" cy="35" r="1.8"/><circle cx="15" cy="35" r="1.8"/></svg>
  </button>
  <div class="drawbar-body">
  <div class="drow">
    <button class="dbtn active" data-tool="pen" title="Pen (P)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2l2 2-9 9L2 13l1-3 9-8z"/><line x1="11" y1="3" x2="13" y2="5"/></svg><span class="dkey">/P</span>
    </button>
    <button class="dbtn" data-tool="line" title="Line (L)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="2" y1="14" x2="14" y2="2"/></svg><span class="dkey">/L</span>
    </button>
    <button class="dbtn" data-tool="arrow" title="Arrow (A)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><line x1="2" y1="14" x2="14" y2="2"/><path d="M14 2l-4 1M14 2l-1 4"/></svg><span class="dkey">/A</span>
    </button>
    <button class="dbtn" data-tool="rect" title="Rectangle (R)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="12" height="12"/></svg><span class="dkey">/R</span>
    </button>
    <button class="dbtn" data-tool="circle" title="Circle (C)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/></svg><span class="dkey">/C</span>
    </button>
    <button class="dbtn" data-tool="hand" title="Hand (H)">
      <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M4 10a4 4 0 0 0 8 0"/><path d="M5 10V6.5M7 10V4.5M9 10V5.5M11 10V7"/></svg><span class="dkey">/H</span>
    </button>
    <div class="dsep"></div>
    <div class="cswatch sel" data-color="#ff0000" style="background:#ff0000" title="Red"></div>
    <div class="cswatch" data-color="#00ff00" style="background:#00ff00" title="Green"></div>
    <div class="cswatch" data-color="#0066ff" style="background:#0066ff" title="Blue"></div>
    <div class="cswatch" data-color="#ffff00" style="background:#ffff00" title="Yellow"></div>
    <div class="cswatch" data-color="#ffffff" style="background:#ffffff" title="White"></div>
    <div class="cswatch" data-color="#000000" style="background:#000000" title="Black"></div>
    <input type="color" id="cpicker" value="#ff0000" title="Custom color">
  </div>
  <div class="drow">
    <input type="range" id="csize" min="1" max="20" value="3" title="Stroke size">
    <div class="dsep"></div>
    <button class="dbtn dseek" data-dir="-5" title="-5s">-5s</button>
    <button class="dbtn dseek" data-dir="+5" title="+5s">+5s</button>
    <button class="dbtn dstep" data-dir="-1" title="-1 Frame">-1F</button>
    <button class="dbtn dstep" data-dir="+1" title="+1 Frame">+1F</button>
    <div class="dsep"></div>
    <button class="dbtn" id="bundo" title="Undo (Mouse Back)">↩</button>
    <button class="dbtn" id="bredo" title="Redo (Mouse Forward)">↪</button>
    <button class="dbtn" id="bclear" title="Clear All (Ctrl+Backspace)">✕</button>
    <div class="dsep"></div>
    <button class="dbtn" id="bclose-draw" title="Exit Draw Mode">✕ Exit</button>
  </div>
  </div>
</div>

<!-- Custom text input dialog (replaces window.prompt which crashes on macOS/WKWebView) -->
<div id="text-dialog" style="display:none;position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.6);align-items:center;justify-content:center;">
  <div style="background:rgba(30,30,30,0.95);backdrop-filter:blur(18px);border:1px solid rgba(255,255,255,0.15);border-radius:14px;padding:24px 28px;min-width:300px;box-shadow:0 16px 48px rgba(0,0,0,0.6);">
    <div style="color:#fff;font-size:14px;font-weight:500;margin-bottom:14px;">Enter text:</div>
    <input id="text-dialog-input" type="text" style="width:100%;padding:8px 12px;border-radius:8px;border:1px solid rgba(255,255,255,0.2);background:rgba(255,255,255,0.08);color:#fff;font-size:14px;outline:none;">
    <div style="display:flex;gap:8px;margin-top:14px;justify-content:flex-end;">
      <button id="text-dialog-cancel" style="padding:6px 16px;border-radius:8px;border:1px solid rgba(255,255,255,0.15);background:transparent;color:rgba(255,255,255,0.7);cursor:pointer;font-size:13px;">Cancel</button>
      <button id="text-dialog-ok" style="padding:6px 20px;border-radius:8px;border:none;background:#e00;color:#fff;cursor:pointer;font-size:13px;font-weight:500;">OK</button>
    </div>
  </div>
</div>

<script>
const vid    = document.getElementById('v');
const topbar = document.getElementById('topbar');
const hud    = document.getElementById('hud');
const bhide  = document.getElementById('bhide');
const bfill  = document.getElementById('bfill');
let hideT    = null;
let uVol      = 1.0;
let hudLock   = false;
let hudPin    = false;
let fillMode  = false;
let loopEnabled = false;
// Try playback with audio; fall back to muted if browser blocks
vid.muted = false;
vid.volume = 1.0;
let playPromise = vid.play();
if (playPromise !== undefined) {
    playPromise.then(() => {
        if (vid.muted) vid.muted = false;
    }).catch(() => {
        vid.muted = true;
        vid.play().catch(() => {});
    });
}

/* ====================== GLOBAL ERROR HANDLER ====================== */
window.addEventListener('error', (event) => {
    console.error('GLOBAL_ERROR', event.message, event.error?.stack);
    event.preventDefault();
});
window.addEventListener('unhandledrejection', (event) => {
    console.error('UNHANDLED_PROMISE', event.reason);
    event.preventDefault();
});
/* ====================== END GLOBAL ERROR HANDLER ====================== */

/* ====================== TEXT INPUT DIALOG (replaces prompt()) ====================== */
const textDialog = document.getElementById('text-dialog');
const textInput = document.getElementById('text-dialog-input');
let textResolve = null;

function closeTextDialog(result) {
    textDialog.style.display = 'none';
    // Never leave focus parked in an input that is no longer on screen
    textInput.blur();
    if (textResolve) textResolve(result);
    textResolve = null;
}
document.getElementById('text-dialog-ok').addEventListener('click', () => {
    closeTextDialog(textInput.value);
});
document.getElementById('text-dialog-cancel').addEventListener('click', () => {
    closeTextDialog(null);
});
// Keys stay inside the prompt only while it is genuinely open, so a closed
// dialog can never swallow a player shortcut, wherever focus happens to sit
textInput.addEventListener('keydown', (e) => {
    if (textDialog.style.display === 'none') return;
    if (e.key === 'Enter') { document.getElementById('text-dialog-ok').click(); }
    else if (e.key === 'Escape') { document.getElementById('text-dialog-cancel').click(); }
    e.stopPropagation();
});

function showTextDialog() {
    return new Promise((resolve) => {
        textResolve = resolve;
        textInput.value = '';
        textDialog.style.display = 'flex';
        setTimeout(() => textInput.focus(), 50);
    });
}
/* ====================== CURSOR HELPER (deferred to avoid WKWebView crash on macOS) ====================== */
function setCanvasCursor(cursor) {
    requestAnimationFrame(() => {
        drawCanvas.style.cursor = cursor;
    });
}
/* ====================== END CURSOR HELPER ====================== */

/* ====================== ON-SCREEN CONTROLS ====================== */
const RESIZE_MARGIN = 8;
// no pointer position before the first mousemove; the centre is not an edge
let curX = window.innerWidth / 2, curY = window.innerHeight / 2;
// The only writer of the body cursor: an inline cursor cannot be overruled by a
// stylesheet rule, so the edge cursors survive the chrome being hidden
function applyBodyCursor() {
    const w = window.innerWidth;
    const h = window.innerHeight;
    const top = curY < RESIZE_MARGIN;
    const bottom = curY > h - RESIZE_MARGIN;
    const left = curX < RESIZE_MARGIN;
    const right = curX > w - RESIZE_MARGIN;

    let cursor = 'default';
    if ((top && left) || (bottom && right)) cursor = 'nwse-resize';
    else if ((top && right) || (bottom && left)) cursor = 'nesw-resize';
    else if (top || bottom) cursor = 'ns-resize';
    else if (left || right) cursor = 'ew-resize';
    document.body.style.cursor = cursor;
}
// The ◎ glyph is written from the resulting state, here and nowhere else
function renderHideGlyph() {
    bhide.textContent = topbar.classList.contains('show') ? '◎' : '◌';
}
function showControls() {
    topbar.classList.add('show');
    hud.classList.add('show');
    renderHideGlyph();
    applyBodyCursor();
}
function hideControls() {
    clearTimeout(hideT);
    topbar.classList.remove('show');
    hud.classList.remove('show');
    renderHideGlyph();
    applyBodyCursor();
}
function showUI() {
    if (hudLock || hudPin || document.body.classList.contains('drawmode')) return;
    showControls();
    clearTimeout(hideT);
    hideT = setTimeout(hideControls, 3000);
}
// H hides the chrome and brings it back pinned
function toggleControls() {
    if (topbar.classList.contains('show') || hud.classList.contains('show')) {
        hudLock = true;
        hideControls();
    } else {
        hudLock = false;
        hudPin = true;
        showControls();
    }
}
// Fill mode reshapes the WINDOW, never the picture. contain already draws the whole frame at
// its true proportions, so giving the window that same ratio leaves nothing for contain to
// letterbox: the picture covers the window edge to edge with no bars, nothing cropped and
// nothing stretched. Off by default: contain and a black surround is the look the app has
// always had.
let videoAspect = null; // videoWidth / videoHeight, null until metadata yields a usable size
let fillOwnResize = null; // {w, h} of the shape fill mode last asked the window to take

function renderFillGlyph() {
    bfill.textContent = fillMode ? '▣' : '▢';
}
function readVideoAspect() {
    const w = vid.videoWidth;
    const h = vid.videoHeight;
    videoAspect = (w > 0 && h > 0) ? w / h : null;
}
// A shape is posted exactly as asked, with no minimum and no maximum, so any size the owner drags
// or a snap wants can reach the window. The one thing refused is a shape that is not a shape, and
// the ratio correction is what can ask for one: it divides, and a NaN sails through a division and a
// round alike, as does a fraction that rounds down to nothing. Either would reach tao as a window
// of zero pixels. Both axes are rounded together, never clamped one at a time, so the ratio survives.
function resizeShape(w, h) {
    if (!Number.isFinite(w) || !Number.isFinite(h)) return null;
    return { w: Math.max(1, Math.round(w)), h: Math.max(1, Math.round(h)) };
}
// Height is the axis kept: a wide frame then widens the window, so the owner sees more of the
// picture rather than less. Does nothing when the ratio is not known yet.
function snapWindowToVideo() {
    if (!fillMode || !videoAspect) return;
    const h = window.innerHeight;
    // Refused means nothing is posted, so nothing is remembered: a shape that never left the page
    // cannot later look to the lock check like a snap that landed.
    const shape = resizeShape(h * videoAspect, h);
    if (!shape) return;
    // Remembered so the lock check can tell this resize apart from an outside one: the window
    // still wears its old shape until the posted size arrives.
    fillOwnResize = shape;
    window.ipc.postMessage(`resize:${shape.w}:${shape.h}`);
}
// How far the window may sit from the video's shape, in pixels. Rounding the posted shape is
// worth under a pixel on each axis, and the drag correction above already treats anything
// within 2px on each axis as on the ratio, so a window the app locked itself can end a gesture
// a few pixels off exact. 12px covers that at any frame shape and stays under the smallest real
// break, because a snap leaving less than 12px of bars has not visibly broken anything.
const FILL_SHAPE_TOL_PX = 12;
// Something outside the app changed the window's shape. Fill mode gives way rather than argue:
// the window is left exactly where Windows put it and the button stops claiming a mode that is
// no longer happening. No corrective resize is posted, on purpose.
function disengageFill() {
    fillMode = false;
    document.body.classList.remove('fillmode');
    renderFillGlyph();
}
function checkFillShapeLock() {
    if (!fillMode || !videoAspect) return;
    // Mid-drag the in-app correction is what holds the ratio, so the window is never judged then.
    if (resizing) return;
    const w = window.innerWidth;
    const h = window.innerHeight;
    if (w <= 0 || h <= 0) return;
    // Arriving at the shape fill mode itself asked for is the snap landing, not a break, and it
    // is passed over once. Judged here it would read the pre-snap shape and cancel the mode the
    // owner had just asked for. Nothing is remembered after this, so an ignored event cannot
    // silence the check later.
    const own = fillOwnResize;
    fillOwnResize = null;
    if (own && Math.abs(w - own.w) <= FILL_SHAPE_TOL_PX && Math.abs(h - own.h) <= FILL_SHAPE_TOL_PX) return;
    // contain fits the whole frame inside the window, so bars fall on one axis only: the sides
    // when the window is wider than the frame, the top and bottom when it is taller.
    const bars = videoAspect * w >= h ? w - h * videoAspect : h - w / videoAspect;
    if (bars > FILL_SHAPE_TOL_PX) disengageFill();
}
function toggleFill() {
    fillMode = !fillMode;
    document.body.classList.toggle('fillmode', fillMode);
    renderFillGlyph();
    // Only on the way in. Turning it off leaves the window exactly where the user left it.
    if (fillMode) snapWindowToVideo();
}
// Metadata is the first moment the frame's own size exists, and it can land after the user has
// already turned fill mode on, so the ratio is read here and a snap deferred by toggleFill is
// done now rather than skipped.
vid.addEventListener('loadedmetadata', () => {
    readVideoAspect();
    snapWindowToVideo();
});
readVideoAspect(); // nothing to read yet on first run; harmless if there ever is
document.addEventListener('mousemove', () => { hudPin = false; showUI(); });
document.addEventListener('keydown', e => {
    if (e.code === 'Escape') return;
    hudPin = false;
    showUI();
});
showUI();

bhide.addEventListener('click', () => {
    hudLock = !hudLock;
    if (hudLock) hideControls();
    else showUI();
});
bfill.addEventListener('click', toggleFill);

/* ====================== WINDOW DRAGGING ====================== */
document.getElementById('drag-handle').addEventListener('mousedown', (e) => {
    e.preventDefault();
    window.ipc.postMessage('drag_window');
});
document.querySelector('.title-capsule').addEventListener('mousedown', (e) => {
    e.preventDefault();
    window.ipc.postMessage('drag_window');
});

/* ====================== WINDOW RESIZING (NO MIN SIZE) ====================== */
let resizing = false;
let resizeDir = '';
let startX = 0, startY = 0, startW = 0, startH = 0;
let resizePending = false;
let resizeW = 0, resizeH = 0;

document.addEventListener('mousemove', (e) => {
    if (resizing) {
        const dx = e.screenX - startX;
        const dy = e.screenY - startY;
        let newW = startW;
        let newH = startH;
        if (resizeDir.includes('e')) newW = startW + dx;
        if (resizeDir.includes('w')) newW = startW - dx;
        if (resizeDir.includes('s')) newH = startH + dy;
        if (resizeDir.includes('n')) newH = startH - dy;
        newW = Math.max(1, newW);
        newH = Math.max(1, newH);
        // Fill mode corrects the axis the drag is not driving, from the one it is, so the
        // window ends every gesture on the video's ratio. Off, or with no ratio yet, this is
        // the untouched free resize.
        if (fillMode && videoAspect) {
            const dragW = newW;
            const dragH = newH;
            const horiz = resizeDir.includes('e') || resizeDir.includes('w');
            const vert = resizeDir.includes('n') || resizeDir.includes('s');
            let w = dragW;
            let h = dragH;
            if (horiz && !vert) {
                h = dragW / videoAspect;
            } else if (vert && !horiz) {
                w = dragH * videoAspect;
            } else if (horiz && vert) {
                // A corner is in play, so both axes are moving. The axis that moved less is the
                // one recomputed, so the axis the cursor actually led is the one kept, and the
                // gesture is followed rather than argued with. A dead heat goes to the width,
                // which for a wide frame is the axis that tracks the cursor most closely.
                if (Math.abs(dragW - startW) >= Math.abs(dragH - startH)) h = dragW / videoAspect;
                else w = dragH * videoAspect;
            }
            const shape = resizeShape(w, h);
            // A shape that is not a shape is not posted, and guessing one would move the window
            // somewhere the cursor never went, so the drag goes through uncorrected, which is
            // exactly what fill mode off would have done with it.
            if (shape) {
                // Already on the ratio: post nothing, so a held cursor cannot cause jitter
                if (Math.abs(shape.w - dragW) <= 2 && Math.abs(shape.h - dragH) <= 2) return;
                newW = shape.w;
                newH = shape.h;
            }
        }
        resizeW = newW;
        resizeH = newH;
        if (!resizePending) {
            resizePending = true;
            requestAnimationFrame(() => {
                resizePending = false;
                window.ipc.postMessage(`resize:${resizeW}:${resizeH}`);
            });
        }
        return;
    }

    curX = e.clientX;
    curY = e.clientY;
    applyBodyCursor();
});

document.addEventListener('mousedown', (e) => {
    const x = e.clientX;
    const y = e.clientY;
    const w = window.innerWidth;
    const h = window.innerHeight;
    const top = y < RESIZE_MARGIN;
    const bottom = y > h - RESIZE_MARGIN;
    const left = x < RESIZE_MARGIN;
    const right = x > w - RESIZE_MARGIN;

    if (top || bottom || left || right) {
        e.preventDefault();
        e.stopPropagation();
        resizing = true;
        resizeDir = '';
        if (top) resizeDir += 'n';
        if (bottom) resizeDir += 's';
        if (left) resizeDir += 'w';
        if (right) resizeDir += 'e';
        startX = e.screenX;
        startY = e.screenY;
        startW = window.outerWidth;
        startH = window.outerHeight;
    }
});

/* ====================== MOVE THE WINDOW WITH THE CHROME HIDDEN ====================== */
// With the chrome hidden there is no title bar left to grab, so a press on the stage
// moves the window instead. Registered after the edge handler above on purpose: that
// handler sets `resizing` on an edge press, and this one bails out, so resize wins.
document.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return;
    if (resizing) return;
    if (drawbar.classList.contains('open')) return;
    if (textDialog.style.display !== 'none') return;
    // Only while hidden: with the chrome up, the title bar is the drag affordance
    if (topbar.classList.contains('show') || hud.classList.contains('show')) return;
    e.preventDefault();
    window.ipc.postMessage('drag_window');
});

document.addEventListener('mouseup', () => {
    resizing = false;
    resizeDir = '';
    resizePending = false;
});

window.addEventListener('blur', () => {
    resizing = false;
    resizePending = false;
    seeking = false;
    vDrag = false;
    dragData = null;
    drawing = false;
    selShape = null;
});

/* ====================== WINDOW BUTTONS ====================== */
document.getElementById('bmin').onclick = () => window.ipc.postMessage('minimize');
document.getElementById('bcls').onclick = () => window.ipc.postMessage('close');

/* ====================== TIME FORMATTER ====================== */
function fmt(s) {
    if (!s || isNaN(s)) return '00:00:00';
    s = Math.floor(Math.abs(s));
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    return [h, m, sec].map(n => String(n).padStart(2,'0')).join(':');
}

/* ====================== SEEKBAR ====================== */
const seekContainer = document.getElementById('seekbar-container');
const seekFill = document.getElementById('seek-fill');
const seekThumb = document.getElementById('seek-thumb');
const tcEl = document.getElementById('tc');
const trEl = document.getElementById('tr');
let seeking = false;

function updateSeekbar() {
    if (!vid.duration) return;
    const pct = (vid.currentTime / vid.duration) * 100;
    seekFill.style.width = pct + '%';
    seekThumb.style.left = pct + '%';
    tcEl.textContent = fmt(vid.currentTime);
    trEl.textContent = '-' + fmt(vid.duration - vid.currentTime);
}
vid.addEventListener('timeupdate', updateSeekbar);
vid.addEventListener('loadedmetadata', updateSeekbar);

function seekFromEvent(e) {
    const rect = seekContainer.getBoundingClientRect();
    const pos = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    vid.currentTime = pos * vid.duration;
}
seekContainer.addEventListener('mousedown', (e) => {
    if (resizing) return;
    seeking = true;
    seekFromEvent(e);
    e.stopPropagation();
});
document.addEventListener('mousemove', (e) => {
    if (seeking) seekFromEvent(e);
});
document.addEventListener('mouseup', () => { seeking = false; });

/* ====================== -5s / +5s ====================== */
document.getElementById('bseek-back').onclick = () => {
    vid.currentTime = Math.max(0, vid.currentTime - 5);
};
document.getElementById('bseek-fwd').onclick = () => {
    vid.currentTime = Math.min(vid.duration || 0, vid.currentTime + 5);
};

/* ====================== VOLUME ====================== */
const volSlider = document.getElementById('vol-slider');
const volFill   = document.getElementById('vol-fill');
const volIcon   = document.getElementById('vol-icon');
let vDrag = false;

function setVol(v) {
    uVol = Math.max(0, Math.min(1, v));
    vid.volume = uVol;
    vid.muted  = (uVol === 0);
    volFill.style.width = (uVol * 100) + '%';
    volIcon.textContent = uVol === 0 ? '🔇' : '🔊';
}
function volFromE(e) {
    const r = volSlider.getBoundingClientRect();
    setVol((e.clientX - r.left) / r.width);
}
volSlider.addEventListener('mousedown', e => { vDrag = true; volFromE(e); e.stopPropagation(); });
document.addEventListener('mousemove', e => { if (vDrag) volFromE(e); });
document.addEventListener('mouseup',   () => { vDrag = false; });

volIcon.addEventListener('click', () => {
    if (vid.muted) {
        vid.muted = false;
        setVol(uVol || 0.5);
    } else {
        vid.muted = true;
        volIcon.textContent = '🔇';
    }
});

/* ====================== FRAME STEPPING ====================== */
document.getElementById('bprev').onclick = () => {
    vid.pause();
    vid.currentTime = Math.max(0, vid.currentTime - 1 / 60);
};
document.getElementById('bnext').onclick = () => {
    vid.pause();
    vid.currentTime = Math.min(vid.duration || 0, vid.currentTime + 1 / 60);
};

document.getElementById('bplay').onclick = () => {
    vid.paused ? vid.play() : vid.pause();
};
vid.addEventListener('play', () => document.getElementById('bplay').textContent = '⏸');
vid.addEventListener('pause', () => document.getElementById('bplay').textContent = '▶');

document.getElementById('bloop').onclick = () => {
    loopEnabled = !loopEnabled;
    const btn = document.getElementById('bloop');
    btn.classList.toggle('active', loopEnabled);
    document.getElementById('loop-off').style.display = loopEnabled ? 'none' : '';
    document.getElementById('loop-on').style.display = loopEnabled ? '' : 'none';
    btn.title = loopEnabled ? 'Loop: On' : 'Loop: Off';
};
vid.addEventListener('ended', () => {
    if (loopEnabled) {
        vid.currentTime = 0;
        vid.play().catch(()=>{});
    }
});
vid.onerror = () => {
    document.getElementById('title').textContent = 'Error loading video';
};

/* ====================== FULLSCREEN ====================== */
// The window owns the real state and pushes it here; the page never assumes it won
let isFullscreen = false;
window.setFullscreenState = state => { isFullscreen = !!state; };
function requestFullscreen(enter) {
    window.ipc.postMessage(enter ? 'fullscreen' : 'exit_fullscreen');
}
document.getElementById('bfs').onclick = () => requestFullscreen(!isFullscreen);

/* ====================== KEYBOARD SHORTCUTS ====================== */
document.addEventListener('keydown', e => {
    // Bail out early in draw mode — draw mode handles its own keys
    if (drawbar.classList.contains('open')) return;
    // H toggles the chrome, fullscreen or not. No modifier test: this WebView2 never
    // delivers modified keys to the page, so gating on ctrl/meta made the chord dead.
    if (e.code === 'KeyH') {
        e.preventDefault();
        toggleControls();
        return;
    }
    // C turns on fill mode, which reshapes the window to the video rather than changing how
    // the picture is drawn. Bare for the same reason as H: this WebView2 never delivers
    // modified keys to the page. In draw mode the early return above has already handed C to
    // the circle tool, as with L and H.
    if (e.code === 'KeyC') {
        e.preventDefault();
        toggleFill();
        return;
    }
    switch (e.code) {
        case 'Space':      e.preventDefault(); vid.paused ? vid.play() : vid.pause(); break;
        case 'ArrowRight': vid.currentTime = Math.min(vid.duration || 0, vid.currentTime + 5); break;
        case 'ArrowLeft':  vid.currentTime = Math.max(0, vid.currentTime - 5); break;
        case 'ArrowUp':    setVol(uVol + 0.1); break;
        case 'ArrowDown':  setVol(uVol - 0.1); break;
        case 'Period':     document.getElementById('bnext').click(); break;
        case 'Comma':      document.getElementById('bprev').click(); break;
        case 'KeyF':
        case 'F11':        document.getElementById('bfs').click(); break;
        case 'Escape':     window.ipc.postMessage('exit_fullscreen'); break;
        case 'KeyM':       volIcon.click(); break;
        case 'KeyL':       document.getElementById('bloop').click(); break;
    }
});

/* ====================== DRAWING OVERLAY ====================== */
const drawCanvas = document.getElementById('draw');
const dctx = drawCanvas.getContext('2d');
const drawbar = document.getElementById('drawbar');
let tool = 'pen';
let color = '#ff0000';
let size = 3;
let drawing = false;

let shapes = [];
let undoStack = [];
let redoStack = [];
const MAX_HISTORY = 50;
let selShape = null;
let selShapeOffX = 0, selShapeOffY = 0;

function applyStyle(ctx, c, s) {
    ctx.strokeStyle = c;
    ctx.lineWidth = s;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.fillStyle = c;
}

function drawShape(ctx, s) {
    if (s.type === 'text') {
        ctx.font = s.fontSize + 'px sans-serif';
        ctx.fillStyle = s.color;
        ctx.fillText(s.text, s.x, s.y);
        return;
    }
    applyStyle(ctx, s.color, s.size);
    ctx.beginPath();
    switch (s.type) {
        case 'pen': {
            const pts = s.points;
            if (pts.length === 1) { pts.push(pts[0]); }
            ctx.moveTo(pts[0].x, pts[0].y);
            for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i].x, pts[i].y);
            ctx.stroke();
            break;
        }
        case 'line':
            ctx.moveTo(s.x1, s.y1); ctx.lineTo(s.x2, s.y2); ctx.stroke();
            break;
        case 'arrow': {
            ctx.moveTo(s.x1, s.y1); ctx.lineTo(s.x2, s.y2); ctx.stroke();
            const angle = Math.atan2(s.y2 - s.y1, s.x2 - s.x1);
            const h = Math.min(16, Math.max(8, s.size * 5));
            ctx.beginPath();
            ctx.moveTo(s.x2, s.y2);
            ctx.lineTo(s.x2 - h * Math.cos(angle - Math.PI / 6), s.y2 - h * Math.sin(angle - Math.PI / 6));
            ctx.lineTo(s.x2 - h * Math.cos(angle + Math.PI / 6), s.y2 - h * Math.sin(angle + Math.PI / 6));
            ctx.closePath();
            ctx.fill();
            break;
        }
        case 'rect':
            ctx.strokeRect(Math.min(s.x1, s.x2), Math.min(s.y1, s.y2), Math.abs(s.x2 - s.x1), Math.abs(s.y2 - s.y1));
            break;
        case 'circle':
            ctx.arc(s.x1, s.y1, Math.sqrt(Math.abs(s.x2 - s.x1) ** 2 + Math.abs(s.y2 - s.y1) ** 2), 0, Math.PI * 2);
            ctx.stroke();
            break;
    }
}

function renderAll() {
    dctx.clearRect(0, 0, drawCanvas.width, drawCanvas.height);
    for (const s of shapes) drawShape(dctx, s);
}

function resizeDrawCanvas() {
    drawCanvas.width = window.innerWidth;
    drawCanvas.height = window.innerHeight;
    renderAll();
}
window.addEventListener('resize', resizeDrawCanvas);
// Fill mode watches the same event for a shape it did not ask for
window.addEventListener('resize', checkFillShapeLock);
resizeDrawCanvas();

function saveDrawState() {
    undoStack.push(JSON.parse(JSON.stringify(shapes)));
    if (undoStack.length > MAX_HISTORY) undoStack.shift();
    redoStack = [];
}

function undoDraw() {
    if (undoStack.length === 0) return;
    redoStack.push(JSON.parse(JSON.stringify(shapes)));
    shapes = undoStack.pop();
    selShape = null;
    renderAll();
}

function redoDraw() {
    if (redoStack.length === 0) return;
    undoStack.push(JSON.parse(JSON.stringify(shapes)));
    shapes = redoStack.pop();
    selShape = null;
    renderAll();
}

function clearDrawCanvas() {
    saveDrawState();
    shapes = [];
    selShape = null;
    renderAll();
}

function getDrawPos(e) {
    const rect = drawCanvas.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
}

function hitTest(x, y) {
    const thresh = 12;
    for (let i = shapes.length - 1; i >= 0; i--) {
        const s = shapes[i];
        if (s.type === 'text') {
            dctx.font = s.fontSize + 'px sans-serif';
            const m = dctx.measureText(s.text);
            if (x >= s.x && x <= s.x + m.width && y >= s.y - s.fontSize && y <= s.y) return i;
        } else if (s.type === 'pen') {
            for (const p of s.points) {
                if (Math.abs(x - p.x) <= thresh && Math.abs(y - p.y) <= thresh) return i;
            }
        } else {
            const x1 = Math.min(s.x1, s.x2), x2 = Math.max(s.x1, s.x2);
            const y1 = Math.min(s.y1, s.y2), y2 = Math.max(s.y1, s.y2);
            if (x >= x1 - thresh && x <= x2 + thresh && y >= y1 - thresh && y <= y2 + thresh) return i;
        }
    }
    return -1;
}

function openDrawMode() {
    drawCanvas.style.pointerEvents = 'auto';
    setCanvasCursor(tool === 'hand' ? 'grab' : 'crosshair');
    drawbar.style.left = '';
    drawbar.style.top = '';
    drawbar.style.bottom = '';
    drawbar.style.transform = '';
    drawbar.classList.add('open');
    document.body.classList.add('drawmode');
    if (!vid.paused) vid.pause();
    showUI();
}

function closeDrawMode() {
    if (drawing) drawing = false;
    selShape = null;
    shapes = [];
    undoStack = [];
    redoStack = [];
    renderAll();
    drawCanvas.style.pointerEvents = 'none';
    setCanvasCursor('default');
    drawbar.classList.remove('open');
    document.body.classList.remove('drawmode');
}

document.getElementById('bdraw').addEventListener('click', () => {
    if (drawbar.classList.contains('open')) closeDrawMode();
    else openDrawMode();
});

let dragData = null;
document.querySelector('.dhandle').addEventListener('mousedown', (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    const rect = drawbar.getBoundingClientRect();
    drawbar.style.left = rect.left + 'px';
    drawbar.style.top = rect.top + 'px';
    drawbar.style.transform = 'none';
    drawbar.style.bottom = 'auto';
    dragData = { offX: e.clientX - rect.left, offY: e.clientY - rect.top };
});
document.addEventListener('mousemove', (e) => {
    if (!dragData) return;
    drawbar.style.left = (e.clientX - dragData.offX) + 'px';
    drawbar.style.top = (e.clientY - dragData.offY) + 'px';
});
document.addEventListener('mouseup', () => { dragData = null; });

drawCanvas.addEventListener('mousedown', (e) => {
    if (!drawbar.classList.contains('open') || e.button !== 0) return;
    // Ignore canvas clicks while text dialog is open
    if (textDialog.style.display !== 'none') return;
    e.preventDefault();
    const p = getDrawPos(e);
    const x = p.x, y = p.y;

    if (tool === 'hand') {
        const idx = hitTest(x, y);
        if (idx >= 0) {
            selShape = shapes[idx];
            if (selShape.type === 'text') {
                selShapeOffX = x - selShape.x;
                selShapeOffY = y - selShape.y;
            } else if (selShape.type === 'pen') {
                selShapeOffX = x - selShape.points[0].x;
                selShapeOffY = y - selShape.points[0].y;
            } else {
                selShapeOffX = x - (selShape.x1 + selShape.x2) / 2;
                selShapeOffY = y - (selShape.y1 + selShape.y2) / 2;
            }
            setCanvasCursor('grabbing');
            drawing = true;
        }
        return;
    }

    if (tool === 'text') {
        saveDrawState();
        showTextDialog().then(txt => {
            if (txt && txt.trim()) {
                shapes.push({ type: 'text', x, y, text: txt, fontSize: Math.max(12, size * 5), color });
                renderAll();
            }
        });
        return;
    }

    saveDrawState();
    drawing = true;
    if (tool === 'pen') {
        shapes.push({ type: 'pen', color, size, points: [{x, y}] });
    } else {
        shapes.push({ type: tool, color, size, x1: x, y1: y, x2: x, y2: y });
    }
});

drawCanvas.addEventListener('mousemove', (e) => {
    if (!drawing) return;
    e.preventDefault();
    const p = getDrawPos(e);
    const x = p.x, y = p.y;

    if (selShape) {
        if (selShape.type === 'text') {
            selShape.x = x - selShapeOffX;
            selShape.y = y - selShapeOffY;
        } else if (selShape.type === 'pen') {
            const dx = x - selShapeOffX - selShape.points[0].x;
            const dy = y - selShapeOffY - selShape.points[0].y;
            for (const pt of selShape.points) { pt.x += dx; pt.y += dy; }
            selShapeOffX = x - selShape.points[0].x;
            selShapeOffY = y - selShape.points[0].y;
        } else {
            const w2 = (selShape.x2 - selShape.x1) / 2;
            const h2 = (selShape.y2 - selShape.y1) / 2;
            const newCx = x - selShapeOffX;
            const newCy = y - selShapeOffY;
            selShape.x1 = newCx - w2;
            selShape.x2 = newCx + w2;
            selShape.y1 = newCy - h2;
            selShape.y2 = newCy + h2;
        }
        renderAll();
        return;
    }

    const s = shapes[shapes.length - 1];
    if (!s || s.type === 'text') { drawing = false; return; }

    if (s.type === 'pen') {
        s.points.push({x, y});
    } else {
        s.x2 = x;
        s.y2 = y;
    }
    renderAll();
});

drawCanvas.addEventListener('mouseup', (e) => {
    if (!drawing) return;
    if (e.button !== 0) return;
    drawing = false;
    if (selShape && tool === 'hand') setCanvasCursor('grab');
    selShape = null;
});

drawCanvas.addEventListener('mouseleave', () => {
    if (selShape && tool === 'hand') setCanvasCursor('grab');
    selShape = null;
    if (drawing) drawing = false;
});

drawCanvas.addEventListener('wheel', (e) => {
    if (!selShape) return;
    e.preventDefault();
    if (selShape.type === 'text') {
        selShape.fontSize = Math.max(8, selShape.fontSize + (e.deltaY > 0 ? -4 : 4));
    } else {
        selShape.size = Math.max(1, Math.min(40, selShape.size + (e.deltaY > 0 ? -1 : 1)));
    }
    renderAll();
});

document.querySelectorAll('.dbtn[data-tool]').forEach(btn => {
    btn.addEventListener('click', () => {
        switchTool(btn.dataset.tool);
    });
});

document.querySelectorAll('.cswatch').forEach(el => {
    el.addEventListener('click', () => {
        document.querySelectorAll('.cswatch').forEach(s => s.classList.remove('sel'));
        el.classList.add('sel');
        color = el.dataset.color;
        document.getElementById('cpicker').value = color;
    });
});

document.getElementById('cpicker').addEventListener('input', (e) => {
    color = e.target.value;
    document.querySelectorAll('.cswatch').forEach(s => s.classList.remove('sel'));
});

document.getElementById('csize').addEventListener('input', (e) => {
    size = parseInt(e.target.value);
});

document.getElementById('bundo').addEventListener('click', undoDraw);
document.getElementById('bredo').addEventListener('click', redoDraw);
document.getElementById('bclear').addEventListener('click', clearDrawCanvas);

document.querySelectorAll('.dseek').forEach(btn => {
    btn.addEventListener('click', () => {
        const dir = parseInt(btn.dataset.dir);
        vid.currentTime = Math.max(0, Math.min(vid.duration || 0, vid.currentTime + dir));
    });
});

document.querySelectorAll('.dstep').forEach(btn => {
    btn.addEventListener('click', () => {
        const dir = parseInt(btn.dataset.dir);
        vid.pause();
        vid.currentTime = Math.max(0, Math.min(vid.duration || 0, vid.currentTime + dir / 60));
    });
});

document.getElementById('bclose-draw').addEventListener('click', closeDrawMode);

drawCanvas.addEventListener('mouseup', (e) => {
    if (!drawbar.classList.contains('open') || drawing) return;
    if (e.button === 3) { undoDraw(); e.preventDefault(); }
    else if (e.button === 4) { redoDraw(); e.preventDefault(); }
});

function switchTool(t) {
    if (drawing) {
        drawing = false;
        if (shapes.length > 0) {
            const last = shapes[shapes.length - 1];
            const incomplete = last.type === 'pen' ? last.points.length <= 1 : (last.x1 === last.x2 && last.y1 === last.y2);
            if (incomplete) { shapes.pop(); renderAll(); }
        }
    }
    document.querySelectorAll('.dbtn[data-tool]').forEach(b => b.classList.remove('active'));
    const btn = document.querySelector(`[data-tool="${t}"]`);
    if (btn) btn.classList.add('active');
    tool = t;
    selShape = null;
    if (drawbar.classList.contains('open')) {
        setCanvasCursor(t === 'hand' ? 'grab' : 'crosshair');
    }
}

document.addEventListener('keydown', (e) => {
    if (!drawbar.classList.contains('open')) return;
    const k = e.key.toLowerCase();
    if (k === 'p') { switchTool('pen'); e.preventDefault(); }
    else if (k === 'l') { switchTool('line'); e.preventDefault(); }
    else if (k === 'a') { switchTool('arrow'); e.preventDefault(); }
    else if (k === 'r') { switchTool('rect'); e.preventDefault(); }
    else if (k === 'c') { switchTool('circle'); e.preventDefault(); }
    else if (k === 'h') { switchTool('hand'); e.preventDefault(); }
    else if (e.key === 'Escape') { closeDrawMode(); e.preventDefault(); }
    else if (k === 'delete' || k === 'backspace') {
        if ((e.ctrlKey || e.metaKey) && drawbar.classList.contains('open')) {
            e.preventDefault();
            clearDrawCanvas();
        } else if (selShape) {
            saveDrawState();
            const idx = shapes.indexOf(selShape);
            if (idx >= 0) { shapes.splice(idx, 1); renderAll(); }
            selShape = null;
            e.preventDefault();
        }
    }
    else if (k >= '0' && k <= '9') {
        const v = parseInt(k);
        size = 2 + v * 2;
        document.getElementById('csize').value = size;
        e.preventDefault();
    }
});
/* ====================== DRAWING KEYBOARD SHORTCUTS END ====================== */

vid.addEventListener('play', () => {
    if (drawbar.classList.contains('open')) closeDrawMode();
});

window.loadVideo = url => { vid.src = url; vid.play().catch(()=>{}); };
window.setTitle  = name => { document.getElementById('title').textContent = name; };
</script>
</body>
</html>
"##;

use clap::Parser;
use std::path::PathBuf;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, Window, WindowBuilder, Fullscreen},
};
use wry::WebViewBuilder;
use wry::WebViewBuilderExtWindows;

#[derive(Parser)]
#[command(name = "dr-player")]
struct Args {
    path: String,
}

use axum::Router;
use tower_http::services::ServeFile;
use rand::Rng;

fn load_icon(bytes: &[u8]) -> Option<Icon> {
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes)).ok()?;
    let entry = icon_dir.entries().iter().max_by_key(|e| e.width())?;
    let img = entry.decode().ok()?;
    Icon::from_rgba(img.rgba_data().to_vec(), img.width(), img.height()).ok()
}

// A posted size is applied exactly as asked, with no minimum and no maximum, so the window can be
// dragged or snapped to any size at all. What is refused is a size that is not one: tao reaches
// SetWindowPos by rounding a logical size into u32 pixels, and a NaN, an infinity and a negative all
// round to zero there, as does anything under half a pixel. A zero sized window is one the owner
// cannot drag back out of, so such a message is dropped and the window keeps the size it has. One
// pixel is the smallest real window, not a limit anyone can feel.
fn is_postable_size(w: f64, h: f64) -> bool {
    w.is_finite() && h.is_finite() && w >= 1.0 && h >= 1.0
}

// The window is the only authority on fullscreen, so the page hears it from here
fn push_fullscreen_state(window: &Window, webview: &wry::WebView) {
    let state = window.fullscreen().is_some();
    let _ = webview.evaluate_script(&format!("window.setFullscreenState({});", state));
}

#[tokio::main]
async fn main() -> wry::Result<()> {
    // Install panic hook for graceful crash handling
    std::panic::set_hook(Box::new(|info| {
        eprintln!("Dr.Player internal error: {}", info);
    }));

    let args = Args::parse();

    let path = std::fs::canonicalize(&args.path).unwrap_or(PathBuf::from(&args.path));

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Dr.Player")
        .to_string();

    let token: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    let app = Router::new().nest_service(&format!("/{}", token), ServeFile::new(&path));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let file_url = format!("http://127.0.0.1:{}/{}", port, token);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let event_loop = EventLoopBuilder::<String>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let app_icon = load_icon(include_bytes!("../resources/icon.ico"));

    let window = WindowBuilder::new()
        .with_title("Dr.Player")
        .with_window_icon(app_icon)
        .with_decorations(false)
        .with_resizable(true)
        .with_inner_size(tao::dpi::LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)
        .unwrap();

    let safe_url = serde_json::to_string(&file_url).unwrap();
    let safe_title = serde_json::to_string(&filename).unwrap();
    let init_script = format!(
        "window.addEventListener('DOMContentLoaded',function(){{window.loadVideo({});window.setTitle({});}});",
        safe_url, safe_title
    );

    let mut builder = WebViewBuilder::new(&window)
        .with_html(HTML)
        .with_devtools(false)
        .with_autoplay(true);

    #[cfg(target_os = "windows")]
    {
        builder = builder.with_additional_browser_args("--autoplay-policy=no-user-gesture-required");
    }

    let webview = builder
        .with_initialization_script(&init_script)
        .with_ipc_handler(move |msg| {
            let _ = proxy.send_event(msg.body().to_string());
        })
        .build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(msg) => {
                if msg == "close" {
                    *control_flow = ControlFlow::Exit;
                } else if msg == "minimize" {
                    window.set_minimized(true);
                } else if msg == "fullscreen" {
                    if window.fullscreen().is_none() {
                        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
                    } else {
                        window.set_fullscreen(None);
                    }
                    push_fullscreen_state(&window, &webview);
                } else if msg == "exit_fullscreen" {
                    window.set_fullscreen(None);
                    push_fullscreen_state(&window, &webview);
                } else if msg == "drag_window" {
                    let _ = window.drag_window();
                } else if msg.starts_with("resize:") {
                    let parts: Vec<&str> = msg.split(':').collect();
                    if parts.len() == 3 {
                        if let (Ok(w), Ok(h)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                            if is_postable_size(w, h) {
                                let _ = window.set_inner_size(tao::dpi::LogicalSize::new(w, h));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    })
}