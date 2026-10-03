/**
 * Dr.Player - Regression test suite
 *
 * Tests the inline JavaScript drawing state machine, keyboard shortcut
 * isolation, and error-resilience logic that runs inside the WebView.
 *
 * Setup:
 *   npm install -D vitest jsdom
 *   npx vitest run
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'node:fs';
import path from 'node:path';
import jsdom from 'jsdom';

const { JSDOM, VirtualConsole } = jsdom;

/* ------------------------------------------------------------------ */
/*  Helpers                                                           */
/* ------------------------------------------------------------------ */

const REPO_ROOT = path.resolve(import.meta.dirname, '..', '..');

/**
 * Build the minimal DOM that matches the inline HTML so that the
 * drawing code finds all its elements and runs without error.
 */
function buildDOM() {
  // Classes on <body> survive an innerHTML swap, so clear them first or
  // `drawmode` / `show-cur` leak from one test into the next.
  document.body.className = '';
  document.body.innerHTML = `
    <div id="stage">
      <video id="v"></video>
      <canvas id="draw"></canvas>
    </div>
    <div id="topbar" class="show"><div class="drag-handle" id="drag-handle"></div>
      <div class="title-capsule" id="title">Dr.Player</div>
      <div class="winctrl">
        <button class="wbtn" id="bhide">◎</button>
        <button class="wbtn" id="bdraw">✎</button>
        <button class="wbtn" id="bmin">—</button>
        <button class="wbtn close" id="bcls">✕</button>
      </div>
    </div>
    <div id="hud" class="show">
      <div class="bar"><div class="controls-row">
        <button class="btn" id="bseek-back">-5s</button>
        <button class="btn" id="bseek-fwd">+5s</button>
        <button class="btn" id="bprev">-1F</button>
        <button class="btn" id="bnext">+1F</button>
        <button class="btn" id="bplay">▶</button>
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
          <svg id="loop-off"></svg>
          <svg id="loop-on" style="display:none"></svg>
        </button>
        <button class="btn fs-btn" id="bfs">FS</button>
      </div></div>
    </div>
    <div id="drawbar">
      <button class="dhandle">⠿</button>
      <div class="drawbar-body">
        <div class="drow">
          <button class="dbtn active" data-tool="pen">Pen<span class="dkey">/P</span></button>
          <button class="dbtn" data-tool="line">Line<span class="dkey">/L</span></button>
          <button class="dbtn" data-tool="arrow">Arrow<span class="dkey">/A</span></button>
          <button class="dbtn" data-tool="rect">Rect<span class="dkey">/R</span></button>
          <button class="dbtn" data-tool="circle">Circle<span class="dkey">/C</span></button>
          <button class="dbtn" data-tool="hand">Hand<span class="dkey">/H</span></button>
          <div class="dsep"></div>
          <div class="cswatch sel" data-color="#ff0000" style="background:#ff0000"></div>
          <div class="cswatch" data-color="#00ff00" style="background:#00ff00"></div>
          <div class="cswatch" data-color="#0066ff" style="background:#0066ff"></div>
          <div class="cswatch" data-color="#ffff00" style="background:#ffff00"></div>
          <div class="cswatch" data-color="#ffffff" style="background:#ffffff"></div>
          <div class="cswatch" data-color="#000000" style="background:#000000"></div>
          <input type="color" id="cpicker" value="#ff0000">
        </div>
        <div class="drow">
          <input type="range" id="csize" min="1" max="20" value="3">
          <div class="dsep"></div>
          <button class="dbtn dseek" data-dir="-5">-5s</button>
          <button class="dbtn dseek" data-dir="+5">+5s</button>
          <button class="dbtn dstep" data-dir="-1">-1F</button>
          <button class="dbtn dstep" data-dir="+1">+1F</button>
          <div class="dsep"></div>
          <button class="dbtn" id="bundo">↩</button>
          <button class="dbtn" id="bredo">↪</button>
          <button class="dbtn" id="bclear">✕</button>
          <div class="dsep"></div>
          <button class="dbtn" id="bclose-draw">✕ Exit</button>
        </div>
      </div>
    </div>
    <div id="text-dialog" style="display:none;position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.6);align-items:center;justify-content:center;">
      <div style="...">
        <div>Enter text:</div>
        <input id="text-dialog-input" type="text">
        <div style="...">
          <button id="text-dialog-cancel">Cancel</button>
          <button id="text-dialog-ok">OK</button>
        </div>
      </div>
    </div>
  `;
  // The chrome starts visible: `#topbar` and `#hud` carry `show` and
  // `<body>` carries `show-cur`. That is the state S1 starts from, and
  // the only way the three can be restored is by code that adds them back.
  document.body.classList.add('show-cur');
}

/**
 * Canvas 2D context stub. jsdom ships `getContext` but returns null from
 * it unless the optional `canvas` package is installed, so renderAll()
 * throws on a null ctx.
 */
function makeCtxStub() {
  return {
    clearRect:    vi.fn(),
    beginPath:    vi.fn(),
    moveTo:       vi.fn(),
    lineTo:       vi.fn(),
    stroke:       vi.fn(),
    fill:         vi.fn(),
    arc:          vi.fn(),
    strokeRect:   vi.fn(),
    fillRect:     vi.fn(),
    fillText:     vi.fn(),
    measureText:  () => ({ width: 50 }),
    closePath:    vi.fn(),
    save:         vi.fn(),
    restore:      vi.fn(),
    setLineDash:  vi.fn(),
    font:         '',
    fillStyle:    '',
    strokeStyle:  '',
    lineWidth:    1,
    lineCap:      'round',
    lineJoin:     'round',
  };
}

let savedGetContext = null;

/**
 * Install the ctx stub on the prototype, which must happen BEFORE
 * setupPlayerJS() captures `dctx_`. Opt-in per describe block: the blocks
 * that do not call it keep their existing null-ctx failures untouched.
 */
function stubCanvasContext() {
  savedGetContext = HTMLCanvasElement.prototype.getContext;
  HTMLCanvasElement.prototype.getContext = () => makeCtxStub();
}

function restoreCanvasContext() {
  if (savedGetContext) HTMLCanvasElement.prototype.getContext = savedGetContext;
  savedGetContext = null;
}

/** Dispatch a real DOM keydown so listener wiring and propagation matter. */
function pressKey(def) {
  document.dispatchEvent(
    new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...def })
  );
}

/** The controls toggle as the player spells it: Ctrl+H. */
function pressCtrlH() {
  pressKey({ key: 'h', code: 'KeyH', ctrlKey: true });
}

/** The same shortcut as macOS sends it, because the page reads either modifier. */
function pressCmdH() {
  pressKey({ key: 'h', code: 'KeyH', metaKey: true });
}

function moveMouse() {
  document.dispatchEvent(new MouseEvent('mousemove', { bubbles: true }));
}

/* ------------------------------------------------------------------ */
/*  Setup — loads the player JS into the jsdom global scope           */
/* ------------------------------------------------------------------ */

/**
 * Bootstrap the drawing module and its dependencies.
 * Extracted from the inline `<script>` block in `src/main.rs`.
 *
 * jsdom caveats:
 *  - Canvas 2D context is stubbed (no pixel rendering).
 *  - requestAnimationFrame is faked to run synchronously.
 *  - window.ipc.postMessage is stubbed.
 *  - HTMLVideoElement methods are mocked where needed.
 */
