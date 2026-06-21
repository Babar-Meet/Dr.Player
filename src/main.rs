#![windows_subsystem = "windows"]

const HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Dr.Player</title>
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
    width: 0;
    height: 3px;
    background: rgba(255,255,255,0.2);
    border-radius: 2px;
    cursor: pointer;
    position: relative;
    overflow: hidden;
    transition: width 0.2s, opacity 0.1s;
    opacity: 0;
}
.vol-container:hover .vol-slider {
    width: 60px;
    opacity: 1;
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
</style>
</head>
<body>
<div id="stage">
  <video id="v"></video>
  <canvas id="draw"></canvas>
</div>

<!-- Top bar -->
<div id="topbar">
    <div class="drag-handle" id="drag-handle"></div>
    <div class="title-capsule" id="title">Dr.Player</div>
    <div class="winctrl">
        <button class="wbtn" id="bhide" title="Toggle on-screen controls">◎</button>
        <button class="wbtn" id="bdraw" title="Draw on video">✎</button>
        <button class="wbtn" id="bmin" title="Minimize">—</button>
        <button class="wbtn close" id="bcls" title="Close">✕</button>
    </div>
</div>

<!-- Bottom HUD – single row -->
<div id="hud">
    <div class="bar">
        <div class="controls-row">
            <button class="btn" id="bseek-back">-5s</button>
            <button class="btn" id="bseek-fwd">+5s</button>
            <button class="btn" id="bprev">-1F</button>
            <button class="btn" id="bnext">+1F</button>
            <button class="btn" id="bplay" title="Play/Pause">▶</button>

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

<script>
const vid    = document.getElementById('v');
const topbar = document.getElementById('topbar');
const hud    = document.getElementById('hud');
let hideT    = null;
let uVol     = 1.0;
let hudLock  = false;

function showUI() {
    if (hudLock || document.body.classList.contains('drawmode')) return;
    topbar.classList.add('show');
    hud.classList.add('show');
    document.body.classList.add('show-cur');
    clearTimeout(hideT);
    hideT = setTimeout(() => {
        topbar.classList.remove('show');
        hud.classList.remove('show');
        document.body.classList.remove('show-cur');
    }, 3000);
}
document.addEventListener('mousemove', showUI);
document.addEventListener('keydown',   showUI);
showUI();

document.getElementById('bhide').addEventListener('click', () => {
    hudLock = !hudLock;
    if (hudLock) {
        topbar.classList.remove('show');
        hud.classList.remove('show');
        document.body.classList.remove('show-cur');
        document.getElementById('bhide').textContent = '◌';
    } else {
        showUI();
        document.getElementById('bhide').textContent = '◎';
    }
});

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
const RESIZE_MARGIN = 8;
let resizing = false;
let resizeDir = '';
let startX = 0, startY = 0, startW = 0, startH = 0;

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
        // Only prevent negative/zero sizes by clamping to 1px (otherwise window may disappear)
        newW = Math.max(1, newW);
        newH = Math.max(1, newH);
        window.ipc.postMessage(`resize:${newW}:${newH}`);
        return;
    }

    const x = e.clientX;
    const y = e.clientY;
    const w = window.innerWidth;
    const h = window.innerHeight;
    const top = y < RESIZE_MARGIN;
    const bottom = y > h - RESIZE_MARGIN;
    const left = x < RESIZE_MARGIN;
    const right = x > w - RESIZE_MARGIN;

    let cursor = 'default';
    if ((top && left) || (bottom && right)) cursor = 'nwse-resize';
    else if ((top && right) || (bottom && left)) cursor = 'nesw-resize';
    else if (top || bottom) cursor = 'ns-resize';
    else if (left || right) cursor = 'ew-resize';
    document.body.style.cursor = cursor;
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