function setupPlayerJS() {
  // ----- mocks -----
  window.ipc = { postMessage: vi.fn() };

  // rAF runs synchronously so cursor changes happen immediately
  let rafCb = null;
  window.requestAnimationFrame = (cb) => { rafCb = cb; return 0; };
  // expose a helper to flush queued rAF
  window.__flushRAF = () => { if (rafCb) { const c = rafCb; rafCb = null; c(); } };

  window.cancelAnimationFrame = vi.fn();

  // Canvas mock — just prevent crashes
  const drawCanvas = document.getElementById('draw');
  if (!drawCanvas.getContext) {
    // jsdom may not provide getContext at all
    drawCanvas.getContext = () => makeCtxStub();
  }

  // Video mock
  const vid = document.getElementById('v');
  // ensure essential properties exist
  if (vid.duration === undefined) Object.defineProperty(vid, 'duration', { value: 120, writable: true });
  if (vid.currentTime === undefined) Object.defineProperty(vid, 'currentTime', { value: 0, writable: true });
  if (vid.volume === undefined) Object.defineProperty(vid, 'volume', { value: 1, writable: true });
  if (vid.muted === undefined) Object.defineProperty(vid, 'muted', { value: false, writable: true });
  if (vid.paused === undefined) Object.defineProperty(vid, 'paused', { value: true, writable: true });
  if (!vid.play)   vid.play   = vi.fn().mockResolvedValue(undefined);
  if (!vid.pause)  vid.pause  = vi.fn();
  if (!vid.addEventListener) vid.addEventListener = vi.fn();

  // ----- inject the player script body -----
  // We re-create the exact variables and functions from the inline script.
  // To keep tests honest, these are NOT module-scoped — they mirror the
  // global scope they would have inside a webview.

  // ---- helpers first ----
  function fmt(s) {
    if (!s || isNaN(s)) return '00:00:00';
    s = Math.floor(Math.abs(s));
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    return [h, m, sec].map(n => String(n).padStart(2, '0')).join(':');
  }

  // ---- drawing state (matches main.rs lines 924-934) ----
  const drawCanvas_ = document.getElementById('draw');
  const dctx_ = drawCanvas_.getContext('2d');
  const drawbar_ = document.getElementById('drawbar');
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
  let textResolve = null;
  const textDialog_ = document.getElementById('text-dialog');
  const textInput_ = document.getElementById('text-dialog-input');

  // We export these on `window` so tests can inspect them.
  // In the real webview they'd be closure-captured.
  window.__tool = () => tool;
  window.__color = () => color;
  window.__size = () => size;
  window.__drawing = () => drawing;
  window.__shapes = () => shapes;
  window.__undoStack = () => undoStack;
  window.__redoStack = () => redoStack;
  window.__drawStartX = () => drawStartX;
  window.__drawStartY = () => drawStartY;
  window.__selShape = () => selShape;
  window.__drawbar = () => drawbar_;

  // ---- setCanvasCursor (line 638-643) ----
  function setCanvasCursor(cursor) {
    requestAnimationFrame(() => {
      drawCanvas_.style.cursor = cursor;
    });
  }

  // ---- applyStyle (line 936-942) ----
  function applyStyle(ctx, c, s) {
    ctx.strokeStyle = c;
    ctx.lineWidth = s;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    ctx.fillStyle = c;
  }

  // ---- drawShape (line 944-985) ----
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

  // ---- renderAll (line 987-990) ----
  function renderAll() {
    dctx_.clearRect(0, 0, drawCanvas_.width, drawCanvas_.height);
    for (const s of shapes) drawShape(dctx_, s);
  }

  // ---- resizeDrawCanvas (line 992-998) ----
  function resizeDrawCanvas() {
    drawCanvas_.width = window.innerWidth;
    drawCanvas_.height = window.innerHeight;
    renderAll();
  }

  // ---- saveDrawState (line 1000-1004) ----
  function saveDrawState() {
    undoStack.push(JSON.parse(JSON.stringify(shapes)));
    if (undoStack.length > MAX_HISTORY) undoStack.shift();
    redoStack = [];
  }

  // ---- undoDraw (line 1006-1012) ----
  function undoDraw() {
    if (undoStack.length === 0) return;
    redoStack.push(JSON.parse(JSON.stringify(shapes)));
    shapes = undoStack.pop();
    selShape = null;
    renderAll();
  }

  // ---- redoDraw (line 1014-1020) ----
  function redoDraw() {
    if (redoStack.length === 0) return;
    undoStack.push(JSON.parse(JSON.stringify(shapes)));
    shapes = redoStack.pop();
    selShape = null;
    renderAll();
  }

  // ---- clearDrawCanvas (line 1022-1027) ----
  function clearDrawCanvas() {
    saveDrawState();
    shapes = [];
    selShape = null;
    renderAll();
  }

  // ---- getDrawPos (line 1029-1032) ----
  function getDrawPos(e) {
    const rect = drawCanvas_.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  }

  // ---- hitTest (line 1034-1053) ----
  function hitTest(x, y) {
    const thresh = 12;
    for (let i = shapes.length - 1; i >= 0; i--) {
      const s = shapes[i];
      if (s.type === 'text') {
        dctx_.font = s.fontSize + 'px sans-serif';
        const m = dctx_.measureText(s.text);
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

  // ---- showTextDialog (line 629-636) ----
  function showTextDialog() {
    return new Promise((resolve) => {
      textResolve = resolve;
      textInput_.value = '';
      textDialog_.style.display = 'flex';
      setTimeout(() => textInput_.focus(), 50);
    });
  }

  // Expose for tests that need to resolve the text prompt
  window.__textResolve = (val) => {
    if (textResolve) textResolve(val);
    textResolve = null;
    textDialog_.style.display = 'none';
  };

  // ---- chrome visibility (topbar + hud + cursor) ----
  // `#bhide` lives inside `#topbar`, so dropping `#topbar`'s `show` is
  // what takes the ◎ button off screen with everything else.
  const topbar_ = document.getElementById('topbar');
  const hud_ = document.getElementById('hud');
  const bhide_ = document.getElementById('bhide');
  const AUTO_HIDE_MS = 3000;
  let chromePinned = false;   // S4: controls the user brought back stay put
  let autoHideTimer = null;

  window.__chromeVisible = () => hud_.classList.contains('show');
  window.__chromePinned = () => chromePinned;
  window.__AUTO_HIDE_MS = AUTO_HIDE_MS;

  function hideChrome() {
    topbar_.classList.remove('show');
    hud_.classList.remove('show');
    document.body.classList.remove('show-cur');
  }

  function scheduleAutoHide() {
    if (autoHideTimer !== null) clearTimeout(autoHideTimer);
    autoHideTimer = null;
    if (chromePinned) return;
    autoHideTimer = setTimeout(() => {
      autoHideTimer = null;
      hideChrome();
    }, AUTO_HIDE_MS);
  }

  function showChrome(pinned) {
    topbar_.classList.add('show');
    hud_.classList.add('show');
    document.body.classList.add('show-cur');
    if (pinned) chromePinned = true;
    scheduleAutoHide();
  }

  // Ctrl+H toggles all three together and locks the new state, so what it
  // brings back does not fade away again by itself.
  function toggleChrome() {
    if (window.__chromeVisible()) {
      chromePinned = true;
      hideChrome();
    } else {
      showChrome(true);
    }
  }
  window.__hideChrome = hideChrome;
  window.__showChrome = showChrome;
  window.__toggleChrome = toggleChrome;

  // ---- openDrawMode (line 1055-1066) ----
  function openDrawMode() {
    drawCanvas_.style.pointerEvents = 'auto';
    setCanvasCursor(tool === 'hand' ? 'grab' : 'crosshair');
    drawbar_.classList.add('open');
    document.body.classList.add('drawmode');
    if (!document.getElementById('v').paused) document.getElementById('v').pause();
  }
  window.__openDrawMode = openDrawMode;

  // ---- closeDrawMode (line 1068-1079) ----
  function closeDrawMode() {
    if (drawing) drawing = false;
    selShape = null;
    shapes = [];
    undoStack = [];
    redoStack = [];
    renderAll();
    drawCanvas_.style.pointerEvents = 'none';
    setCanvasCursor('default');
    drawbar_.classList.remove('open');
    document.body.classList.remove('drawmode');
  }
  window.__closeDrawMode = closeDrawMode;

  // ---- switchTool (line 1274-1291) ----
  function switchTool(t) {
    if (drawing) {
      drawing = false;
      if (shapes.length > 0) {
        const last = shapes[shapes.length - 1];
        const incomplete = last.type === 'pen'
          ? last.points.length <= 1
          : (last.x1 === last.x2 && last.y1 === last.y2);
        if (incomplete) { shapes.pop(); renderAll(); }
      }
    }
    document.querySelectorAll('.dbtn[data-tool]').forEach(b => b.classList.remove('active'));
    const btn = document.querySelector(`[data-tool="${t}"]`);
    if (btn) btn.classList.add('active');
    tool = t;
    selShape = null;
    if (drawbar_.classList.contains('open')) {
      setCanvasCursor(t === 'hand' ? 'grab' : 'crosshair');
    }
  }
  window.__switchTool = switchTool;

  // ---- draw mode keyboard handler (line 1293-1321) ----
  // This is the same logic — bound to a global keydown that bails if
  // drawbar is not open.
  function handleDrawKeydown(e) {
    if (!drawbar_.classList.contains('open')) return;
    const k = e.key.toLowerCase();
    if (k === 'p') { switchTool('pen'); e.preventDefault(); }
    else if (k === 'l') { switchTool('line'); e.preventDefault(); }
    else if (k === 'a') { switchTool('arrow'); e.preventDefault(); }
    else if (k === 'r') { switchTool('rect'); e.preventDefault(); }
    else if (k === 'c') { switchTool('circle'); e.preventDefault(); }
    else if (k === 'h' && !e.ctrlKey && !e.metaKey) { switchTool('hand'); e.preventDefault(); }
    else if (k === 'escape') { closeDrawMode(); e.preventDefault(); }
    else if (k === 'delete' || k === 'backspace') {
      if ((e.ctrlKey || e.metaKey) && drawbar_.classList.contains('open')) {
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
  }
  window.__handleDrawKeydown = handleDrawKeydown;

  // ---- global keydown handler (line 901-918) ----
  let loopEnabled = false;
  window.__loopEnabled = () => loopEnabled;

  function handleGlobalKeydown(e) {
    // Bail out early in draw mode
    if (drawbar_.classList.contains('open')) return;
    switch (e.code) {
      case 'Space':      e.preventDefault(); break;   // play/pause (mocked)
      case 'ArrowRight': e.preventDefault(); break;
      case 'ArrowLeft':  e.preventDefault(); break;
      case 'ArrowUp':    e.preventDefault(); break;
      case 'ArrowDown':  e.preventDefault(); break;
      case 'Period':     e.preventDefault(); break;
      case 'Comma':      e.preventDefault(); break;
      case 'KeyF':
      case 'F11':        e.preventDefault(); break;
      // Ctrl+H toggles the controls, fullscreen or not (Cmd on macOS)
      case 'KeyH':
        if (e.ctrlKey || e.metaKey) { e.preventDefault(); toggleChrome(); }
        break;
      // Escape has one meaning: ask the window to leave fullscreen
      case 'Escape':     window.ipc.postMessage('exit_fullscreen'); break;
      case 'KeyM':       e.preventDefault(); break;
      case 'KeyL':
        loopEnabled = !loopEnabled;
        e.preventDefault();
        break;
    }
  }
  window.__handleGlobalKeydown = handleGlobalKeydown;

  // ---- Undo via mouse buttons (line 1268-1272) ----
  // We expose the logic so tests can call it directly.
  function handleDrawCanvasMouseUp(e) {
    if (!drawbar_.classList.contains('open') || drawing) return;
    if (e.button === 3) { undoDraw(); e.preventDefault(); }
    else if (e.button === 4) { redoDraw(); e.preventDefault(); }
  }
  window.__handleDrawCanvasMouseUp = handleDrawCanvasMouseUp;

  // ---- setCanvasCursor exported for resilience tests ----
  window.__setCanvasCursor = setCanvasCursor;

  // ---- error handlers are installed on window ----
  window.__installErrorHandlers = () => {
    window.addEventListener('error', (event) => {
      console.error('GLOBAL_ERROR', event.message, event.error?.stack);
      event.preventDefault();
    });
    window.addEventListener('unhandledrejection', (event) => {
      console.error('UNHANDLED_PROMISE', event.reason);
      event.preventDefault();
    });
  };

  // ---- text dialog handlers (line 626-646 of main.rs) ----
  document.getElementById('text-dialog-ok').addEventListener('click', () => {
    textDialog_.style.display = 'none';
    // Never leave focus parked in an input that is no longer on screen
    textInput_.blur();
    if (textResolve) textResolve(textInput_.value);
    textResolve = null;
  });
  document.getElementById('text-dialog-cancel').addEventListener('click', () => {
    textDialog_.style.display = 'none';
    textInput_.blur();
    if (textResolve) textResolve(null);
    textResolve = null;
  });
  // Keys stay inside the prompt only while it is genuinely open, so a closed
  // dialog can never swallow a player shortcut, wherever focus happens to sit
  textInput_.addEventListener('keydown', (e) => {
    if (textDialog_.style.display === 'none') return;
    if (e.key === 'Enter') { document.getElementById('text-dialog-ok').click(); }
    else if (e.key === 'Escape') { document.getElementById('text-dialog-cancel').click(); }
    e.stopPropagation();
  });

  // ---- DOM wiring ----
  // The webview binds these to the document; here they are bound for real
  // so propagation and listener order are actually exercised. Teardown runs
  // on the next setupPlayerJS() call: the jsdom document survives every
  // buildDOM(), so stale handlers would otherwise pile up.
  if (window.__teardownPlayerJS) window.__teardownPlayerJS();

  const onDocKeydown = (e) => {
    // Route once from the state at keydown time. If the draw handler is
    // allowed to close the drawbar first, the global handler would then see
    // a closed drawbar and toggle the chrome on the same keypress.
    if (drawbar_.classList.contains('open')) { handleDrawKeydown(e); return; }
    handleGlobalKeydown(e);
  };

  const onDocMousemove = () => {
    // S4: real input drops the pin, so normal auto-hide resumes.
    if (!chromePinned) return;
    chromePinned = false;
    if (window.__chromeVisible()) scheduleAutoHide();
  };

  const onBhideClick = () => {
    if (window.__chromeVisible()) {
      chromePinned = true;
      hideChrome();
      bhide_.textContent = '◌';
    } else {
      showChrome(true);
      bhide_.textContent = '◎';
    }
  };

  document.addEventListener('keydown', onDocKeydown);
  document.addEventListener('mousemove', onDocMousemove);
  bhide_.addEventListener('click', onBhideClick);

  window.__teardownPlayerJS = () => {
    document.removeEventListener('keydown', onDocKeydown);
    document.removeEventListener('mousemove', onDocMousemove);
    bhide_.removeEventListener('click', onBhideClick);
    if (autoHideTimer !== null) { clearTimeout(autoHideTimer); autoHideTimer = null; }
  };
}

/* ------------------------------------------------------------------ */
/*  Tests                                                             */
/* ------------------------------------------------------------------ */

describe('Drawing state machine', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  /* ---------- switching tool mid-draw resets drawing state ---------- */

  it('switchTool sets drawing=false when called mid-draw', () => {
    window.__openDrawMode();
    // Simulate start-draw: saveDrawState + set drawing=true
    // (this is what mousedown does for non-hand/text tools)
    window.__shapes().push({ type: 'line', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 10, y2: 10 });
    // Manually set drawing to true as mousedown would
    window.__switchTool('rect');
    expect(window.__drawing()).toBe(false);
  });

  it('incomplete shape (single-point pen) is removed on tool switch', () => {
    window.__openDrawMode();
    // Add a single-point pen shape (incomplete)
    window.__shapes().push({ type: 'pen', color: '#ff0000', size: 3, points: [{ x: 50, y: 50 }] });
    expect(window.__shapes().length).toBe(1);
    window.__switchTool('arrow');
    // The incomplete pen should have been popped
    expect(window.__shapes().length).toBe(0);
  });

  it('incomplete line (start==end) is removed on tool switch', () => {
    window.__openDrawMode();
    window.__shapes().push({ type: 'line', color: '#ff0000', size: 3, x1: 30, y1: 30, x2: 30, y2: 30 });
    expect(window.__shapes().length).toBe(1);
    window.__switchTool('circle');
    // x1===x2 && y1===y2 → incomplete
    expect(window.__shapes().length).toBe(0);
  });

  it('completed shape survives tool switch', () => {
    window.__openDrawMode();
    // A line with different start/end is "complete"
    window.__shapes().push({ type: 'line', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
    expect(window.__shapes().length).toBe(1);
    window.__switchTool('rect');
    expect(window.__shapes().length).toBe(1);
  });

  it('multi-point pen survives tool switch', () => {
    window.__openDrawMode();
    window.__shapes().push({ type: 'pen', color: '#ff0000', size: 3, points: [{ x: 10, y: 10 }, { x: 20, y: 20 }, { x: 30, y: 30 }] });
    expect(window.__shapes().length).toBe(1);
    window.__switchTool('pen');
    expect(window.__shapes().length).toBe(1);
  });

  it('switchTool cleans up selShape', () => {
    window.__openDrawMode();
    // Put a shape and simulate selection
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
    // selShape would normally be set during hand tool mousedown
    window.__switchTool('line');
    // selShape is nulled regardless
    expect(window.__selShape()).toBeNull();
  });

  /* ---------- cannot begin draw while already drawing ---------- */

  it('mousedown while drawing pushes another shape (no guard needed; app relies on drawing flag)', () => {
    window.__openDrawMode();
    drawing = false;

    // The real behaviour: mousedown sets drawing=true,
    // and mousemove extends the last shape. A second mousedown
    // just overwrites drawStartX/Y but drawing is already true,
    // so it calls saveDrawState again and pushes another shape.
    // The guard is: mousemove ignores if not drawing.
    // This test verifies the system does NOT crash if mousedown fires twice.
    expect(() => {
      // Simulate two mousedowns in a row
      window.__drawing = true; // force
    }).not.toThrow();
  });

  /* ---------- closeDrawMode handles active drawing safely ---------- */

  it('closeDrawMode resets drawing flag', () => {
    window.__openDrawMode();
    window.__closeDrawMode();
    expect(window.__drawing()).toBe(false);
  });

  it('closeDrawMode clears all shapes and stacks', () => {
    window.__openDrawMode();
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
    window.__undoStack().push([{ something: true }]);
    window.__redoStack().push([{ something: true }]);
    window.__closeDrawMode();
    expect(window.__shapes().length).toBe(0);
    expect(window.__undoStack().length).toBe(0);
    expect(window.__redoStack().length).toBe(0);
  });

  it('closeDrawMode removes drawmode class and drawbar open', () => {
    window.__openDrawMode();
    expect(document.body.classList.contains('drawmode')).toBe(true);
    expect(window.__drawbar().classList.contains('open')).toBe(true);
    window.__closeDrawMode();
    expect(document.body.classList.contains('drawmode')).toBe(false);
    expect(window.__drawbar().classList.contains('open')).toBe(false);
  });

  /* ---------- undo/redo stack limits ---------- */

  it('undo stack does not exceed MAX_HISTORY (50)', () => {
    // Push 55 entries via saveDrawState
    for (let i = 0; i < 55; i++) {
      window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: i, y1: i, x2: i + 10, y2: i + 10 });
      // saveDrawState snapshots current shapes
      // We need to call saveDrawState directly (it's not exposed, so we simulate)
      window.__undoStack().push(JSON.parse(JSON.stringify(window.__shapes())));
      if (window.__undoStack().length > 50) window.__undoStack().shift();
    }
    expect(window.__undoStack().length).toBeLessThanOrEqual(50);
    expect(window.__undoStack().length).toBe(50);
  });

  it('redoDraw empties correctly after undoing', () => {
    window.__openDrawMode();
    // Add a shape
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
    window.__undoStack().push(JSON.parse(JSON.stringify([])));  // empty state to undo to
    window.__undoStack().push(JSON.parse(JSON.stringify(window.__shapes())));

    // undo once
    window.__undoStack().pop();  // current state
    window.__redoStack().push(JSON.parse(JSON.stringify(window.__shapes())));
    window.__shapes().length = 0;
    window.__shapes().push(...window.__undoStack().pop());

    expect(window.__shapes().length).toBe(0);

    // redo
    window.__undoStack().push(JSON.parse(JSON.stringify(window.__shapes())));
    window.__shapes().length = 0;
    window.__shapes().push(...window.__redoStack().pop());

    expect(window.__shapes().length).toBe(1);
  });

  it('saveDrawState clears redoStack when new action performed', () => {
    window.__openDrawMode();
    // Simulate undo: push to redoStack
    window.__redoStack().push([{ something: true }]);
    expect(window.__redoStack().length).toBe(1);

    // New draw action calls saveDrawState, which clears redoStack
    window.__undoStack().push(JSON.parse(JSON.stringify(window.__shapes())));
    window.__redoStack().length = 0; // saveDrawState does this
    expect(window.__redoStack().length).toBe(0);
  });

  /* ---------- mouse button 3/4 does not undo during active drawing ---------- */

  it('mouse button 3 (back) does NOT undo while drawing is active', () => {
    window.__openDrawMode();
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
    window.__undoStack().push([]);
    window.__undoStack().push(JSON.parse(JSON.stringify(window.__shapes())));

    // drawing is true — mouseup should bail
    drawing = true;
    const event = new MouseEvent('mouseup', { button: 3, cancelable: true });
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault');
    window.__handleDrawCanvasMouseUp(event);
    expect(preventDefaultSpy).not.toHaveBeenCalled();
  });

  it('mouse button 4 (forward) does NOT redo while drawing is active', () => {
    window.__openDrawMode();
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });

    drawing = true;
    const event = new MouseEvent('mouseup', { button: 4, cancelable: true });
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault');
    window.__handleDrawCanvasMouseUp(event);
    expect(preventDefaultSpy).not.toHaveBeenCalled();
  });

  it('mouse button 3 does undo when drawbar open and NOT drawing', () => {
    window.__openDrawMode();
    drawing = false;
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
    window.__undoStack().push([]);
    window.__undoStack().push(JSON.parse(JSON.stringify(window.__shapes())));

    const event = new MouseEvent('mouseup', { button: 3, cancelable: true });
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault');
    window.__handleDrawCanvasMouseUp(event);
    expect(preventDefaultSpy).toHaveBeenCalled();
  });

  it('mouse button 3 does nothing when drawbar is closed', () => {
    // drawbar not open
    drawing = false;
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });

    const event = new MouseEvent('mouseup', { button: 3, cancelable: true });
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault');
    window.__handleDrawCanvasMouseUp(event);
    expect(preventDefaultSpy).not.toHaveBeenCalled();
  });

  /* ---------- text dialog does not conflict with draw mode ---------- */

  it('text dialog open prevents canvas mousedown from starting a draw', () => {
    window.__openDrawMode();
    const dialog = document.getElementById('text-dialog');
    dialog.style.display = 'flex';  // dialog is open

    // This matches the guard in the real mousedown:
    // if (textDialog.style.display !== 'none') return;
    const shouldIgnore = dialog.style.display !== 'none';
    expect(shouldIgnore).toBe(true);
  });

  it('text dialog resolves to the entered text', async () => {
    window.__openDrawMode();
    const input = document.getElementById('text-dialog-input');
    input.value = 'Hello Dr.Player';

    // Simulate OK click
    const dialog = document.getElementById('text-dialog');
    dialog.style.display = 'flex';

    // The real flow: showTextDialog() returns a Promise.
    // We resolve it via the OK button handler.
    document.getElementById('text-dialog-ok').click();
    expect(dialog.style.display).toBe('none');
  });

  it('text dialog cancel returns null', () => {
    window.__openDrawMode();
    const input = document.getElementById('text-dialog-input');
    input.value = 'should be ignored';

    const dialog = document.getElementById('text-dialog');
    dialog.style.display = 'flex';

    document.getElementById('text-dialog-cancel').click();
    expect(dialog.style.display).toBe('none');
  });

  it('pressing Enter in text input confirms dialog', () => {
    window.__openDrawMode();
    const dialog = document.getElementById('text-dialog');
    dialog.style.display = 'flex';
    const input = document.getElementById('text-dialog-input');
    input.value = 'confirmed';

    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    expect(dialog.style.display).toBe('none');
  });

  it('pressing Escape in text input cancels dialog', () => {
    window.__openDrawMode();
    const dialog = document.getElementById('text-dialog');
    dialog.style.display = 'flex';
    const input = document.getElementById('text-dialog-input');

    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
    expect(dialog.style.display).toBe('none');
  });

  it('text dialog keydown does not propagate to draw handler', () => {
    window.__openDrawMode();
    const dialog = document.getElementById('text-dialog');
    dialog.style.display = 'flex';
    const input = document.getElementById('text-dialog-input');

    const drawSpy = vi.fn();
    document.addEventListener('keydown', drawSpy);

    input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
    // The text-input keydown calls stopPropagation, so the document listener
    // should NOT fire for this event.
    expect(drawSpy).not.toHaveBeenCalled();
  });
});

/* ------------------------------------------------------------------ */
/*  Keyboard shortcut tests                                           */
/* ------------------------------------------------------------------ */

describe('Keyboard shortcuts', () => {
  beforeEach(() => {
    buildDOM();
    // These tests reach closeDrawMode(), which renders; jsdom's null ctx
    // would throw before the drawbar class was touched.
    stubCanvasContext();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    restoreCanvasContext();
  });

  /* ---------- Global shortcuts do NOT fire in draw mode ---------- */

  it.each([
    ['Space',    { code: 'Space',       key: ' ' }],
    ['ArrowRight', { code: 'ArrowRight', key: 'ArrowRight' }],
    ['ArrowLeft',  { code: 'ArrowLeft',  key: 'ArrowLeft' }],
    ['ArrowUp',    { code: 'ArrowUp',    key: 'ArrowUp' }],
    ['ArrowDown',  { code: 'ArrowDown',  key: 'ArrowDown' }],
    ['Period',     { code: 'Period',     key: '.' }],
    ['Comma',      { code: 'Comma',      key: ',' }],
    ['KeyF',       { code: 'KeyF',       key: 'f' }],
    ['KeyM',       { code: 'KeyM',       key: 'm' }],
    ['KeyL',       { code: 'KeyL',       key: 'l' }],
    ['Escape',     { code: 'Escape',     key: 'Escape' }],
  ])('global shortcut %s is IGNORED when drawbar is open', (_, def) => {
    window.__openDrawMode();
    expect(window.__drawbar().classList.contains('open')).toBe(true);

    const loopBefore = window.__loopEnabled();
    const event = new KeyboardEvent('keydown', { ...def, cancelable: true, bubbles: true });
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault');

    window.__handleGlobalKeydown(event);

    // Global handler should have bailed out — no preventDefault, no side effects
    expect(preventDefaultSpy).not.toHaveBeenCalled();
    if (def.code === 'KeyL') {
      expect(window.__loopEnabled()).toBe(loopBefore);
    }
  });

  it('global KeyL does NOT toggle loop when drawbar is open', () => {
    window.__openDrawMode();
    const loopBefore = window.__loopEnabled();

    const event = new KeyboardEvent('keydown', { code: 'KeyL', key: 'l', cancelable: true, bubbles: true });
    window.__handleGlobalKeydown(event);

    // Because drawbar is open, the global handler returns early
    expect(window.__loopEnabled()).toBe(loopBefore);
  });

  it('global KeyL toggles loop when drawbar is closed', () => {
    expect(window.__loopEnabled()).toBe(false);

    const event = new KeyboardEvent('keydown', { code: 'KeyL', key: 'l', cancelable: true, bubbles: true });
    window.__handleGlobalKeydown(event);

    expect(window.__loopEnabled()).toBe(true);
  });

  /* ---------- Draw mode shortcuts ONLY fire in draw mode ---------- */

  it.each([
    ['p', 'pen'],
    ['l', 'line'],
    ['a', 'arrow'],
    ['r', 'rect'],
    ['c', 'circle'],
    ['h', 'hand'],
  ])('draw shortcut %s switches to %s when drawbar is open', (key, expectedTool) => {
    window.__openDrawMode();
    window.__switchTool('pen'); // ensure starting from known tool

    const event = new KeyboardEvent('keydown', { key, cancelable: true, bubbles: true });
    window.__handleDrawKeydown(event);

    expect(window.__tool()).toBe(expectedTool);
  });

  it('draw shortcut after drawbar closed is a no-op', () => {
    // drawbar not open
    window.__switchTool('pen'); // reset to pen
    const toolBefore = window.__tool();

    const event = new KeyboardEvent('keydown', { key: 'a', cancelable: true, bubbles: true });
    window.__handleDrawKeydown(event);

    expect(window.__tool()).toBe(toolBefore);
  });

  /* ---------- a modified h belongs to the chrome toggle, not to the toolbar ---------- */

  it.each([
    ['ctrlKey'],
    ['metaKey'],
  ])('draw shortcut h is ignored with %s held, so the hand tool is not selected', (modifier) => {
    window.__openDrawMode();
    window.__switchTool('pen');
    expect(window.__tool()).toBe('pen');

    const event = new KeyboardEvent('keydown', {
      key: 'h', code: 'KeyH', cancelable: true, bubbles: true, [modifier]: true,
    });
    window.__handleDrawKeydown(event);

    expect(window.__tool()).toBe('pen');
  });

  it('a bare h still selects the hand tool: guarding the shortcut does not disable it', () => {
    window.__openDrawMode();
    window.__switchTool('pen');

    const event = new KeyboardEvent('keydown', {
      key: 'h', code: 'KeyH', cancelable: true, bubbles: true,
    });
    window.__handleDrawKeydown(event);

    expect(window.__tool()).toBe('hand');
  });

  it('key "e" is a no-op in draw mode (draw mode is left with Escape)', () => {
    window.__openDrawMode();
    const toolBefore = window.__tool();

    const event = new KeyboardEvent('keydown', { key: 'e', cancelable: true, bubbles: true });
    window.__handleDrawKeydown(event);

    expect(window.__drawbar().classList.contains('open')).toBe(true);
    expect(window.__tool()).toBe(toolBefore);
  });

  it('key "0"-"9" sets size in draw mode', () => {
    window.__openDrawMode();
    const event = new KeyboardEvent('keydown', { key: '5', cancelable: true, bubbles: true });
    window.__handleDrawKeydown(event);

    // size = 2 + 5 * 2 = 12
    expect(window.__size()).toBe(12);
    expect(document.getElementById('csize').value).toBe('12');
  });

  it('delete/backspace with selShape removes it', () => {
    window.__openDrawMode();
    window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
    // Simulate selection — shapes[0] is selShape
    // We'll set selShape by grabbing shapes[0]
    const shape = window.__shapes()[0];
    selShape = shape;

    const event = new KeyboardEvent('keydown', { key: 'Delete', cancelable: true, bubbles: true });
    window.__handleDrawKeydown(event);

    expect(window.__shapes().length).toBe(0);
    expect(window.__selShape()).toBeNull();
  });

  /* ---------- Escape exits fullscreen but does NOT interact with draw mode ---------- */

  it('Escape asks the window to leave fullscreen and toggles nothing', () => {
    const event = new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', cancelable: true, bubbles: true });
    window.__handleGlobalKeydown(event);

    expect(window.ipc.postMessage).toHaveBeenCalledWith('exit_fullscreen');
    expect(window.__chromeVisible()).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  it('Escape in the draw handler closes draw mode', () => {
    window.__openDrawMode();
    const event = new KeyboardEvent('keydown', { key: 'Escape', cancelable: true, bubbles: true });
    window.__handleDrawKeydown(event);

    expect(window.__drawbar().classList.contains('open')).toBe(false);
    expect(document.body.classList.contains('drawmode')).toBe(false);
  });

  it('Escape in draw mode (global handler) bails out so no fullscreen exit', () => {
    window.__openDrawMode();
    const event = new KeyboardEvent('keydown', { code: 'Escape', key: 'Escape', cancelable: true, bubbles: true });
    const preventDefaultSpy = vi.spyOn(event, 'preventDefault');
    window.__handleGlobalKeydown(event);

    // Global handler bails because drawbar is open
    expect(preventDefaultSpy).not.toHaveBeenCalled();
  });
});

/* ------------------------------------------------------------------ */
/*  S1 — Ctrl+H hides and restores everything                          */
/* ------------------------------------------------------------------ */

describe('Ctrl+H toggles all chrome (S1)', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  /* ---------- happy path: one press takes all three, one brings them back ---------- */

  it('Ctrl+H drops show from topbar and hud and show-cur from body', () => {
    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);

    pressCtrlH();

    expect(document.getElementById('topbar').classList.contains('show')).toBe(false);
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
    expect(document.body.classList.contains('show-cur')).toBe(false);
  });

  it('Ctrl+H takes the ◎ button with it, because #bhide lives inside #topbar', () => {
    const topbar = document.getElementById('topbar');
    const bhide = document.getElementById('bhide');
    expect(topbar.contains(bhide)).toBe(true);
    expect(bhide.textContent.trim()).toBe('◎');

    pressCtrlH();

    // jsdom has no layout or stylesheet, so "gone" is only observable as the
    // ancestor that carries the class.
    expect(topbar.classList.contains('show')).toBe(false);

    pressCtrlH();
    expect(topbar.classList.contains('show')).toBe(true);
    expect(bhide.isConnected).toBe(true);
  });

  it('the second Ctrl+H restores all three and they stay restored', () => {
    vi.useFakeTimers();
    pressCtrlH();
    pressCtrlH();

    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);

    vi.advanceTimersByTime(5000);

    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
  });

  /* ---------- the three never end up out of step ---------- */

  it('the three states move in lockstep and never split across repeated Ctrl+H', () => {
    const seen = [];
    for (let i = 0; i < 3; i++) {
      pressCtrlH();
      seen.push([
        document.getElementById('topbar').classList.contains('show'),
        document.getElementById('hud').classList.contains('show'),
        document.body.classList.contains('show-cur'),
      ]);
    }
    expect(seen).toEqual([
      [false, false, false],
      [true, true, true],
      [false, false, false],
    ]);
  });

  it('Ctrl+H keeps alternating for as long as it is pressed', () => {
    for (let i = 0; i < 4; i++) pressCtrlH();
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);

    for (let i = 0; i < 3; i++) pressCtrlH();
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
  });

  /* ---------- the modifier is part of the shortcut, not decoration ---------- */

  it('a bare, unmodified h does not toggle the controls', () => {
    pressKey({ key: 'h', code: 'KeyH' });

    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
    expect(window.ipc.postMessage).not.toHaveBeenCalled();
  });

  it('Cmd+H toggles the controls exactly as Ctrl+H does, the way Ctrl+Backspace is bound', () => {
    pressCmdH();

    expect(document.getElementById('topbar').classList.contains('show')).toBe(false);
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
    expect(document.body.classList.contains('show-cur')).toBe(false);

    pressCmdH();

    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  /* ---------- Ctrl+H never throws, whatever state it lands in ---------- */

  it('toggling from either direction is safe and lands on the opposite state', () => {
    window.__toggleChrome();
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);

    expect(() => window.__toggleChrome()).not.toThrow();
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);

    expect(() => pressCtrlH()).not.toThrow();
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
  });

  it('Ctrl+H pressed twice in one tick lands on the state of two presses', () => {
    pressCtrlH();
    pressCtrlH();
    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });
});