document.addEventListener('mouseup', () => {
    resizing = false;
    resizeDir = '';
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
    if (vid.volume > 0) {
        vid.muted = true;
        volIcon.textContent = '🔇';
    } else {
        vid.muted = false;
        setVol(0.5);
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

/* ====================== FULLSCREEN ====================== */
document.getElementById('bfs').onclick = () => window.ipc.postMessage('fullscreen');

/* ====================== KEYBOARD SHORTCUTS ====================== */
document.addEventListener('keydown', e => {
    showUI();
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
let drawStartX = 0, drawStartY = 0;
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
    drawCanvas.style.cursor = tool === 'hand' ? 'grab' : 'crosshair';
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
    shapes = [];
    undoStack = [];
    redoStack = [];
    selShape = null;
    renderAll();
    drawCanvas.style.pointerEvents = 'none';
    drawCanvas.style.cursor = 'default';
    drawbar.classList.remove('open');
    document.body.classList.remove('drawmode');
    if (drawing) drawing = false;
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
            drawCanvas.style.cursor = 'grabbing';
            drawing = true;
        }
        return;
    }

    if (tool === 'text') {
        saveDrawState();
        const txt = prompt('Enter text:');
        if (txt && txt.trim()) {
            shapes.push({ type: 'text', x, y, text: txt, fontSize: Math.max(12, size * 5), color });
            renderAll();
        }
        return;
    }

    saveDrawState();
    drawing = true;
    drawStartX = x;
    drawStartY = y;
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

    if (tool === 'pen') {
        s.points.push({x, y});
    } else {
        s.x2 = x;
        s.y2 = y;
    }
    renderAll();
});

drawCanvas.addEventListener('mouseup', (e) => {
    if (!drawing || e.button !== 0) return;
    drawing = false;
    if (selShape && tool === 'hand') drawCanvas.style.cursor = 'grab';
    selShape = null;
});

drawCanvas.addEventListener('mouseleave', () => {
    if (selShape && tool === 'hand') drawCanvas.style.cursor = 'grab';
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
        document.querySelectorAll('.dbtn[data-tool]').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        tool = btn.dataset.tool;
        selShape = null;
        if (drawbar.classList.contains('open')) {
            drawCanvas.style.cursor = tool === 'hand' ? 'grab' : 'crosshair';
        }
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
    if (!drawbar.classList.contains('open')) return;
    if (e.button === 3) { undoDraw(); e.preventDefault(); }
    else if (e.button === 4) { redoDraw(); e.preventDefault(); }
});

document.addEventListener('keydown', (e) => {
    if (!drawbar.classList.contains('open')) return;
    const k = e.key.toLowerCase();
    if (k === 'p') { document.querySelector('[data-tool="pen"]').click(); e.preventDefault(); }
    else if (k === 'l') { document.querySelector('[data-tool="line"]').click(); e.preventDefault(); }
    else if (k === 'a') { document.querySelector('[data-tool="arrow"]').click(); e.preventDefault(); }
    else if (k === 'r') { document.querySelector('[data-tool="rect"]').click(); e.preventDefault(); }
    else if (k === 'c') { document.querySelector('[data-tool="circle"]').click(); e.preventDefault(); }
    else if (k === 'h') { document.querySelector('[data-tool="hand"]').click(); e.preventDefault(); }
    else if (k === 'e') { closeDrawMode(); e.preventDefault(); }
    else if (k === 'delete' || k === 'backspace') {
        if (selShape) {
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

document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.code === 'Backspace') {
        e.preventDefault();
        if (drawbar.classList.contains('open')) clearDrawCanvas();
    }
});

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
    window::{Icon, WindowBuilder, Fullscreen},
};
use wry::WebViewBuilder;

#[derive(Parser)]
#[command(name = "dr-player")]
struct Args {
    path: String,
}

use axum::Router;
use tower_http::services::ServeFile;

fn load_icon(bytes: &[u8]) -> Option<Icon> {
    let icon_dir = ico::IconDir::read(std::io::Cursor::new(bytes)).ok()?;
    let entry = icon_dir.entries().iter().max_by_key(|e| e.width())?;
    let img = entry.decode().ok()?;
    Icon::from_rgba(img.rgba_data().to_vec(), img.width(), img.height()).ok()
}

#[tokio::main]
async fn main() -> wry::Result<()> {
    let args = Args::parse();

    let path = std::fs::canonicalize(&args.path).unwrap_or(PathBuf::from(&args.path));

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Dr.Player")
        .to_string();

    let app = Router::new().nest_service("/video.mp4", ServeFile::new(&path));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let file_url = format!("http://127.0.0.1:{}/video.mp4", port);

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

    let webview = WebViewBuilder::new(&window)
        .with_html(HTML)
        .with_ipc_handler(move |msg| {
            let _ = proxy.send_event(msg.body().to_string());
        })
        .build()?;

    let filename_js = filename.replace('\'', "\\'");
    let load_script = format!(
        "window.loadVideo('{}'); window.setTitle('{}');",
        file_url, filename_js
    );
    let _ = webview.evaluate_script(&load_script);

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
                } else if msg == "exit_fullscreen" {
                    window.set_fullscreen(None);
                } else if msg == "drag_window" {
                    let _ = window.drag_window();
                } else if msg.starts_with("resize:") {
                    let parts: Vec<&str> = msg.split(':').collect();
                    if parts.len() == 3 {
                        if let (Ok(w), Ok(h)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                            let _ = window.set_inner_size(tao::dpi::LogicalSize::new(w, h));
                        }
                    }
                }
            }
            _ => {}
        }
    })
}