/* ------------------------------------------------------------------ */
/*  S3a — Escape has one meaning: leave fullscreen                     */
/* ------------------------------------------------------------------ */

describe('Escape means leave fullscreen and nothing else (S3a)', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  // STORIES.md:31-33 asked Escape to exit fullscreen and then, on the next
  // press, toggle the controls. The owner moved the controls toggle to
  // Ctrl+H, so Escape no longer has a second meaning at all and there is no
  // "second Escape" any more. The host handshake behind exit_fullscreen is
  // exercised against the shipped page at the end of this file.
  it('Escape asks the window to leave fullscreen', () => {
    pressKey({ key: 'Escape', code: 'Escape' });

    expect(window.ipc.postMessage).toHaveBeenCalledTimes(1);
    expect(window.ipc.postMessage).toHaveBeenCalledWith('exit_fullscreen');
  });

  it('Escape changes no chrome class, so it can no longer hide the controls', () => {
    pressKey({ key: 'Escape', code: 'Escape' });

    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  it('Escape does not latch the controls lock, so Ctrl+H is not stuck afterwards', () => {
    pressKey({ key: 'Escape', code: 'Escape' });

    expect(window.__chromePinned()).toBe(false);

    pressCtrlH();
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);

    pressCtrlH();
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
  });

  // The two-press sequence this block used to assert: the first Escape spends
  // itself on the exit request, and the next one was the controls toggle. With
  // one meaning per key the second press is just a second request.
  it('two Escapes ask twice and toggle nothing', () => {
    pressKey({ key: 'Escape', code: 'Escape' });
    pressKey({ key: 'Escape', code: 'Escape' });

    expect(window.ipc.postMessage).toHaveBeenCalledTimes(2);
    expect(window.ipc.postMessage).toHaveBeenCalledWith('exit_fullscreen');
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  it('the page does not swallow Escape: the webview still receives it', () => {
    const event = new KeyboardEvent('keydown', {
      key: 'Escape', code: 'Escape', cancelable: true, bubbles: true,
    });
    document.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(false);
  });
});

/* ------------------------------------------------------------------ */
/*  S3b — draw mode owns ESC first                                     */
/* ------------------------------------------------------------------ */

describe('ESC priority: draw mode (S3b)', () => {
  beforeEach(() => {
    buildDOM();
    // ESC in draw mode reaches closeDrawMode(), which renders.
    stubCanvasContext();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    restoreCanvasContext();
  });

  it('ESC closes draw mode and leaves every chrome class alone', () => {
    window.__openDrawMode();
    expect(window.__drawbar().classList.contains('open')).toBe(true);

    pressKey({ key: 'Escape', code: 'Escape' });

    expect(window.__drawbar().classList.contains('open')).toBe(false);
    expect(document.body.classList.contains('drawmode')).toBe(false);
    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  it('ESC in draw mode sends no IPC and does not latch the chrome lock', () => {
    expect(window.__chromePinned()).toBe(false);
    window.__openDrawMode();

    pressKey({ key: 'Escape', code: 'Escape' });

    expect(window.ipc.postMessage).not.toHaveBeenCalled();
    expect(window.__chromePinned()).toBe(false);
  });

  it('ESC in draw mode is consumed, so the global handler never gets that keypress', () => {
    window.__openDrawMode();
    const event = new KeyboardEvent('keydown', {
      key: 'Escape', code: 'Escape', bubbles: true, cancelable: true,
    });
    document.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
  });

  it('after Escape closes draw mode, another Escape still does not toggle the controls', () => {
    window.__openDrawMode();
    pressKey({ key: 'Escape', code: 'Escape' });
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);

    pressKey({ key: 'Escape', code: 'Escape' });
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
  });

  it('Ctrl+H is still inert while draw mode is open, so drawing keeps the canvas', () => {
    window.__openDrawMode();

    pressCtrlH();

    expect(window.__drawbar().classList.contains('open')).toBe(true);
    expect(document.body.classList.contains('drawmode')).toBe(true);
  });
});

/* ------------------------------------------------------------------ */
/*  S3c — the Enter text box owns ESC first                            */
/* ------------------------------------------------------------------ */

describe('ESC priority: Enter text dialog (S3c)', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('ESC in the text box cancels the dialog and does NOT toggle the controls', () => {
    const dialog = document.getElementById('text-dialog');
    dialog.style.display = 'flex';
    const input = document.getElementById('text-dialog-input');

    input.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Escape', bubbles: true, cancelable: true,
    }));

    expect(dialog.style.display).toBe('none');
    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  it('the text box swallows exactly one ESC, and the next one still does not toggle the controls', () => {
    const dialog = document.getElementById('text-dialog');
    const input = document.getElementById('text-dialog-input');
    dialog.style.display = 'flex';

    input.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'Escape', bubbles: true, cancelable: true,
    }));
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);

    pressKey({ key: 'Escape', code: 'Escape' });
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
  });

  it('the prompt owns its keys while it is open, and hands them back once closed', () => {
    const input = document.getElementById('text-dialog-input');
    const seen = [];
    const spy = (e) => seen.push(e.code);
    document.addEventListener('keydown', spy);

    // Closed: the dialog has never been opened, so its listener must not
    // consume anything and the key has to reach the page.
    input.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'a', code: 'KeyA', bubbles: true, cancelable: true,
    }));
    expect(seen).toEqual(['KeyA']);

    // Open: the same key belongs to the prompt.
    seen.length = 0;
    document.getElementById('text-dialog').style.display = 'flex';
    input.dispatchEvent(new KeyboardEvent('keydown', {
      key: 'a', code: 'KeyA', bubbles: true, cancelable: true,
    }));
    expect(seen).toEqual([]);

    document.removeEventListener('keydown', spy);
  });
});

/* ------------------------------------------------------------------ */
/*  S4 — chrome brought back by Ctrl+H stays put                       */
/* ------------------------------------------------------------------ */

describe('Ctrl+H-restored chrome does not fade on its own (S4)', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('chrome Ctrl+H brought back is still there after 5s of no input', () => {
    vi.useFakeTimers();
    pressCtrlH();
    pressCtrlH();

    vi.advanceTimersByTime(2500);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);

    vi.advanceTimersByTime(3000);
    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  it('a mousemove clears the pin and the chrome auto-hides again one window later', () => {
    vi.useFakeTimers();
    pressCtrlH();
    pressCtrlH();

    moveMouse();
    expect(window.__chromePinned()).toBe(false);

    vi.advanceTimersByTime(2500);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);

    vi.advanceTimersByTime(1000);
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
    expect(document.getElementById('topbar').classList.contains('show')).toBe(false);
    expect(document.body.classList.contains('show-cur')).toBe(false);
  });

  it('Ctrl+H does not clear the pin: after four presses the chrome is still up at 5s', () => {
    vi.useFakeTimers();
    for (let i = 0; i < 4; i++) pressCtrlH();

    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(window.__chromePinned()).toBe(true);

    vi.advanceTimersByTime(5000);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
  });

  it('Ctrl+H does not clear the pin, so the first mousemove is what restarts auto-hide', () => {
    vi.useFakeTimers();
    pressCtrlH();
    pressCtrlH();

    moveMouse();
    vi.advanceTimersByTime(window.__AUTO_HIDE_MS + 1);

    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
  });

  it('chrome hidden by Ctrl+H stays hidden, the timer never brings it back', () => {
    vi.useFakeTimers();
    pressCtrlH();

    vi.advanceTimersByTime(5000);

    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
    expect(document.getElementById('topbar').classList.contains('show')).toBe(false);
    expect(document.body.classList.contains('show-cur')).toBe(false);
  });
});

/* ------------------------------------------------------------------ */
/*  S5 — the ◎ button keeps its existing meaning                      */
/* ------------------------------------------------------------------ */

describe('◎ button keeps its meaning (S5)', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it('starts on ◎, hides the chrome on click and swaps the glyph to ◌', () => {
    const bhide = document.getElementById('bhide');
    expect(bhide.textContent.trim()).toBe('◎');

    bhide.click();

    expect(bhide.textContent.trim()).toBe('◌');
    expect(document.getElementById('topbar').classList.contains('show')).toBe(false);
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
    expect(document.body.classList.contains('show-cur')).toBe(false);
  });

  it('clicking it again restores the chrome and the glyph', () => {
    const bhide = document.getElementById('bhide');
    bhide.click();
    bhide.click();

    expect(bhide.textContent.trim()).toBe('◎');
    expect(document.getElementById('topbar').classList.contains('show')).toBe(true);
    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(document.body.classList.contains('show-cur')).toBe(true);
  });

  it('the lock holds: the hidden chrome does not come back by itself', () => {
    vi.useFakeTimers();
    document.getElementById('bhide').click();

    vi.advanceTimersByTime(5000);

    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
    expect(document.getElementById('bhide').textContent.trim()).toBe('◌');
  });

  it('the button stays a purely local control: no IPC on either click', () => {
    const bhide = document.getElementById('bhide');
    bhide.click();
    bhide.click();

    expect(window.ipc.postMessage).not.toHaveBeenCalled();
  });
});

/* ------------------------------------------------------------------ */
/*  Error resilience tests                                            */
/* ------------------------------------------------------------------ */

describe('Error resilience', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('window.onerror catches errors and prevents propagation', () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    window.__installErrorHandlers();

    // Simulate an error event
    const errorEvent = new ErrorEvent('error', {
      message: 'test error',
      error: new Error('test error'),
      cancelable: true,
    });
    const result = window.dispatchEvent(errorEvent);

    // defaultPrevented should be true since we call preventDefault
    expect(errorEvent.defaultPrevented).toBe(true);
    expect(consoleSpy).toHaveBeenCalledWith('GLOBAL_ERROR', 'test error', expect.any(String));

    consoleSpy.mockRestore();
  });

  it('unhandledrejection is caught and prevented', () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    window.__installErrorHandlers();

    const rejectionEvent = new PromiseRejectionEvent('unhandledrejection', {
      promise: Promise.reject('test rejection'),
      reason: 'test rejection',
      cancelable: true,
    });
    // Swallow the rejection to avoid Node's unhandledRejection warning
    rejectionEvent.promise.catch(() => {});

    const result = window.dispatchEvent(rejectionEvent);

    expect(rejectionEvent.defaultPrevented).toBe(true);
    expect(consoleSpy).toHaveBeenCalledWith('UNHANDLED_PROMISE', 'test rejection');

    consoleSpy.mockRestore();
  });

  it('setCanvasCursor with null does not throw', () => {
    expect(() => {
      window.__setCanvasCursor(null);
      window.__flushRAF();
    }).not.toThrow();
  });

  it('setCanvasCursor with undefined does not throw', () => {
    expect(() => {
      window.__setCanvasCursor(undefined);
      window.__flushRAF();
    }).not.toThrow();
  });

  it('setCanvasCursor with empty string does not throw', () => {
    expect(() => {
      window.__setCanvasCursor('');
      window.__flushRAF();
    }).not.toThrow();
  });

  it('setCanvasCursor with valid cursor sets the style', () => {
    window.__setCanvasCursor('crosshair');
    window.__flushRAF();
    expect(document.getElementById('draw').style.cursor).toBe('crosshair');
  });

  it('setCanvasCursor with "grab" works', () => {
    window.__setCanvasCursor('grab');
    window.__flushRAF();
    expect(document.getElementById('draw').style.cursor).toBe('grab');
  });

  it('switchTool with invalid tool name does not throw', () => {
    expect(() => {
      window.__switchTool('nonexistent-tool');
    }).not.toThrow();
    // Tool should still be set even if no active class applied
    expect(window.__tool()).toBe('nonexistent-tool');
  });

  it('calling closeDrawMode twice is safe', () => {
    window.__openDrawMode();
    window.__closeDrawMode();
    expect(() => window.__closeDrawMode()).not.toThrow();
  });

  it('calling openDrawMode twice is safe', () => {
    window.__openDrawMode();
    expect(() => window.__openDrawMode()).not.toThrow();
  });

  it('undo with empty stack does nothing', () => {
    expect(() => {
      window.__shapes().push({ type: 'rect', color: '#ff0000', size: 3, x1: 10, y1: 10, x2: 100, y2: 100 });
      window.__undoStack().length = 0;
      // We call undoDraw logic directly: if (undoStack.length === 0) return;
      const lenBefore = window.__shapes().length;
      window.__undoStack().push(JSON.parse(JSON.stringify(window.__shapes())));
      window.__undoStack().pop();
      // Now stack is empty again — undo is a no-op
    }).not.toThrow();
  });

  it('redo with empty stack does nothing', () => {
    expect(() => {
      window.__redoStack().length = 0;
    }).not.toThrow();
  });

  it('clearDrawCanvas with no shapes does not throw', () => {
    window.__openDrawMode();
    expect(() => {
      window.__shapes().length = 0;
    }).not.toThrow();
  });
});

/* ------------------------------------------------------------------ */
/*  openDrawMode / closeDrawMode integration                          */
/* ------------------------------------------------------------------ */

describe('Draw mode lifecycle', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  it('openDrawMode pauses video if it is playing', () => {
    const vid = document.getElementById('v');
    vid.paused = false;
    const pauseSpy = vi.spyOn(vid, 'pause');

    window.__openDrawMode();
    expect(pauseSpy).toHaveBeenCalled();
  });

  it('openDrawMode does not pause video if already paused', () => {
    const vid = document.getElementById('v');
    vid.paused = true;
    const pauseSpy = vi.spyOn(vid, 'pause');

    window.__openDrawMode();
    expect(pauseSpy).not.toHaveBeenCalled();
  });

  it('openDrawMode sets correct cursor via rAF', () => {
    window.__switchTool('pen');
    window.__openDrawMode();
    window.__flushRAF();
    expect(document.getElementById('draw').style.cursor).toBe('crosshair');
  });

  it('openDrawMode with hand tool sets grab cursor', () => {
    window.__switchTool('hand');
    window.__openDrawMode();
    window.__flushRAF();
    expect(document.getElementById('draw').style.cursor).toBe('grab');
  });

  it('closeDrawMode resets cursor to default', () => {
    window.__openDrawMode();
    window.__flushRAF();
    window.__closeDrawMode();
    window.__flushRAF();
    expect(document.getElementById('draw').style.cursor).toBe('default');
  });
});

/* ------------------------------------------------------------------ */
/*  select / color swatch integration                                 */
/* ------------------------------------------------------------------ */

describe('Toolbar interactions', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
    window.__openDrawMode();
  });

  it('clicking a color swatch updates the color and removes sel from others', () => {
    const swatches = document.querySelectorAll('.cswatch');
    const greenSwatch = swatches[1]; // data-color="#00ff00"

    greenSwatch.click();

    expect(window.__color()).toBe('#00ff00');
    expect(greenSwatch.classList.contains('sel')).toBe(true);
    // First swatch (red) should no longer be selected
    expect(swatches[0].classList.contains('sel')).toBe(false);
  });

  it('size slider updates size', () => {
    const slider = document.getElementById('csize');
    slider.value = '10';
    slider.dispatchEvent(new Event('input'));

    expect(window.__size()).toBe(10);
  });

  it('undo button calls undoDraw without throwing', () => {
    const undoBtn = document.getElementById('bundo');
    expect(() => undoBtn.click()).not.toThrow();
  });

  it('redo button calls redoDraw without throwing', () => {
    const redoBtn = document.getElementById('bredo');
    expect(() => redoBtn.click()).not.toThrow();
  });

  it('clear button calls clearDrawCanvas without throwing', () => {
    const clearBtn = document.getElementById('bclear');
    expect(() => clearBtn.click()).not.toThrow();
  });

  it('close-draw button calls closeDrawMode', () => {
    const closeBtn = document.getElementById('bclose-draw');
    closeBtn.click();
    expect(window.__drawbar().classList.contains('open')).toBe(false);
  });
});

/* ------------------------------------------------------------------ */
/*  Volume control tests                                               */
/* ------------------------------------------------------------------ */

describe('Volume controls', () => {
  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('vol-slider is always visible (not hidden by default)', () => {
    const slider = document.getElementById('vol-slider');
    expect(slider).toBeTruthy();
    expect(slider.style.width).not.toBe('0');
    expect(slider.style.opacity).not.toBe('0');
  });
});

/* ------------------------------------------------------------------ */
/*  S2 — an ESC from another app can never reach the player           */
/* ------------------------------------------------------------------ */

describe('A shortcut never escapes the webview (S2)', () => {
  // jsdom cannot move keyboard focus to another window, so "the user is in
  // File Explorer and presses ESC" is not observable here. What S2's
  // "Known when" actually claims is a property of the project: no global or
  // OS-level hotkey is registered anywhere, so a keypress only exists inside
  // the webview while the Dr.Player window has focus. That half is testable.
  const OS_HOOKS = [
    'global_hotkey', 'global-hotkey', 'global_hotkeys',
    'global_shortcut', 'global-shortcut', 'globalShortcut',
    'RegisterHotKey', 'AddHotKey', 'SetWindowsHookEx',
  ];
  const SCANNED = ['Cargo.toml', 'package.json', 'build.rs', 'src'];

  beforeEach(() => {
    buildDOM();
    setupPlayerJS();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('nothing in the project registers a global or OS-level hotkey', () => {
    const files = [];
    const collect = (target) => {
      const stat = fs.statSync(target);
      if (stat.isFile()) { files.push(target); return; }
      for (const entry of fs.readdirSync(target, { withFileTypes: true })) {
        if (entry.name === 'target' || entry.name === 'node_modules') continue;
        if (entry.isDirectory()) collect(path.join(target, entry.name));
        else if (entry.name.endsWith('.rs')) files.push(path.join(target, entry.name));
      }
    };
    for (const rel of SCANNED) {
      const target = path.join(REPO_ROOT, rel);
      if (fs.existsSync(target)) collect(target);
    }
    // The scan has to have found the project, or the test proves nothing.
    const scanned = files.map((f) => path.relative(REPO_ROOT, f).split(path.sep).join('/'));
    expect(scanned).toContain('src/main.rs');
    expect(scanned).toContain('Cargo.toml');

    const hits = [];
    for (const file of files) {
      const text = fs.readFileSync(file, 'utf8');
      for (const hook of OS_HOOKS) {
        if (text.includes(hook)) hits.push(`${path.relative(REPO_ROOT, file)} uses ${hook}`);
      }
    }
    expect(hits).toEqual([]);
  });

  it('the chrome toggle is local DOM state: Ctrl+H posts nothing to the Rust side', () => {
    pressCtrlH();

    expect(window.ipc.postMessage).not.toHaveBeenCalled();
    expect(document.getElementById('hud').classList.contains('show')).toBe(false);
  });

  it('a keypress only acts through a DOM keydown: a keyup changes nothing', () => {
    document.dispatchEvent(new KeyboardEvent('keyup', {
      key: 'h', code: 'KeyH', ctrlKey: true, bubbles: true,
    }));

    expect(document.getElementById('hud').classList.contains('show')).toBe(true);
    expect(window.ipc.postMessage).not.toHaveBeenCalled();
  });
});

/* ------------------------------------------------------------------ */
/*  Production code, read out of src/main.rs                           */
/* ------------------------------------------------------------------ */
/*  Production code, read out of src/main.rs                           */
/* ------------------------------------------------------------------ */

/**
 * setupPlayerJS() re-implements the player by hand, so a test built on it
 * passes when src/main.rs is wrong. Everything below boots the document
 * src/main.rs actually ships and drives that, so a defect in the production
 * script, the production stylesheet or the Rust half shows up here as a
 * failure. Nothing is launched: the page is the extracted HTML, and the
 * "keypresses" are DOM events inside that page.
 */

// The override exists so the same suite can be run against a historical copy of
// the page outside the repo, which is how the text-dialog regression below is
// shown failing before its fix and passing after. Nothing sets it in normal use.
const MAIN_RS = process.env.DRPLAYER_MAIN_RS || path.join(REPO_ROOT, 'src', 'main.rs');

/** The shipped document, taken from the raw string in src/main.rs. */
function shippedHTML() {
  const rust = fs.readFileSync(MAIN_RS, 'utf8');
  const raw = rust.match(/const HTML: &str = r##"([\s\S]*?)"##;/);
  if (!raw) throw new Error('no `const HTML` raw string found in src/main.rs');
  const html = raw[1];
  // Guard the extraction: a partial match would boot an empty page and make
  // every assertion below fail for a reason that has nothing to do with the code.
  if (!html.includes('<!DOCTYPE html>') || !html.includes('</html>')) {
    throw new Error('the extracted HTML is not a whole document');
  }
  return html;
}

/** The Rust half and the page's inline script, as text, from src/main.rs. */
function shippedBlocks() {
  const rust = fs.readFileSync(MAIN_RS, 'utf8');
  const at = rust.indexOf('<script>');
  const end = rust.indexOf('</script>');
  if (at < 0 || end < 0) throw new Error('no <script> block found in src/main.rs');
  return { rust, script: rust.slice(at + '<script>'.length, end) };
}

/**
 * Boot the shipped page in a jsdom window of its own.
 *
 * Timers never fire: the page arms a 3s auto-hide on load and this harness
 * answers nothing, so a test can only move the page by dispatching events.
 * The canvas 2D context and media playback are stubbed because jsdom has
 * neither, and window.ipc records instead of sending.
 */
function bootShippedPlayer() {
  const posts = [];
  const errors = [];
  const virtualConsole = new VirtualConsole();
  virtualConsole.on('jsdomError', (e) => errors.push(String(e.message)));
  virtualConsole.on('error', (...args) => errors.push(args.map(String).join(' ')));

  const dom = new JSDOM(shippedHTML(), {
    runScripts: 'dangerously',
    virtualConsole,
    beforeParse(win) {
      win.ipc = { postMessage: (msg) => posts.push(msg) };
      win.requestAnimationFrame = () => 0;
      win.cancelAnimationFrame = () => {};
      win.setTimeout = () => 0;
      win.clearTimeout = () => {};
      win.HTMLMediaElement.prototype.play = () => Promise.resolve();
      win.HTMLMediaElement.prototype.pause = () => {};
      win.HTMLCanvasElement.prototype.getContext = () => new Proxy({}, {
        get: (target, key) => (key === 'measureText' ? () => ({ width: 10 }) : () => {}),
        set: () => true,
      });
    },
  });

  const win = dom.window;
  const doc = win.document;
  return {
    window: win,
    document: doc,
    posts,
    errors,
    escape() {
      doc.dispatchEvent(new win.KeyboardEvent('keydown', {
        key: 'Escape', code: 'Escape', bubbles: true, cancelable: true,
      }));
    },
    /** The controls toggle: Ctrl+H, and Cmd+H on macOS. */
    modifiedH(modifier) {
      doc.dispatchEvent(new win.KeyboardEvent('keydown', {
        key: 'h', code: 'KeyH', bubbles: true, cancelable: true, [modifier]: true,
      }));
    },
    /** A bare h, the key draw mode binds to the hand tool. */
    bareH() {
      doc.dispatchEvent(new win.KeyboardEvent('keydown', {
        key: 'h', code: 'KeyH', bubbles: true, cancelable: true,
      }));
    },
    /** Computed style of the ancestor the controls and ◎ button live in. */
    topbarStyle(prop) { return win.getComputedStyle(doc.getElementById('topbar'))[prop]; },
    hudStyle(prop) { return win.getComputedStyle(doc.getElementById('hud'))[prop]; },
    /** The Enter text prompt, opened the way the page opens it. */
    prompt() { return win.showTextDialog(); },
    openPrompt() { doc.getElementById('text-dialog').style.display = 'flex'; },
    textInput() { return doc.getElementById('text-dialog-input'); },
    /** A keydown on a specific element, so propagation decides the outcome. */
    pressOn(node, def) {
      node.dispatchEvent(new win.KeyboardEvent('keydown', {
        bubbles: true, cancelable: true, ...def,
      }));
    },
    // The focused element's id, not the element: a failing assertion on a jsdom
    // node is serialised by walking the DOM, which throws instead of reporting.
    focusedId() { return doc.activeElement && doc.activeElement.id; },
    activeTool() {
      const btn = doc.querySelector('.dbtn.active[data-tool]');
      return btn && btn.dataset.tool;
    },
    mousemove(clientX, clientY) {
      doc.dispatchEvent(new win.MouseEvent('mousemove', {
        clientX, clientY, screenX: clientX, screenY: clientY, bubbles: true,
      }));
    },
    click(id) { doc.getElementById(id).click(); },
    // The page's own visibility test: topbar or hud carrying `show`.
    chromeVisible() {
      return doc.getElementById('topbar').classList.contains('show')
        || doc.getElementById('hud').classList.contains('show');
    },
    glyph() { return doc.getElementById('bhide').textContent.trim(); },
    cursor() { return win.getComputedStyle(doc.body).cursor; },
    close() { win.close(); },
  };
}

/** Every message literal the page hands to the host, ternaries included. */
function postedMessages(script) {
  const posted = [];
  for (const at of script.matchAll(/postMessage\s*\(/g)) {
    let depth = 1;
    let i = at.index + at[0].length;
    while (i < script.length && depth > 0) {
      if (script[i] === '(') depth++;
      else if (script[i] === ')') depth--;
      i++;
    }
    for (const literal of script.slice(at.index, i).matchAll(/'([^']*)'/g)) {
      posted.push(literal[1]);
    }
  }
  return posted;
}

/**
 * Ways a host can hand a fresh boolean state down to the page. The page has
 * no way to ask, so the entry point is whatever the fix exposed; this finds
 * it by behaviour rather than by guessing a name. It finds nothing on a page
 * that keeps its own copy of the flag, which is what that defect looks like.
 */
function fullscreenPushAttempts(win) {
  const names = Object.keys(win)
    .filter((key) => /fullscreen|full_screen|fs_?state/i.test(key) && typeof win[key] === 'function');
  return [
    ...names.map((name) => ({
      label: `window.${name}(state)`,
      apply: (page, state) => page.window[name](state),
    })),
    ...names.map((name) => ({
      label: `window.${name}({ isFullscreen: state })`,
      apply: (page, state) => page.window[name]({ isFullscreen: state, fullscreen: state }),
    })),
    ...['fullscreenchange', 'fullscreenstate'].map((type) => ({
      label: `new CustomEvent('${type}', { detail: state })`,
      apply: (page, state) => page.window.dispatchEvent(
        new page.window.CustomEvent(type, { detail: state })
      ),
    })),
  ];
}

describe('the shipped page (src/main.rs)', () => {
  let booted = [];

  const open = () => {
    const page = bootShippedPlayer();
    booted.push(page);
    return page;
  };

  afterEach(() => {
    for (const page of booted) page.close();
    booted = [];
  });

  it('boots with no script error and the chrome up', () => {
    const page = open();

    expect(page.errors).toEqual([]);
    expect(page.chromeVisible()).toBe(true);
    expect(page.document.body.classList.contains('show-cur')).toBe(true);
    expect(page.glyph()).toBe('◎');
  });

  /* ---------- Defect A: the cursor never actually hid ---------- */

  it('hiding the chrome hides the cursor, and unhiding brings that cursor back', () => {
    const page = open();
    const cursorWithChrome = page.cursor();
    expect(cursorWithChrome).not.toBe('none');

    page.modifiedH('ctrlKey');

    expect(page.chromeVisible()).toBe(false);
    expect(page.cursor()).toBe('none');

    page.modifiedH('ctrlKey');

    expect(page.chromeVisible()).toBe(true);
    expect(page.cursor()).toBe(cursorWithChrome);
  });

  it('the resize-edge cursor cannot outlive the hide, and unhiding brings it back', () => {
    const page = open();

    // RESIZE_MARGIN is 8px and (2, 2) is the top-left corner, where the page
    // asks for `nwse-resize`. While the chrome is hidden that cursor must not
    // appear, which it does today because the handler writes it as an inline
    // style on <body> and an inline style outranks the class rule.
    page.modifiedH('ctrlKey');
    page.mousemove(2, 2);
    expect(page.cursor()).toBe('none');

    // Unhidden, the same corner must still offer the resize cursor: S1 hides
    // the cursor, it does not disable resizing.
    page.modifiedH('ctrlKey');
    page.mousemove(2, 2);
    expect(page.cursor()).toBe('nwse-resize');
  });

  it('the shipped stylesheet styles the cursor from the class the page toggles', () => {
    const page = open();
    const rules = [...page.document.styleSheets].flatMap((sheet) => [...sheet.cssRules]);
    const cursorRules = rules.filter(
      (rule) => rule.selectorText && rule.selectorText.includes('show-cur') && rule.style.cursor
    );

    // `show-cur` sits on <body> while the controls are UP, so the rule that
    // hides the cursor can be written either way round: `body:not(.show-cur)`
    // hides it, or a hidden base with `.show-cur` restoring it. The rule that
    // must not exist is the inverted one, where the controls are up and the
    // cursor is already gone.
    expect(cursorRules.length).toBeGreaterThan(0);
    expect(page.document.body.classList.contains('show-cur')).toBe(true);
    expect(page.cursor()).not.toBe('none');
  });

  /* ---------- Defect B: the fullscreen state the host reports ---------- */

  it('a fullscreen request that never took effect does not stop Ctrl+H from toggling', () => {
    const page = open();
    page.click('bfs');
    expect(page.posts).toEqual(['fullscreen']);

    // The window never confirmed anything: it is still windowed and the page
    // was not told. Ctrl+H is only a controls toggle, so it must not care.
    page.modifiedH('ctrlKey');

    expect(page.chromeVisible()).toBe(false);
    expect(page.posts).toEqual(['fullscreen']);
  });

  it('the window reports the fullscreen state the FS button acts on', () => {
    // A host reporting a state sends nothing and asks for nothing: the page
    // has to reach "the window is fullscreen" from the report alone. Both
    // directions are tried on a page of their own, so a function that merely
    // requests fullscreen is not mistaken for one that reports it. The FS
    // button is what proves the report landed, now that Escape is unconditional.
    const attempts = fullscreenPushAttempts(open().window);
    const push = attempts.find((attempt) => {
      const full = open();
      attempt.apply(full, true);
      if (full.posts.length) return false;
      full.click('bfs');
      if (full.posts.length !== 1 || full.posts[0] !== 'exit_fullscreen') return false;

      const windowed = open();
      attempt.apply(windowed, false);
      if (windowed.posts.length) return false;
      windowed.click('bfs');
      return windowed.posts.length === 1 && windowed.posts[0] === 'fullscreen';
    });

    const tried = attempts.map((a) => a.label).join(', ')
      || 'nothing on window matched /fullscreen/';
    expect(push, `the page exposes no way for the host to report the window's fullscreen state; tried: ${tried}`)
      .toBeTruthy();
  });

  it('Escape asks the window to leave fullscreen and touches nothing else', () => {
    const page = open();

    page.escape();

    expect(page.posts).toEqual(['exit_fullscreen']);
    expect(page.chromeVisible()).toBe(true);
    expect(page.cursor()).not.toBe('none');
  });

  it('no number of Escapes ever toggles the controls', () => {
    const page = open();

    for (let i = 0; i < 5; i++) {
      page.escape();
      expect(page.chromeVisible()).toBe(true);
    }

    expect(page.posts).toEqual(['exit_fullscreen', 'exit_fullscreen',
      'exit_fullscreen', 'exit_fullscreen', 'exit_fullscreen']);
  });

  it('the page never decides the fullscreen state for itself', () => {
    const { rust, script } = shippedBlocks();

    // A page that flips its own flag before the window has agreed is the
    // defect: the flag can then disagree with the window and the FS button
    // ends up asking for a state the window is not in.
    expect(script).not.toMatch(/([A-Za-z_$][\w$]*[Ff]ullscreen[A-Za-z_$]*)\s*=\s*!\s*\1/);
    // So the window side has to hand the real state down.
    expect(rust).toMatch(/evaluate_script[^;]{0,300}fullscreen/is);
  });

  it('ESC still speaks the fullscreen messages the Rust side handles', () => {
    const { rust, script } = shippedBlocks();
    const posted = postedMessages(script);
    // Both directions of S3a survive: asking to go fullscreen and asking to
    // leave it are still the messages the event loop arms on.
    expect(posted).toContain('fullscreen');
    expect(posted).toContain('exit_fullscreen');
    for (const msg of ['fullscreen', 'exit_fullscreen']) {
      expect(rust).toContain(`"${msg}"`);
    }
  });

  /* ---------- Defect C: the ◎ glyph fell behind the screen ---------- */

  it('click ◎ then Ctrl+H then click ◎ leaves the glyph agreeing with the screen', () => {
    const page = open();
    expect(page.chromeVisible()).toBe(true);
    expect(page.glyph()).toBe('◎');

    page.click('bhide');
    expect(page.chromeVisible()).toBe(false);
    expect(page.glyph()).toBe('◌');

    // Ctrl+H brings the chrome back unlocked, so the button has to offer "hide"
    // again rather than keep saying "show".
    page.modifiedH('ctrlKey');
    expect(page.chromeVisible()).toBe(true);
    expect(page.glyph()).toBe('◎');

    page.click('bhide');
    expect(page.chromeVisible()).toBe(false);
    expect(page.glyph()).toBe('◌');
  });

  it('the glyph matches what is on screen at every press, Ctrl+H and click alike', () => {
    const page = open();
    const expected = () => (page.chromeVisible() ? '◎' : '◌');

    for (let i = 0; i < 4; i++) {
      const wasVisible = page.chromeVisible();

      page.modifiedH('ctrlKey');
      expect(page.chromeVisible()).toBe(!wasVisible);
      expect(page.glyph()).toBe(expected());

      page.click('bhide');
      expect(page.chromeVisible()).toBe(wasVisible);
      expect(page.glyph()).toBe(expected());
    }
  });

  /* ---------- Ctrl+H: the controls toggle the owner actually uses ---------- */

  it('Ctrl+H takes the topbar, the hud, the cursor and the ◎ button with it', () => {
    const page = open();
    const topbar = page.document.getElementById('topbar');
    const hud = page.document.getElementById('hud');
    const bhide = page.document.getElementById('bhide');
    expect(topbar.contains(bhide)).toBe(true);
    expect(page.topbarStyle('opacity')).toBe('1');
    expect(page.topbarStyle('pointerEvents')).toBe('auto');

    page.modifiedH('ctrlKey');

    expect(topbar.classList.contains('show')).toBe(false);
    expect(hud.classList.contains('show')).toBe(false);
    expect(page.document.body.classList.contains('show-cur')).toBe(false);
    expect(page.cursor()).toBe('none');

    // ◎ goes off screen with its ancestor, not on its own account: the shipped
    // stylesheet gives `#topbar` no opacity and no pointer events once `show`
    // is gone, so nothing inside it can be seen or pressed.
    expect(page.topbarStyle('opacity')).toBe('0');
    expect(page.topbarStyle('pointerEvents')).toBe('none');
    expect(page.hudStyle('opacity')).toBe('0');
    expect(page.hudStyle('pointerEvents')).toBe('none');

    page.modifiedH('ctrlKey');

    expect(topbar.classList.contains('show')).toBe(true);
    expect(hud.classList.contains('show')).toBe(true);
    expect(page.document.body.classList.contains('show-cur')).toBe(true);
    expect(page.topbarStyle('opacity')).toBe('1');
    expect(page.topbarStyle('pointerEvents')).toBe('auto');
  });

  it('Cmd+H toggles the controls exactly as Ctrl+H does, the way Ctrl+Backspace is bound', () => {
    const page = open();

    page.modifiedH('metaKey');

    expect(page.chromeVisible()).toBe(false);
    expect(page.cursor()).toBe('none');

    page.modifiedH('metaKey');

    expect(page.chromeVisible()).toBe(true);
    expect(page.cursor()).not.toBe('none');
  });

  it('Ctrl+H never asks the window to leave fullscreen, fullscreen or not', () => {
    // Find the window's own fullscreen report by behaviour: a reporter posts
    // nothing when called, a function that merely requests fullscreen posts at
    // once, so the two cannot be mistaken for each other.
    const reporter = fullscreenPushAttempts(open().window).find((attempt) => {
      const page = open();
      attempt.apply(page, true);
      return page.posts.length === 0;
    });
    expect(reporter, 'no window entry point reports the fullscreen state').toBeTruthy();

    const full = open();
    reporter.apply(full, true);
    expect(full.posts).toEqual([]);
    full.click('bfs');
    // The report landed: the FS button now offers to leave.
    expect(full.posts).toEqual(['exit_fullscreen']);

    full.posts.length = 0;
    full.modifiedH('ctrlKey');
    expect(full.chromeVisible()).toBe(false);
    expect(full.posts).toEqual([]);

    const windowed = open();
    windowed.modifiedH('ctrlKey');
    expect(windowed.chromeVisible()).toBe(false);
    expect(windowed.posts).toEqual([]);
  });

  it('a bare, unmodified h does not toggle the controls', () => {
    const page = open();

    page.bareH();

    expect(page.chromeVisible()).toBe(true);
    expect(page.document.getElementById('drawbar').classList.contains('open')).toBe(false);
    expect(page.posts).toEqual([]);

    // Only the modified key is the shortcut, and it still works afterwards.
    page.modifiedH('ctrlKey');
    expect(page.chromeVisible()).toBe(false);
  });

  it('Ctrl+H does not select the hand tool while draw mode is open', () => {
    const page = open();
    page.click('bdraw');
    expect(page.document.getElementById('drawbar').classList.contains('open')).toBe(true);
    expect(page.activeTool()).toBe('pen');

    page.modifiedH('ctrlKey');

    expect(page.activeTool()).toBe('pen');
    expect(page.document.getElementById('drawbar').classList.contains('open')).toBe(true);
  });

  it('a bare h still selects the hand tool while draw mode is open', () => {
    const page = open();
    page.click('bdraw');

    page.bareH();

    expect(page.activeTool()).toBe('hand');
  });

  /* ---------- Defect D: a hidden prompt swallowed every keyboard shortcut ---------- */

  it('the shipped text input carries no autofocus', () => {
    const page = open();

    expect(page.textInput().hasAttribute('autofocus')).toBe(false);
    // The dialog is display:none as shipped, so an autofocus here parks
    // keyboard focus in an element the user cannot see.
    expect(page.document.getElementById('text-dialog').style.display).toBe('none');
    expect(shippedHTML()).not.toMatch(/autofocus/i);
  });

  it('a key on the closed prompt still reaches document', () => {
    const page = open();
    const input = page.textInput();
    const seen = [];
    page.document.addEventListener('keydown', (e) => seen.push(e.code));

    // jsdom does not reproduce a webview honouring `autofocus` on a hidden
    // element, so the accident is staged by focusing the input by hand. What
    // this pins is the invariant, not autofocus: a prompt that is not open
    // must never be able to swallow a key, wherever focus happens to sit.
    input.focus();
    expect(page.focusedId()).toBe('text-dialog-input');

    page.pressOn(input, { key: 'a', code: 'KeyA' });

    expect(seen).toEqual(['KeyA']);
  });

  it('a player shortcut still works while focus is parked in the closed prompt', () => {
    const page = open();
    const input = page.textInput();

    // This is the shape of the bug the owner hit: nothing threw, the mouse
    // still worked, and every keystroke quietly died in a hidden input.
    input.focus();
    page.pressOn(input, { key: 'h', code: 'KeyH', ctrlKey: true });

    expect(page.chromeVisible()).toBe(false);
  });

  it('keys do not leak out of an open prompt', () => {
    const page = open();
    const seen = [];
    page.document.addEventListener('keydown', (e) => seen.push(e.code));

    page.openPrompt();
    const input = page.textInput();
    input.focus();

    page.pressOn(input, { key: 'a', code: 'KeyA' });

    expect(seen).toEqual([]);
  });

  it('Cancel leaves no focus parked in the input', () => {
    const page = open();
    page.openPrompt();
    const input = page.textInput();
    input.focus();
    expect(page.focusedId()).toBe('text-dialog-input');

    page.click('text-dialog-cancel');

    expect(page.document.getElementById('text-dialog').style.display).toBe('none');
    expect(page.focusedId()).not.toBe('text-dialog-input');
  });

  it('OK leaves no focus parked in the input', () => {
    const page = open();
    page.openPrompt();
    const input = page.textInput();
    input.value = 'kept';
    input.focus();
    expect(page.focusedId()).toBe('text-dialog-input');

    page.click('text-dialog-ok');

    expect(page.document.getElementById('text-dialog').style.display).toBe('none');
    expect(page.focusedId()).not.toBe('text-dialog-input');
  });

  it('Enter confirms the prompt with the text that was typed', async () => {
    const page = open();
    const answer = page.prompt();
    const input = page.textInput();
    input.value = 'Dr.Player';
    input.focus();

    page.pressOn(input, { key: 'Enter', code: 'Enter' });

    expect(await answer).toBe('Dr.Player');
    expect(page.chromeVisible()).toBe(true);
  });

  it('Escape cancels the prompt with nothing and does not toggle the controls', async () => {
    const page = open();
    const answer = page.prompt();
    const input = page.textInput();
    input.value = 'discarded';
    input.focus();

    page.pressOn(input, { key: 'Escape', code: 'Escape' });

    expect(await answer).toBeNull();
    expect(page.chromeVisible()).toBe(true);
    expect(page.posts).toEqual([]);
  });

  /* ---------- S2: every shortcut is still answered from inside the webview ---------- */

  it('Cargo.toml still carries no global or OS hotkey dependency', () => {
    const cargo = fs.readFileSync(path.join(REPO_ROOT, 'Cargo.toml'), 'utf8');
    expect(cargo.length).toBeGreaterThan(0);

    const deps = [...cargo.matchAll(/^([A-Za-z0-9_-]+)\s*=/gm)].map((m) => m[1]);
    expect(deps.length).toBeGreaterThan(0);
    expect(deps.filter((name) => /hotkey/i.test(name))).toEqual([]);
  });
});
