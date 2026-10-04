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

/* Opaque, for the same reason as the bottom chrome: a 12% white wash has nothing behind it to be
   read against, so the caption would end up sitting on whatever frame is playing. */
.title-capsule {
    background: #16161a;
    border: 1px solid #2e2e34;
    border-radius: 10px;
    padding: 6px 16px;
    font-size: 14px;
    font-weight: 500;
    color: #e8e8ea;
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
    /* Neither this row nor anything in it can give up width, and that is the whole fix for the
       ellipses. The shrink used to be shared: a button had a 28px width and no flex-shrink of its
       own, so a narrow window or a long caption squeezed it on the main axis while its height
       stayed at 28, and a 50% radius on a box narrower than it is tall is an ellipse. The caption
       is the one item on this bar that still shrinks, and it is the right one to shrink:
       overflow: hidden drops its automatic minimum width to zero, and truncating a filename is
       what a caption is for. margin-left: auto holds this row at the right-hand end when the
       caption is display: none, which space-between does not do on its own with one item left in
       flow: without it the window buttons slide left onto the drag handle at exactly the widths
       where that handle has become the whole bar. */
    flex: none;
    margin-left: auto;
}
/* Same flat chip as the bottom row, for the same reason, and still 28 by 28: this is the window's
   own row and its target size was never part of the bottom chrome rework. flex: none is the
   button's half of the rule above and it says the same thing: 28 is the size, not the starting
   point of a negotiation. */
.wbtn {
    flex: none;
    width: 28px; height: 28px;
    border-radius: 50%;
    background: #303036;
    border: 1px solid #5a5a63;
    color: #e8e8ea;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s;
}
.wbtn:hover {
    background: #3a3a41;
    border-color: #6b6b74;
    color: #fff;
}
.wbtn.close:hover {
    background: #e81123;
    border-color: #e81123;
    color: #fff;
}
/* The icons on this row are the loop icon in the loop's convention: 14 by 14 out of a 14-unit
   viewBox, currentColor so the colour and the hover state are the button's own, and the shape
   carried by strokes rather than fills so a 1.4 line stays a line at this size. Sized here rather
   than left to .btn because .btn is not this row, and 14 is what clears a 28 round chip on all
   sides without the mark touching the border. */
.wbtn svg {
    width: 14px; height: 14px;
    display: block;
}

/* ==================== UPDATE PROMPT ==================== */
/* One chip between the caption and the window buttons, and being inside #topbar rather than beside
   it is what makes "never shown while the chrome is hidden" a fact about the tree instead of
   something to remember: the bar fades out after three seconds and this goes with it, and draw
   mode takes the whole bar away with the one display: none it already applies. It is display: none
   until the window says a newer release exists, and there is no timer, no poll and no retry behind
   that: one answer per launch, which is all this is allowed to do.

   The chip is the window's own language rather than a new one: 28 tall like the six buttons beside
   it, radius 7 and a 1px #5a5a63 edge like every chip on the bottom row, and the #303036 fill those
   chips use, so a prompt about an update does not look like a dialog laid over the video. Centred in
   the bar's 48 it runs from 10 to 38 down, the band the window buttons occupy, which is 2px clear of
   the 8px resize margin along the top edge and 3px clear of the 41 the bottom row's block reaches:
   a press on this chip can neither resize the window nor land on the row.

   97 wide, which is the two 1px borders, the two 8px paddings, the 14 of arrow, two 5px gaps,
   "Update" at 12px Segoe UI and the 18 of the dismiss box. The width arithmetic that spends that
   is on the 600px rule below, which is the only rule it takes part in.

   Contrast, computed rather than guessed. #e8e8ea on #303036 is 10.7:1, the same two colours the
   bottom row's chips already use, over the 4.5:1 SC 1.4.3 asks of the word and over the 3:1 SC
   1.4.11 asks of the arrow beside it; hover is #3a3a41 with #fff on it at 11.3:1. The one boundary
   here with no determinate neighbour is the chip's own outer edge, which can land on any frame, and
   the #5a5a63 border is what carries it: 6.8:1 on a white frame and 3.1:1 on a black one, so it
   clears 3:1 at both ends of the range rather than only against a middle one. Opaque fill, no glass
   and nothing here that depends on what is playing underneath. */
.update-pill {
    display: none;
    align-items: center;
    gap: 5px;
    height: 28px;
    padding: 0 8px;
    margin-left: 10px;
    border: 1px solid #5a5a63;
    border-radius: 7px;
    background: #303036;
    color: #e8e8ea;
    font-size: 12px;
    font-weight: 500;
    /* flex: none is this chip's half of the rule on the window buttons: 28 and 97 are the sizes,
       not the starting points of a negotiation, and the caption is what gives up width. z-index 2
       puts it above the drag handle, which fills the whole bar underneath it. */
    flex: none;
    position: relative;
    z-index: 2;
    pointer-events: auto;
}
.update-pill.show {
    display: flex;
}
.update-pill:hover {
    background: #3a3a41;
    border-color: #6b6b74;
}
/* Two buttons inside the chip, both taking their colour from it rather than carrying a fill of
   their own, which is what signalling hover with the fill alone means here. Native buttons so both
   are announced as buttons and both are reachable by Tab, which costs no binding: Enter and Space
   activate a button everywhere else in this chrome already. */
.update-open, .update-x {
    display: flex;
    align-items: center;
    background: none;
    border: none;
    padding: 0;
    color: inherit;
    font: inherit;
    cursor: pointer;
}
.update-open {
    gap: 5px;
}
/* The up arrow, 14 by 14 out of a 14-unit viewBox with the shape carried by strokes, which is what
   every other mark on this bar does, and currentColor so the glyph takes the chip's own colour and
   its hover. */
.update-open svg {
    width: 14px; height: 14px;
    display: block;
}
.update-x {
    width: 18px; height: 18px;
    justify-content: center;
}

/* ==================== BOTTOM CHROME: ONE ROW ==================== */
/* #hud is the one element that carries the show state and the whole row is inside it, so
   showControls and hideControls keep writing one class on one element and the chrome can never come
   back with half of it. It is inset on all four sides and pins its child to the bottom, so the row,
   backing and padding both, sits on the window's last pixel at any height, and it paints nothing
   itself: the backing belongs to the row, because the row is the one element that covers the whole
   block, and #hud spanning the window cannot be the thing that is opaque. */
#hud {
    position: absolute;
    inset: 0;
    pointer-events: none;
    opacity: 0;
    transition: opacity 0.25s;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-end;
    z-index: 10;
}
#hud.show {
    opacity: 1;
    pointer-events: auto;
}

/* The row, in the markup order: play/pause, elapsed, track, remaining, then the two seek steps, the
   two frame steps, the rate group, volume, loop and fullscreen. One flat level of siblings, plain
   flex and no order, so this markup is the visual order and the focus order alike and a CSS order
   cannot put one of the controls out of step with the others. The rate group is the one wrapper on
   the row, and its own rule below says why; it carries no order either. Play/pause leads because a
   pass across VLC, mpv, Plex, Jellyfin and Microsoft's own guidance found it immediately left of
   the timeline in every one of them and in none after, and the owner has asked for that position
   twice. Fullscreen is last because that is the one position worth defending.

   The background is on this element and nowhere else, so the block is opaque from its top edge to
   the window's last pixel: the padding is inside the background, so the controls can stand off the
   edges and the backing behind them still runs to the bottom of the window, with nothing left below
   it for the frame to show through. Opaque rather than tinted, because at alpha 1 the frame playing
   underneath adds nothing to what is painted here: the contrast of every glyph and every readout
   here is then fixed by the two declared colours alone and survives a white frame as well as a
   black one. Ten steps off pure black is dark enough to stay out of the picture's way and light
   enough to read as a surface rather than as a hole cut in the frame.

   Nine a side and nine a foot, and the number is the window's: RESIZE_MARGIN is 8 and the edge
   handler claims any press within it of a border, so a control has to end at least 8px short of
   every edge or a press on it resizes the window instead of pressing it. Nine is the smallest whole
   number that clears eight, one pixel of slack, and that is what keeps a fractional pointer
   coordinate or a half-pixel border out of the resize zone. It is padding rather than a handler on
   the row, and the difference is the whole point: a mousedown on the row has to swallow every press
   on it, the row spans the full width, so it takes the bottom edge and both bottom corners with it
   and the window can no longer be resized from below while the chrome is up. Nine a foot leaves that
   band bare instead, so a press on the backing below the controls reaches the edge handler on its
   own and only a press within 8px of the bottom starts a resize. The one pixel of slack is dead to a
   press, which is the price of the clearance and is not worth reclaiming.
   #seekbar-container and #vol-slider keep the stopPropagation they have always had, which is why
   scrubbing and volume still work with the bar sitting 9px off the bottom of the window.

   Four a head is not resize margin but air, and it is the inside of the owner's 3 to 5: a control
   flush with the top of its own block reads as cropped by it, which is the same complaint as one
   flush with the bottom, and it is the padding that fixes both. The 10px gap between the chips
   stays: they were never touching each other, they were pressed against the edges of the bar.

   41 tall, which is 4 plus the tallest control plus 9 and nothing else: a button is 28 counting its
   own 1px border a side and #bplay is 26, the track's container stretches to the row's own content
   height and floors itself at the same 28, the rate field is 22, and a readout is a 12px line box
   with no height and no line-height of its own, so none of them can put a pixel under the buttons or
   push one above them. */
.controls-row {
    background: #111114;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 9px 9px;
}

/* Play/pause is the one control that has to survive every window size, including the ones that drop
   the buttons to its right, so its glyph is a step larger than the text buttons' at 14px. The chip
   itself is 30 by 26, a step under the 34 by 28 every other button here is held to and clear of
   WCAG 2.2 SC 2.5.8's 24 by 24 floor by 6px across and 2px down. It is carried by its place at the
   head of the row and by that glyph rather than by being the largest chip on it, and stepping it
   down under the rest evens the row's rhythm instead of announcing one button louder than the eleven
   beside it. min-width is restated because .btn's 34px floor would otherwise clamp the width back
   up, and the padding goes so the glyph is centred in the smaller chip rather than measured against
   8px a side it no longer has. */
#bplay {
    font-size: 14px;
    width: 30px;
    min-width: 30px;
    height: 26px;
    padding: 0;
}

/* Two readouts bracketing the track: the bracket only means anything next to the timeline it
   describes, and each holds a 62px floor so the digits cannot jitter sideways as they change. The
   floor is what the narrow-width rule gives up, not this one: they are the first thing on the row
   that is worth shrinking, because the track is what has to stay pressable. */
.seek-time {
    font-family: 'SF Mono', 'Consolas', monospace;
    font-size: 12px;
    color: #e8e8ea;
    min-width: 62px;
    text-align: center;
    white-space: nowrap;
}

/* The track is the control, and this container is the full height of the row's content box, so a
   scrub is a press anywhere across 28px rather than on the 4px line itself. min-height with
   align-self: stretch rather than a fixed height, so the hit area is the row's own height and can
   never be less than it: raise a button and the track follows. The floor is the buttons' own 28,
   so the track is never the smaller target even before the stretch is taken into account, and it is
   the same number the row's content height is made of, so the track cannot be what drives that
   height either: it follows the buttons instead of leading them. min-width 0 lets the track give up
   width instead of refusing to, which is what keeps every button on screen as the window narrows;
   the width breakpoints are what decide when it has given up enough. Both the click and the drag
   measure against this element's own getBoundingClientRect and normalise the pointer into 0 to 1, so
   whatever sits to the left of it moves the rectangle without moving the mapping: the track's zero
   is its own left edge wherever that lands. */
.seekbar-container {
    flex: 1;
    min-width: 0;
    align-self: stretch;
    min-height: 28px;
    display: flex;
    align-items: center;
    cursor: pointer;
    position: relative;
}
/* 35% white on the #111114 backing, which composites to #646466 and is 3.2:1 on it: the 3:1 a
   control has to reach against the surface it sits on. It is the unfilled part of the track, not an
   indicator, but it is what tells the owner there is a timeline to press. */
.seek-track {
    width: 100%;
    height: 4px;
    background: rgba(255,255,255,0.35);
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

/* Flat chips on the opaque backing, no tint of their own: a translucent fill over a translucent bar
   was only ever readable because of what showed through, and nothing shows through any more. The
   glyph is what identifies a button and it is 10.7:1 on the fill; the 1px border is a quiet 1.9:1
   against it, a line of demarcation rather than an outline to read at a glance, and reaching 3:1 on
   the border itself would need a near-#797979 edge that is louder than the rest of this chrome.
   Hover and press are signalled by the fill alone, so no shadow is needed to lift them. */
.btn {
    background: #303036;
    border: 1px solid #5a5a63;
    color: #e8e8ea;
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
    min-width: 34px;
    height: 28px;
}
.btn:hover {
    background: #3a3a41;
    border-color: #6b6b74;
    color: #fff;
}
.btn:active {
    transform: scale(0.95);
    background: #232328;
}

/* ==================== PLAYBACK SPEED: MINUS, FIELD, PLUS ==================== */
/* One group of three items with a unit label hanging off the field, and it is the one wrapper on
   this row. It exists for exactly one reason: the unit has to sit outside the field or it would be
   typed into, so the field needs a neighbour, and a flat row would then have to spell the group's
   internal gap out on three of the row's own items. Plain flex and no order inside it, so the same
   one-level-of-siblings argument that holds the row together holds here too, and flex none so the
   field is never the thing that gives up width: the track is, which is what it is for. Positioned,
   because the note below hangs off its top edge. */
.rate-group {
    display: flex;
    align-items: center;
    gap: 4px;
    position: relative;
    flex: 0 0 auto;
}

/* A minus and a plus, and the reason is legibility rather than width, which is why the old comment
   on the chevrons got it backwards. ‹ and › read as skip and the owner rejected them for it; « and »
   are worse, because VLC and WMP spend those on speed while every other player spends them on skip
   to the start and skip to the end; and ⏩ is fast-forward. Minus and plus flanking the number is
   what YouTube shipped in 2024 and what the WAI-ARIA APG spinbutton pattern draws, which is what
   this is. 16px so the mark is readable at this row's height, and free: the chip is still .btn's
   34 by 28 because a 16px minus needs 5 of the 16 the padding and border leave and a plus needs 10.
   tabindex -1 takes both out of the Tab sequence, where the field's own arrow keys already cover
   what these do, and leaves them reachable by pointer. */
#bslower, #bfaster {
    font-size: 16px;
}

/* The field is the readout and the control in one, so it is always on and always holds the rate,
   1.00 included: a speed control that shows nothing until you have already changed the speed is a
   control you have to guess the state of. A plain text input rather than type=number, which
   silently discards a value it considers invalid, which is exactly the value somebody is here to
   type, and which grows spinners that have no place in a 41px row.

   Fixed at six characters rather than sized to whatever it holds, which is both the jitter floor
   and the jitter ceiling: 1.00 is four, 16.00 is five and 0.0625 is six, so six is the widest value
   the control can be given and the width is the same whatever the rate is, so nothing to the right
   of it moves and the row cannot widen on a long value. Six ch is six digits of this element's own
   monospace whatever the machine has, and a value that will not fit scrolls inside the field rather
   than out of it. 22 tall so it cannot make the row taller than the 28 of the buttons beside it.

   user-select text is here because html, body carry user-select: none, which would make the field's
   own contents unselectable and so impossible to edit by hand. SC 1.4.11 has to reach 3:1 for the
   focus indicator and #e00 on this field's own #1c1c20 is 4.1:1. */
.rate-input {
    width: 6ch;
    min-width: 6ch;
    max-width: 6ch;
    height: 22px;
    padding: 0 3px;
    border: 1px solid #5a5a63;
    border-radius: 5px;
    background: #1c1c20;
    color: #e8e8ea;
    font-family: 'SF Mono', 'Consolas', monospace;
    font-size: 12px;
    text-align: center;
    cursor: text;
    outline: none;
    user-select: text;
    -webkit-user-select: text;
}
.rate-input:focus {
    border-color: #e00;
}

/* The unit, a sibling of the field and not inside it, so the number goes in alone and 0.0625 can be
   typed as typed. The same 12px monospace and the same colour as the two time readouts, so it reads
   as part of a number rather than as a fourth control. A label and never a target: pointer-events
   none, so a press beside the field reaches the row's own backing rather than a box that is not
   there, and none of the eight-pixel edge bands reach into it either. */
.rate-x {
    font-family: 'SF Mono', 'Consolas', monospace;
    font-size: 12px;
    color: #e8e8ea;
    pointer-events: none;
}

/* Why a rate was refused, above the row rather than in it: the 41px of this row holds the
   controls and nothing else, and a message inside the group would widen the group the width
   arithmetic below is built on. Absolute, so it costs that arithmetic nothing at all, and pinned 32
   above the group's own bottom edge, which is the 9 of margin plus the 4 of air plus the 28 of the
   controls, so its box starts on the row's top edge and stands up over the picture the way a
   tooltip does. Empty, which is nearly all the time, it is an empty span with no box and so draws
   nothing, and it stays in the DOM and in the accessibility tree empty, which is what lets
   role=status announce the text when it arrives. Same flat opaque backing as the row, no glass, and
   pointer-events none so a press on it reaches the picture rather than a box that is not a
   control. */
.rate-note {
    position: absolute;
    bottom: 32px;
    left: 0;
    padding: 4px 8px;
    background: #111114;
    border: 1px solid #2e2e34;
    border-radius: 7px;
    font-size: 12px;
    color: #e8e8ea;
    white-space: nowrap;
    pointer-events: none;
    z-index: 1;
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

/* 35% white on the same backing as the seek track, for the same reason: #646466, 3.2:1. */
.vol-slider {
    width: 60px;
    height: 3px;
    background: rgba(255,255,255,0.35);
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
/* Loop on is a solid red chip with a bright red edge rather than a tint and a glow: a 30% red wash
   composites to #582426 over this backing, which is 1.1:1 against the chip it is meant to replace,
   and the glow had nothing left to glow against. The edge is what marks the state and it is 3.1:1
   on the fill, which is the 3:1 a state indicator has to reach, and the glyph differs as well. */
.btn-loop.active {
    background: #8c2626;
    border-color: #ff6b6b;
    color: #fff;
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
    background: #111114;
    border: 1px solid #2e2e34;
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

/* ==================== KEYBOARD SHORTCUTS OVERLAY ==================== */
/* A top-level sibling of #hud, not a child of it, and that placement is load-bearing rather than
   tidiness: the focusin guard at the foot of this file blurs anything focused inside a hidden
   #hud, and the chrome auto-hides after three seconds, so a panel inside #hud would have its
   focus thrown out of it and then vanish with it. Fixed over the whole viewport at z 200, above
   the text dialog's 100 and the draw bar's 30, because it is modal and nothing behind it is
   meant to be reached. */
.keys-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: none;
    align-items: center;
    justify-content: center;
    padding: 12px;
    /* The dim is the picture being dimmed, not a surface of its own. The panel on top of it is
       opaque, so every colour inside the panel is fixed by its own declared values and does not
       depend on what happens to be playing underneath. The one thing the panel cannot fix is the
       colour immediately outside its own edge, which is this at 0.75 over whatever frame is
       playing: 75% is the deepest dim that still reads as the video underneath rather than as a
       black window, and the worst case it leaves is a white frame composited to #404040. */
    background: rgba(0,0,0,0.75);
}
.keys-overlay.open {
    display: flex;
}

/* Opaque #111114, the same flat backing as .controls-row and #drawbar, and opaque for the same
   reason they are: a rgba panel over arbitrary video has no determinate background colour, so
   nothing inside it has a determinate contrast ratio at all.

   The colours and what they reach, computed rather than guessed. On this #111114, L 0.0057:
   #e8e8ea is 15.4:1 and #fff 18.9:1 for the keys, the title and the group headings; #c9c9cf is
   11.4:1 for the descriptions, the focus line and the footer hint. #e8e8ea on the #1c1c20 key
   cap is 13.9:1 and on the #303036 Close button 10.7:1, all of them over the 4.5:1 SC 1.4.3
   asks of text.

   The panel's own edge is the one boundary that has no determinate neighbour, so it is the
   border's job and the border is the lightest thing in here: #9a9aa2 is 6.8:1 on this backing and
   3.7:1 on that #404040 worst case, which is the 3:1 SC 1.4.11 asks of a component boundary.
   The app's own #6b6b74 cannot do it, and no dim can make it: a dark panel on a darkened
   backdrop is dark on dark at every alpha, so the darker the dim the worse the edge gets, which
   is why this is a lighter rule on the outside and not a darker one. Inside the panel the
   #6b6b74 chips are measured against this backing and clear 3.6:1, and the #e00 focus ring is
   4.2:1 on it. The two #3a3a41 rules under the header and over the footer are dividers at
   1.7:1 and are nothing else: they separate regions, they identify no control and carry no
   state, which is all SC 1.4.11 covers. No translucent hairline stands in for any of the three.

   The height cap is the other half of the job. The window has no minimum and can be a few
   hundred pixels tall, and it is frameless, so content that overflowed would be painted outside
   the visible area with no way to reach it. max-height caps the panel against the viewport,
   overflow hidden clips whatever does not fit, and the scroll region below is the only thing in
   here that ever scrolls. */
.keys-panel {
    background: #111114;
    border: 1px solid #9a9aa2;
    border-radius: 12px;
    box-shadow: 0 16px 48px rgba(0,0,0,0.6);
    color: #e8e8ea;
    font-size: 12.5px;
    text-align: left;
    width: min(560px, 100%);
    max-height: calc(100vh - 24px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

/* Header and footer are outside the scroll region, so the title and the way out are in the same
   place at a window 200 tall as at one 1000 tall. flex none on both, because a header that
   shrank to fit would be a title with its top cut off rather than a shorter header. */
.keys-head {
    flex: none;
    padding: 14px 18px 12px;
    border-bottom: 1px solid #3a3a41;
}
.keys-title {
    font-size: 15px;
    font-weight: 600;
    color: #fff;
}

/* The one line the panel takes focus on, and it is a p and not the Close button on purpose: a
   dialog that opens on its own dismiss control announces "Close, button" and the list behind it
   is never read. No aria-describedby either, for the same reason: with structured content
   underneath, a description attribute announces the whole list as one unbroken string. */
.keys-intro {
    flex: none;
    padding: 10px 18px 6px;
    font-size: 12px;
    color: #c9c9cf;
}

/* The scrollable region is focusable, because a scroll only the wheel can drive is SC 2.1.1
   Keyboard, Level A. tabindex 0 on the container and not on the list inside it, so Tab reaches
   the scroll once rather than once per row. min-height 0 is the flex floor: without it a tall
   list refuses to shrink below its own content and pushes the panel past the max-height above,
   which is the overflow this window could not show.

   The scrollbar is hidden and the scroll is not. The owner saw the native bar down the
   right-hand side of the list, with an arrow button at each end of it, and called it ugly, and
   a frameless window that styles no scrollbar of its own gets exactly that bar. Two properties
   carry it and both are declared, because they are the pair: scrollbar-width is the standard
   one and keeps the behaviour whatever the engine does with its own pseudo-elements, and
   ::-webkit-scrollbar is the pseudo-element this webview has always taken the rule from.
   Neither of them touches overflow-y, so the box still scrolls and still has a height to scroll
   within, and tabindex 0 stays on it, so the arrow keys and the page keys still reach the rows
   below the fold. What goes is the bar, its thumb and its two buttons, and nothing that
   scrolls. */
.keys-scroll {
    overflow-y: auto;
    min-height: 0;
    padding: 8px 18px 12px;
    scrollbar-width: none;
}
.keys-scroll::-webkit-scrollbar {
    display: none;
}
.keys-scroll:focus {
    outline: 2px solid #e00;
    outline-offset: -2px;
}

.keys-groups {
    list-style: none;
    padding: 0;
    display: grid;
    grid-template-columns: 1fr;
    gap: 14px 28px;
}
.keys-group-title {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #e8e8ea;
    padding-bottom: 5px;
}
.keys-rows {
    list-style: none;
    padding: 0;
}
/* The row is the grid: description on the left, keys on the right in a column sized by the widest
   of them, so the eye scans down one edge instead of hunting across for the keys. baseline on
   both axes so a two-line description keeps its first line level with its cap. */
.keys-row {
    display: grid;
    grid-template-columns: 1fr auto;
    column-gap: 14px;
    align-items: baseline;
    padding: 3px 0;
    color: #c9c9cf;
}
.keys-keys {
    display: flex;
    gap: 3px;
    align-items: baseline;
    justify-content: flex-end;
    white-space: nowrap;
}
/* Monospace on the key column only, which is what lets it read as a column, and it carries no ARIA
   role of its own, so the description beside it is what says what the key does. */
.keys-key {
    font-family: 'SF Mono', 'Consolas', monospace;
    font-size: 11px;
    color: #e8e8ea;
    background: #1c1c20;
    border: 1px solid #6b6b74;
    border-radius: 4px;
    padding: 1px 5px;
    text-align: right;
}

/* The footer names the two dismissal routes that are not the button and not Escape, so all four
   are discoverable from inside the panel rather than only from the keyboard. */
.keys-foot {
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 10px 18px 14px;
    border-top: 1px solid #3a3a41;
    font-size: 11.5px;
    color: #c9c9cf;
}
.keys-close {
    flex: none;
    font: inherit;
    font-weight: 500;
    color: #e8e8ea;
    background: #303036;
    border: 1px solid #6b6b74;
    border-radius: 7px;
    padding: 6px 14px;
    cursor: pointer;
}
.keys-close:hover {
    background: #3a3a41;
}
.keys-close:focus {
    outline: 2px solid #e00;
    outline-offset: 2px;
}

/* Two group columns when the window is short and one when it is tall, and it is height that
   decides because at a short height the constraint is how much list there is: six groups in two
   columns is half the height, and at a tall height one column is the easier read. Two media
   queries rather than a collapsing grid because the panel's height is the window's height and
   there is nothing for a grid to measure. The width rule comes after the height one, so a window
   that is both short and narrow lands on one column: two columns of rows at 300 wide is
   unreadable, and a list that scrolls is not. */
@media (max-height: 620px) {
    .keys-groups { grid-template-columns: 1fr 1fr; }
}
@media (max-width: 520px) {
    .keys-groups { grid-template-columns: 1fr; }
}

/* ==================== COMPACT LAYOUT FOR SMALL WINDOWS ==================== */
/* The window has no minimum size any more, so the block has to survive being squeezed on both axes.
   Width first, every item at its own minimum in the row's order: play at 30, the two time readouts
   at their 62px floors, -5s at 34.5 and +5s at 37.5, -1F at 35.1 and +1F at 38.1 (each glyph at
   12px plus 8px of padding and a 1px border a side), the rate group at 138, volume at 83 (18px of
   emoji, a 5px gap and the 60px slider), loop and fullscreen at their 34px floors where a 14px
   glyph needs only 32, and the track at nothing, which is the one item here with no floor of its
   own. The group is 138: 34 and 34 for the two buttons, three 4px gaps, 51 for the field, which is
   six monospace characters of 7.2 plus 3px of padding and a 1px border a side, and 7 for the unit.
   Six characters is the widest value the field can be given, 0.0625 at one end and 16.00 at the
   other, and the field is fixed at six, so the group is the same width at 1.00 as it is at 16.00
   and no value a keyboard can produce widens the row. Twelve items come to 588, eleven 10px gaps to
   110, the padding to 18, so 716 is the row with a track of no width at all. A track worth pressing
   is about 120px, so 836 is where the whole row stops fitting and 849 is that with the few pixels of
   slack another machine's font metrics are worth.

   The group is what moved the switch, and the arithmetic is 849 - 835 = 14 either way: the old rate
   group cost 104 of content and three gaps, 134, where this one costs 138 of content and two gaps,
   158. Twenty-four more of cost, less the 10 a gap gives back by collapsing three row items into
   one wrapper. Under 849 the buttons on the right go whole rather than half: play/pause has left
   them, so there is no part of them left worth keeping, and what is lost costs nothing on the
   keyboard. The two seek steps and the two frame steps are the arrow keys and comma and period, the
   two rate steps are the bare [ and ], volume is the up and down arrows and M, loop is L, fullscreen
   is F or F11, and each of those runs off the keyboard whatever its button looks like, because a
   click fired by id is dispatched by the DOM and does not care whether the button is on screen.
   What survives is play/pause, both readouts and the whole track, so a mouse-only viewer is still
   able to pause and to scrub. */
@media (max-width: 849px) {
    #bseek-back, #bseek-fwd, #bprev, #bnext, #rate-group, .vol-container, #bloop, #bfs { display: none; }
}
/* The top bar runs out of room on its own arithmetic, and on nothing to do with the row: the caption
   is 60% of the window and the six window buttons with their 8px gaps are 6 x 28 plus 5 x 8, which
   is 208, so 0.6 of a window plus 240 (208 of buttons and 32 of the bar's own padding) fits from
   600 up and below that the buttons would be pushed off the right edge. The switch is that
   arithmetic solved for the width: 240 on the 0.4 the caption does not claim is 600. Hide and fill
   are the bare H and C either way and minimize has nowhere useful to go on a window this size, so
   what is left up there is the question mark, the pen, the cross and the drag handle, and the window
   is still moved by dragging it with the controls up. The handle is absolute with inset: 0, so it
   fills the 48px top bar already and costs the bottom block nothing: out of flow, and in the other
   bar besides. What it does cost is the resize margin, 8px off three edges, and a press there
   belongs to the edge handler and not to a move. The bottom stays at 0 rather than 8 because that
   band is the top bar's own bottom 8px and not the row's: the row holds its own 9px of margin, 9px
   below and 4px above, in the other bar besides, so insetting the handle there would hand back 8px
   of dead band under the pen and the cross for no window that gains anything. Where the band and
   the block meet is a height question, and it is answered by the max-height rule below rather than
   here.

   The update prompt rides this rule rather than carrying one of its own, so everything under 600 is
   the bar it was: the prompt goes with the caption, the 132px rule below is untouched by it, and
   nothing is measured twice. Above 600 it costs what its own comment gives, 97 plus the 10 it stands
   off the caption, which is 107 on top of the 240 above, so 347 rather than 240 is the fixed cost
   up here and 347 on the 0.4 the caption does not claim is 868: that, not 600, is the width at which
   the caption is given its whole 60% again. Between 600 and 868 it is up to 107 narrower than it
   was, which is the trade its own overflow and ellipsis exist for, and dismissing the prompt hands
   every one of those pixels back, because a display: none item is out of the row and the flex line
   is laid out again without it. */
@media (max-width: 600px) {
    .title-capsule, #update-pill, #bhide, #bfill, #bmin { display: none; }
    .drag-handle { inset: 8px 8px 0 8px; }
}
/* Help is the last thing to go on this bar, and it is the one control that says what the keys are:
   a window too narrow for the caption has already lost the file name, and this is the control that
   puts the shortcuts back within reach. It has its own switch rather than riding the one above
   because it is not on the same arithmetic: with the caption gone, what is left up here is this
   button, the pen and the cross, which is 3 x 28 plus 2 x 8, so 100, and 100 plus the bar's own 32
   of padding is 132, which is the narrowest window that can hold all three without pushing the last
   one off the right edge. Below it the pen and the cross stay, because those two are the only way
   out of a window this size, and their own 2 x 28 plus 8 is 64, which with the same 32 of padding
   is 96 and so clears 132 by 36. The bottom row's switches are untouched and nothing here moves
   them: this is the top bar's arithmetic and the top bar's alone. */
@media (max-width: 132px) {
    #bhelp { display: none; }
}
/* Narrower than that the row is four controls, so the readouts are what has to give: 18px of
   padding, play at 30, three 10px gaps and the two readouts at their 62px floors put the track's
   zero at 202px of window, and a track worth pressing is about 120px, so 328 is where the full-size
   digits stop paying for themselves. Under it the digits drop to 10px with no floor and the gaps to
   6px, which is 162px of fixed width instead of 202, so the track grows to 166px at the switch and
   widens from there as the window narrows: it is the track, not the digits, that has to stay
   pressable. Below 162px of window there is no track left to press, which is the one thing this
   layout gives up and only there; play/pause is at the left of the row from the very first pixel of
   width and stays clickable, and every seek is on the keyboard. */
@media (max-width: 328px) {
    .controls-row { gap: 6px; }
    #tc, #tr { font-size: 10px; min-width: 0; }
}
/* Height, and the row is never hidden for it: it is 41px, it is the only bottom row there is, and
   it carries play/pause and the whole track, so a window too short to hold everything must still be
   able to pause and scrub. What a short window cannot hold is the top bar. The row is 41 from the
   window's bottom edge, because it is 4 of air, 28 of controls and 9 of margin, so its top edge is
   41 up. The top bar's box is 48 deep and its drag handle fills it, so the band reaches 48 down,
   and 48 + 41 = 89 is where the two meet: below it the band lies over the top pixels of the row,
   and a press there moves the window instead of pressing a control. 89 is the switch and the caption
   is what goes, because it is the tallest thing up there (33px, ending 41px down) and the only one
   carrying no function: it reaches the block's top edge at 41 + 41 = 82, so from 82 down the band
   and the caption both lie over it. The six window buttons end 38px down and reach the block at
   38 + 41 = 79, so from 79 down even they sit on it, and they are kept: H and C aside they are the
   only way out of a window this size. 89 rather than 82 is the number here because the band reaches
   the block first and it is the band that swallows the press. Longhands only, so this composes with
   the width rules above. */
@media (max-height: 89px) {
    .title-capsule { display: none; }
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
    <!-- The update prompt, and the only thing on this bar that is not always there. The answer
         arrives from the window after the page has loaded, so this is display: none until
         window.showUpdateAvailable() runs, which the window's own code does and nothing else can.
         Click only, and the webview is never navigated: the first button asks the window to hand an
         address to the shell and the window knows which address that is, and the second puts the
         prompt away for the rest of this session. Both are aria-hidden-free: the names are the
         visible text and the title, and the glyphs inside carry nothing. -->
    <div class="update-pill" id="update-pill">
        <button class="update-open" title="A newer release is on GitHub. Opens the releases page in your browser.">
            <svg viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M7 12.4V2.2"/>
                <path d="M3.2 6L7 2.2 10.8 6"/>
            </svg>
            <span>Update</span>
        </button>
        <button class="update-x" title="Dismiss"><span aria-hidden="true">✕</span></button>
    </div>
    <!-- Six window buttons, and the markup order is the visual order: help, hide, fill, draw,
         minimize, close. Help leads because it is the one control that says what the keys are,
         and a lost user needs it before they need anything else up here.

         Hide and fill are inline SVG rather than the characters they were, and both swap two
         variants by display exactly as #bloop does, because Unicode has no glyph that says either
         idea. Hide is an eye while the controls are up and the same eye crossed through while they
         are not, which is the button's own effect read back: you can see them, you cannot. Fill is
         two nested rectangles on one frame, so the pair says contain and fill without either state
         being carried by colour alone: while the picture fits inside the frame the inner rectangle
         stands well clear of it and the gap is the bars, and while the picture fills the frame the
         same inner rectangle runs out to all four edges and the frame crops it. The arrows this
         replaced read as resize at this size rather than as either state. It is not the fullscreen
         shape two rows down: that one is corner brackets around an open square, and there is nothing
         here that can be mistaken for an eye.

         All four are aria-hidden and the accessible name is the button's title, which is how the
         loop and fullscreen buttons already work: an inline svg with no name of its own contributes
         nothing to the button's name, so title is what a screen reader reads. -->
    <div class="winctrl">
        <button class="wbtn" id="bhelp" title="Keyboard shortcuts (/)">?</button>
        <button class="wbtn" id="bhide" title="Toggle on-screen controls">
            <svg viewBox="0 0 14 14" id="hide-shown" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M1 7C2.6 4.4 4.7 3.2 7 3.2S11.4 4.6 13 7c-1.6 2.6-3.7 3.8-6 3.8S2.6 9.6 1 7z"/>
                <circle cx="7" cy="7" r="2"/>
            </svg>
            <svg viewBox="0 0 14 14" id="hide-hidden" style="display:none" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M1 7C2.6 4.4 4.7 3.2 7 3.2S11.4 4.6 13 7c-1.6 2.6-3.7 3.8-6 3.8S2.6 9.6 1 7z"/>
                <circle cx="7" cy="7" r="2"/>
                <path d="M1.4 12.6L12.6 1.4"/>
            </svg>
        </button>
        <button class="wbtn" id="bfill" title="Fill: shape the window to the video (C)">
            <svg viewBox="0 0 14 14" id="fill-off" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="1.1" y="1.1" width="11.8" height="11.8"/>
                <rect x="4" y="4" width="6" height="6"/>
            </svg>
            <svg viewBox="0 0 14 14" id="fill-on" style="display:none" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <rect x="1.1" y="1.1" width="11.8" height="11.8"/>
                <rect x="3" y="3" width="8" height="8"/>
            </svg>
        </button>
        <button class="wbtn" id="bdraw" title="Draw on video">✎</button>
        <button class="wbtn" id="bmin" title="Minimize">—</button>
        <button class="wbtn close" id="bcls" title="Close">✕</button>
    </div>
</div>

<!-- Bottom chrome: one row, in this order. Play/pause first and immediately left of the timeline,
     the two readouts bracketing it, then the two seek steps, the two frame steps, the rate group,
     volume, loop, and fullscreen last. Plain flex with no order, so this markup is the visual order
     and the focus order. The rate group sits at the left end of the group to the right of the track
     and before the volume, because everything to its right keeps the x it had and the timeline is
     the only thing that pays for it. The group is three items, a minus, the editable number and a
     plus, with the x outside the field, and the field is the one control on this bar that is always
     on and always holds the rate. Its aria-valuemin and aria-valuemax are the engine's own bounds,
     quoted in the script below; Home and End are left alone there on purpose. -->
<div id="hud">
    <div class="controls-row">
        <button class="btn" id="bplay" title="Play/Pause">▶</button>
        <span class="seek-time" id="tc">00:00:00</span>

        <div class="seekbar-container" id="seekbar-container">
            <div class="seek-track">
                <div class="seek-fill" id="seek-fill"></div>
                <div class="seek-thumb" id="seek-thumb"></div>
            </div>
        </div>

        <span class="seek-time" id="tr">-00:00:00</span>

        <button class="btn" id="bseek-back">-5s</button>
        <button class="btn" id="bseek-fwd">+5s</button>
        <button class="btn" id="bprev">-1F</button>
        <button class="btn" id="bnext">+1F</button>

        <div class="rate-group" id="rate-group">
            <button class="btn" id="bslower" tabindex="-1" title="Slower ([)"><span aria-hidden="true">-</span></button>
            <input class="rate-input" id="rate-input" type="text" role="spinbutton" aria-label="Playback speed"
                   aria-valuemin="0.0625" aria-valuemax="16" aria-valuenow="1" aria-valuetext="1.00 times"
                   value="1.00" size="6" inputmode="decimal" spellcheck="false" autocomplete="off">
            <span class="rate-x" aria-hidden="true">x</span>
            <span class="rate-note" id="rate-note" role="status" aria-live="polite"></span>
            <button class="btn" id="bfaster" tabindex="-1" title="Faster (])"><span aria-hidden="true">+</span></button>
        </div>

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

<!-- Keyboard shortcuts. A top-level sibling of #hud, for the reason the CSS above gives: inside
     #hud the focus guard and the three-second hide would take this panel's focus and then the
     panel itself. Header and footer sit outside the scroll region and the list is the only part
     that scrolls, so the title and the way out stay put at any height. Every key below is a
     binding that exists in this file and works; nothing dead, unreachable or contradicted is
     listed, and the four keys that mean one thing while playing and another while drawing say so
     in their own row rather than being printed once without a scope. A row is one action and not
     one key: two keys that reach the same action share that row and both are printed on it,
     because the owner read "Fullscreen on or off" twice over as two separate things to look up. -->
<div id="keys-overlay" class="keys-overlay">
  <div class="keys-panel" role="dialog" aria-modal="true" aria-labelledby="keys-title">
    <div class="keys-head">
      <h2 class="keys-title" id="keys-title">Keyboard shortcuts</h2>
    </div>
    <p class="keys-intro" id="keys-focus" tabindex="-1">Tab moves through this panel, Escape closes it.</p>
    <div class="keys-scroll" id="keys-list" tabindex="0">
      <ul class="keys-groups">
        <li class="keys-group">
          <h3 class="keys-group-title">Playback</h3>
          <ul class="keys-rows">
            <li class="keys-row">
              <span class="keys-desc">Play or pause</span>
              <span class="keys-keys"><kbd class="keys-key">Space</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Loop the video on or off</span>
              <span class="keys-keys"><kbd class="keys-key">L</kbd></span>
            </li>
          </ul>
        </li>
        <li class="keys-group">
          <h3 class="keys-group-title">Seeking</h3>
          <ul class="keys-rows">
            <li class="keys-row">
              <span class="keys-desc">Back five seconds</span>
              <span class="keys-keys"><kbd class="keys-key">&#8592;</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Forward five seconds</span>
              <span class="keys-keys"><kbd class="keys-key">&#8594;</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Step one frame back</span>
              <span class="keys-keys"><kbd class="keys-key">,</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Step one frame forward</span>
              <span class="keys-keys"><kbd class="keys-key">.</kbd></span>
            </li>
          </ul>
        </li>
        <li class="keys-group">
          <h3 class="keys-group-title">Speed</h3>
          <ul class="keys-rows">
            <li class="keys-row">
              <span class="keys-desc">Slower</span>
              <span class="keys-keys"><kbd class="keys-key">[</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Faster</span>
              <span class="keys-keys"><kbd class="keys-key">]</kbd></span>
            </li>
          </ul>
        </li>
        <li class="keys-group">
          <h3 class="keys-group-title">Window and display</h3>
          <ul class="keys-rows">
            <li class="keys-row">
              <span class="keys-desc">Fullscreen on or off</span>
              <span class="keys-keys"><kbd class="keys-key">F</kbd><kbd class="keys-key">F11</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Reshape the window to the video's shape</span>
              <span class="keys-keys"><kbd class="keys-key">C</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Hide or show the on-screen controls</span>
              <span class="keys-keys"><kbd class="keys-key">H</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Leave fullscreen</span>
              <span class="keys-keys"><kbd class="keys-key">Escape</kbd></span>
            </li>
          </ul>
        </li>
        <li class="keys-group">
          <h3 class="keys-group-title">Audio</h3>
          <ul class="keys-rows">
            <li class="keys-row">
              <span class="keys-desc">Volume up</span>
              <span class="keys-keys"><kbd class="keys-key">&#8593;</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Volume down</span>
              <span class="keys-keys"><kbd class="keys-key">&#8595;</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Mute or unmute</span>
              <span class="keys-keys"><kbd class="keys-key">M</kbd></span>
            </li>
          </ul>
        </li>
        <li class="keys-group">
          <h3 class="keys-group-title">Annotation, while drawing is open</h3>
          <ul class="keys-rows">
            <li class="keys-row">
              <span class="keys-desc">Pen tool</span>
              <span class="keys-keys"><kbd class="keys-key">P</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Line tool; loop while playing</span>
              <span class="keys-keys"><kbd class="keys-key">L</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Arrow tool</span>
              <span class="keys-keys"><kbd class="keys-key">A</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Rectangle tool</span>
              <span class="keys-keys"><kbd class="keys-key">R</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Circle tool; reshapes the window while playing</span>
              <span class="keys-keys"><kbd class="keys-key">C</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Hand tool; hides the controls while playing</span>
              <span class="keys-keys"><kbd class="keys-key">H</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Stroke size, two to twenty</span>
              <span class="keys-keys"><kbd class="keys-key">0</kbd>&ndash;<kbd class="keys-key">9</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Hand tool; hold the shape with the mouse</span>
              <span class="keys-keys"><kbd class="keys-key">Delete</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Clear every shape, still undoable</span>
              <span class="keys-keys"><kbd class="keys-key">Backspace</kbd></span>
            </li>
            <li class="keys-row">
              <span class="keys-desc">Leave drawing; leaves fullscreen while playing</span>
              <span class="keys-keys"><kbd class="keys-key">Escape</kbd></span>
            </li>
          </ul>
        </li>
      </ul>
    </div>
    <div class="keys-foot">
      <span class="keys-hint">/ or a click outside closes this too.</span>
      <button class="keys-close" id="keys-close" type="button">Close</button>
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
    <button class="dbtn" id="bclear" title="Clear All (Backspace)">✕</button>
    <div class="dsep"></div>
    <button class="dbtn" id="bclose-draw" title="Exit Draw Mode">✕ Exit</button>
  </div>
  </div>
</div>

<!-- Custom text input dialog (replaces window.prompt which crashes on macOS/WKWebView) -->
<div id="text-dialog" style="display:none;position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.6);align-items:center;justify-content:center;">
  <div style="background:rgba(30,30,30,0.95);border:1px solid rgba(255,255,255,0.15);border-radius:14px;padding:24px 28px;min-width:300px;box-shadow:0 16px 48px rgba(0,0,0,0.6);">
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
// The one thing that says "the shortcuts panel is open", and nothing else does. It is set and
// cleared only by openHelp and closeHelp below, and read by three places: the toggle and the
// panel's own key handling, the global player handler, and the edge cursor. It lives up here
// rather than with the panel because the cursor helper below is called before that section is
// reached.
let helpOpen  = false;
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
    // The shortcuts panel is modal and its backdrop covers all four edges, so while it is up no
    // edge of the window is a resize target. An edge cursor there would be promising a press
    // that cannot happen, because the panel stops the press before it reaches the edge handler.
    if (helpOpen) { document.body.style.cursor = 'default'; return; }
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
// Which of the two eye variants is on screen is written from the resulting state, here and nowhere
// else. Shown while the chrome is up, crossed out while it is not.
function renderHideIcon() {
    document.getElementById('hide-shown').style.display  = topbar.classList.contains('show') ? '' : 'none';
    document.getElementById('hide-hidden').style.display = topbar.classList.contains('show') ? 'none' : '';
}
function showControls() {
    topbar.classList.add('show');
    hud.classList.add('show');
    renderHideIcon();
    applyBodyCursor();
}
function hideControls() {
    clearTimeout(hideT);
    topbar.classList.remove('show');
    hud.classList.remove('show');
    renderHideIcon();
    applyBodyCursor();
}
function showUI() {
    if (hudLock || hudPin || document.body.classList.contains('drawmode')) return;
    showControls();
    clearTimeout(hideT);
    // Re-checked when the timer fires, not only when it was set, because the rate field can take
    // focus somewhere inside those three seconds. The chrome does not go to opacity 0 with the
    // keyboard sitting in it: an invisible control cannot be read and cannot be escaped from by eye,
    // and it takes the focus ring with it. Inside the chrome rather than in the field alone, so a
    // control that took focus from a click is in the same position.
    hideT = setTimeout(() => {
        if (hud.contains(document.activeElement)) { showUI(); return; }
        hideControls();
    }, 3000);
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

// Which of the two arrow variants is on screen is written from fillMode, here and nowhere else.
// Inward while the picture fits inside the frame, outward while it fills it.
function renderFillIcon() {
    document.getElementById('fill-off').style.display = fillMode ? 'none' : '';
    document.getElementById('fill-on').style.display  = fillMode ? '' : 'none';
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
    renderFillIcon();
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
    renderFillIcon();
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
// pointer-events: none on a hidden #hud keeps a press from reaching the chrome, but it does not keep
// Tab out of it: without this a Tab with the chrome down would put the keyboard into a control at
// opacity 0, and the rate field is the first one in this row that can be typed into at all. Focus
// landing in a hidden chrome is let go again, and no press can produce that case in the first place,
// because pointer-events has already turned the press away.
document.addEventListener('focusin', (e) => {
    if (hud.contains(e.target) && !hud.classList.contains('show')) e.target.blur();
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

/* No handler of its own on .controls-row, and that is the point rather than an omission. The row
   spans the full width, so a mousedown here would have to swallow every press on it, on a control
   or on the bare backing, and swallowing the backing is what cost the bottom edge: for as long as
   the chrome is up no press on the bottom 8px could start a resize, the two bottom corners
   included. The 9px of padding under the controls holds the same line without touching a single
   press, so the backing below and beside the controls reaches the edge handler and starts a resize
   where it is meant to, and nothing above the padding does. #seekbar-container and #vol-slider keep
   the stopPropagation they have always had, which is why scrubbing and volume work with the bar off
   the bottom of the window, and stopPropagation rather than preventDefault there as here: cancelling
   the default on a mousedown is what would suppress focus, and the press still belongs to the
   control under it. */

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
// The one way in from the mouse, and it is the same function the bare / calls below, so the panel
// opens and closes and takes focus identically whichever route opened it and there is no second
// copy of any of it. A declaration, not a call site reached by name, so this line runs before the
// text of openHelp further down is evaluated and still gets the hoisted binding. No toggle: the
// panel is modal and its dim covers this button, so a second press here cannot arrive while it is
// up. The overlay's own / stays the only key that opens it.
document.getElementById('bhelp').onclick = openHelp;

/* ====================== OPTIONAL UPDATE PROMPT ====================== */
/* The window asks GitHub once per launch whether the newest published release is newer than this
   build, and calls showUpdateAvailable when it is. Nothing here asks for that: the answer is never
   fetched from the page, which this page's own CSP forbids anyway, since connect-src allows
   127.0.0.1 and nothing else. No timer, no poll, no retry and no decision to wait for sits behind
   any of it, and the video has been playing since before the request went out. */
const updatePill = document.getElementById('update-pill');
// Dismissal lasts for this session and no longer: nothing on disk, no storage, no cookie, so the
// next launch offers the prompt again if that release is still newer. That is the difference between
// a prompt and a setting, and it is also why the prompt comes back with the chrome rather than
// staying on top of the picture.
let updateDismissed = false;

// The one writer of this chip's state. Reached from the window's hook below, which is installed as
// an initialization script and therefore exists before this file does, so an answer cannot be lost
// by arriving early.
function paintUpdatePill() {
    if (updateDismissed) return;
    updatePill.classList.add('show');
}
window.paintUpdatePill = paintUpdatePill;
// An answer that landed before this file ran is waiting on the flag that hook sets, and this is
// the only other way the chip ever appears. The flag is cleared either way so a later answer is
// painted by the hook rather than here.
if (window.__drUpdatePending) {
    window.__drUpdatePending = false;
    paintUpdatePill();
}

// This page is never navigated and is never handed a URL to load: it asks the window to open one
// address with the shell, and the window holds the address. Clicking costs the picture nothing.
updatePill.querySelector('.update-open').addEventListener('click', () => {
    window.ipc.postMessage('open_release_page');
});
updatePill.querySelector('.update-x').addEventListener('click', () => {
    updateDismissed = true;
    updatePill.classList.remove('show');
});

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

/* ====================== PLAYBACK RATE ====================== */
/* The rate goes through the media element's own playbackRate and through nothing else: no second
   clock, no timer, no transform, nothing that touches how the frame is painted. Nothing about the
   window's shape follows from it either: fill mode reads videoWidth and videoHeight, which are the
   frame's own size and which no rate has any say in, and a rate change posts no resize.

   The two bounds below are the engine's and not this app's. Chromium declares kMinPlaybackRate
   0.0625 and kMaxPlaybackRate 16.0 in html_media_element.h, and IsValidPlaybackRate admits
   anything between them plus exactly zero. The Windows build is a WebView2, which is Chromium, so
   this is the range the element really has. The HTML spec itself sets no bound at all on
   playbackRate, which is the whole reason these are a copy of the engine rather than a decision
   about what an owner should be allowed to ask for, and the reason nothing here narrows them: 25 is
   over Blink's ceiling and 0.001 is under its floor, so both throw there and both quietly mislead
   on WKWebView, and the widest honest range is the one below. For what other players do, none of
   which agrees with any other: YouTube 0.25 to 2.00, Plex 0.5 to 2.00, JW Player 0.5 to 2.00, VLC's
   Android UI 0.25 to 4.00, VLC's own engine 0.03125 to 31.25 behind a Faster and Slower pair, mpv
   0.01 to 100, and Films & TV ships no rate control at all.

   The step stays a quarter and is coarse on purpose. Chromium issue 40190553 recorded audio and
   video drifting apart when playbackRate is written repeatedly at roughly 200ms, which is what
   holding a speed key down produces, so the grid is not finer than this. The grid's own ends are not
   the bounds above: its lowest point is 0.25 and its highest is 16.00, so one step off either end asks
   for a rate the engine refuses, and stepRate clamps those two to RATE_MIN and RATE_MAX rather than
   refusing them. That is what puts the real floor a single press away instead of out of reach, and it
   is what leaves both bounds as the place a press stops. Between two grid points the field is still
   the only way in, and that is what the field is for. A typed value outside the two bounds is
   clamped to the edge it overshot rather than refused, and zero is what makes that the right rule:
   the spec means 0 by leaving the position still with paused false and no pause event, so a 0.00x
   next to a play triangle would be a control that looks like it took effect and did not. Clamping a
   typed 0 up to the floor removes that trap with no case of its own, because the rule stays one rule:
   anything finite clamps, and only text that is not a number is refused. */
const RATE_MIN = 0.0625;
const RATE_MAX = 16;
// The grid a button, a bracket key or an arrow key walks. Neither of its ends is a bound above, which
// is the whole reason the step below has to clamp.
const RATE_STEP = 0.25;
// Page keys move four steps, which is 1.00: a whole rate rather than a nudge, and still coarse.
const RATE_PAGE = 4;
const bslower   = document.getElementById('bslower');
const bfaster   = document.getElementById('bfaster');
const rateInput = document.getElementById('rate-input');
const rateNote  = document.getElementById('rate-note');
// The rate in effect, and the only thing the two buttons and the two bracket keys step from. It is
// not persisted: a newly opened file starts at 1.00, like any other player, and loadedmetadata
// below is what resets it.
let rate = 1;

// Two decimals for everything on the grid and above it, four only for the values between the grid
// points. The floor is the one value where two decimals is wrong rather than short: 0.0625 through
// toFixed(2) is 0.06, which is neither the rate that was asked for nor even in range.
function fmtRate(v) {
    const four = v.toFixed(4);
    return four.endsWith('00') ? v.toFixed(2) : four.replace(/0$/, '');
}

// The field is the readout, so this is what keeps it true: the number on screen is the rate, and
// aria-valuetext carries the unit, because a bare number is announced with none and "1.00" alone
// tells a screen reader nothing about what it is a rate of.
function renderRate() {
    const shown = fmtRate(rate);
    rateInput.value = shown;
    rateInput.setAttribute('aria-valuenow', String(rate));
    rateInput.setAttribute('aria-valuetext', shown + ' times');
    rateNote.textContent = '';
}

// The only writer of playbackRate in this file, and the range is checked before the write so a value
// outside it never reaches the element at all. Blink throws NotSupportedError and leaves the stored
// value alone, so there is no read-back here and no restore branch: on a platform this app ships to
// a value read straight back cannot disagree with the value written, and a comparison that can never
// fire is not a safety net. WKWebView, which is the macOS build, does the opposite and is the reason
// the check is load-bearing rather than tidy: it enforces nothing, never throws, and reports back
// whatever it was asked for, so an unchecked write there is a rate that looks live and is not. The
// try/catch is left as the belt and braces for an engine that does neither.
function applyRate(next) {
    if (!Number.isFinite(next) || next < RATE_MIN || next > RATE_MAX) return false;
    try {
        vid.playbackRate = next;
    } catch (err) {
        console.error('RATE_THREW', next, err);
        // The write did not happen, so the field goes back to the rate really in effect before the
        // reason goes up, and the reason is written after that restore so it outlives it. This is
        // the only refusal that can leave the field holding the value it was given: the typed path
        // refuses in commitTypedRate, which puts the field back itself.
        renderRate();
        rateNote.textContent = 'This engine refused the rate';
        return false;
    }
    rate = next;
    renderRate();
    return true;
}

// Typed entry takes any value in range rather than snapping it to the quarter, because a value
// between two grid points, 3.33 or 2.5, is one no press can reach and typing is where it comes from.
// The floor is a press away now rather than a typed value only, and it is still taken as typed here
// rather than as the nearest grid point, because 0.0625 is a legal rate in its own right.
//
// Anything finite outside the range is clamped to the edge it overshot and applied, never refused:
// 17 becomes 16.00, and 0.01, 0 and -2 all become 0.0625. That is the same clamp stepRate below
// already applies to the two steps off its own grid, so there is one rule for a value out of range
// rather than a clamp path beside a refusal path, and it puts a typed 0 on the floor instead of on
// the one rate that stands the picture still without pausing it. applyRate renders from the clamped
// rate, so the field comes back holding the number that is really playing and the two cannot disagree
// at any moment, and the note naming the edge is written after that render because renderRate clears
// it, which is also what clears it on the next change that is not clamped.
//
// The two refusals left are the two things that are not a number at all. An empty field has to be
// caught before the parse and not after it, because Number('') is 0 and Number(' ') is 0. Those two
// write nothing at all and put the field back to the rate in effect here in this same call rather
// than on blur, with the reason written after the restore.
function commitTypedRate() {
    const raw = rateInput.value.trim();
    const v = raw === '' ? NaN : Number(raw);
    if (raw === '' || !Number.isFinite(v)) {
        renderRate();
        rateNote.textContent = raw === '' ? 'Enter a number' : 'Not a number';
        return false;
    }
    const clamped = Math.min(RATE_MAX, Math.max(RATE_MIN, v));
    // applyRate refuses by putting the field back and saying so, so its own note is left standing
    // here and a clamp is only announced once the write has actually landed.
    if (!applyRate(clamped)) return false;
    if (clamped !== v) {
        rateNote.textContent = clamped === RATE_MAX
            ? 'Clamped to the fastest rate, ' + fmtRate(RATE_MAX)
            : 'Clamped to the slowest rate, ' + fmtRate(RATE_MIN);
    }
    return true;
}

// One step off the grid rather than one added to the last value. The index is the integer the rate is
// really stored as, so every rate written here is a whole number of steps from a known start and
// nothing accumulates behind however many keys are held down, and a base typed off the grid snaps to
// its nearest point rather than dragging the rest of the grid with it. The candidate is then clamped
// to the engine's own bounds and not refused: the grid's lowest point is 0.25, so a step below it
// asks for 0.00, which is the one rate applyRate and Blink both refuse, and the grid's highest is
// 16.00, so a step above it asks for 16.25, which Blink refuses too. Clamping is what makes the floor
// reachable at all and what leaves both bounds as the place a press stops. From 1.00 that is 0.75,
// 0.50, 0.25, 0.0625 and no lower, and 1.25 upward on the quarter to 16.00 and no higher; pressed
// again at either end this rewrites the bound it is already on, so nothing wraps, jumps or reaches 0
// and the field and the note stay true. The base comes in from the field so an arrow key steps from
// what is in the field; from anything but an in-range number that base is the rate in effect.
function stepRate(dir, from) {
    const base = Number.isFinite(from) ? from : rate;
    const candidate = (Math.round(base / RATE_STEP) + dir) * RATE_STEP;
    return applyRate(Math.min(RATE_MAX, Math.max(RATE_MIN, candidate)));
}

function typedRate() {
    const raw = rateInput.value.trim();
    const v = raw === '' ? NaN : Number(raw);
    return Number.isFinite(v) && v >= RATE_MIN && v <= RATE_MAX ? v : rate;
}

// A new source starts at 1.00 and the field says so. The rate belongs to the file rather than to
// the session, so opening another video never inherits the last one's speed, which is what the
// owner asked for and what every other player does. loadedmetadata is the one event a load raises
// here and a seek raises none of them, so a scrub cannot reset the rate, and it fires on the first
// load as well as on every later one. Written even though the element may already have taken its
// playbackRate back to 1.00 on a new src, because WKWebView may not have and the field must not
// depend on which of the two it got.
vid.addEventListener('loadedmetadata', () => {
    rate = 1;
    applyRate(1);
});

// The spinbutton contract, WAI-ARIA APG, which is what role="spinbutton" above promises. Every key
// here stops propagating first and that is the load-bearing half of it: the player handler at the
// foot of this file switches on e.code with no focus guard, so without this a keystroke aimed at the
// field would fire a shortcut instead of reaching it. Left to it are c for fill, h for the chrome,
// l for loop, m for mute, f and F11 for fullscreen, space for play, the brackets for the rate, comma
// and period for the frame steps, the arrow keys for seeking and volume, and / for the shortcuts
// panel, which is why a slash typed into the box inserts a slash and opens nothing. The typing of
// 0.0625 is the sharpest case, because the period in it is a frame step that pauses the picture.
// The arrows are default-prevented here, which the player handler does not do for them, so the
// caret stays put as well as the volume. The same stop the text dialog's input makes for the same
// reason.
rateInput.addEventListener('keydown', (e) => {
    e.stopPropagation();
    switch (e.code) {
        case 'ArrowUp':
        case 'ArrowRight':
            e.preventDefault();
            stepRate(1, typedRate());
            break;
        case 'ArrowDown':
        case 'ArrowLeft':
            e.preventDefault();
            stepRate(-1, typedRate());
            break;
        case 'PageUp':
            e.preventDefault();
            stepRate(RATE_PAGE, typedRate());
            break;
        case 'PageDown':
            e.preventDefault();
            stepRate(-RATE_PAGE, typedRate());
            break;
        // Home and End are deliberately absent. In a text field they move the caret and a screen
        // reader expects them to, and the APG guidance that mapped them to the minimum and the
        // maximum is being withdrawn for that reason, so they are left to behave natively.
        case 'Enter':
            e.preventDefault();
            commitTypedRate();
            rateInput.blur();
            break;
        case 'Escape':
            e.preventDefault();
            renderRate();
            rateInput.blur();
            break;
    }
});

// Focus holds the chrome up and restarts its clock, the first half of which is the guard in showUI
// and the second is here: the timer was set three seconds ago and has to be set again from now, or
// it fires mid-edit. select on the way in because the whole value is being replaced, so typing a
// rate should replace the rate rather than land inside it.
rateInput.addEventListener('focus', () => {
    showUI();
    rateInput.select();
});

// Blur commits what can be committed, and it is the only path that does: an edit the user walked away
// from rather than pressing Enter reaches the field here, and commitTypedRate clamps it or refuses
// it and puts the field back itself, so the field is still never left holding a number that is not
// playing, and the clamp here is the same one Enter gets rather than a second path of its own. The
// note goes with the reason, because focus has left and a message about a keystroke nobody is typing
// any more is noise. Skipped entirely when the field already holds the rate in effect, which is every blur
// that is not an edit: Enter and Escape leave it holding the rate whether they committed or refused,
// and re-writing the rate the element already has would be a second write of the same value for
// nothing.
rateInput.addEventListener('blur', () => {
    if (rateInput.value !== fmtRate(rate)) commitTypedRate();
    rateNote.textContent = '';
    showUI();
});

bslower.onclick = () => { stepRate(-1); };
bfaster.onclick = () => { stepRate(1); };
renderRate();

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
/* Bare /, and bare because it was measured rather than argued: the owner pressed it on this
   build and Shift + / never arrived while the same key with no modifier opened the panel every
   time. WebView2 holds back a modified key the way it holds back Ctrl+H, Ctrl+Backspace and the
   rest of its own chords, and every binding in this file is bare for that one reason, the one
   Shift left being the Tab trap's own Shift+Tab below, which is focus order rather than a
   binding. So the label is the key rather than the character it makes on a layout, and it is the
   slash on the US layout that is printed too. Do not put the Shift back on this to make the label
   read nicer: the label would then promise a key that does not work.

   Still bound to the physical key rather than to a character, so e.code rather than e.key: e.key
   is '?' on a US layout and whatever that layout prints on every other one, and the bare / is the
   one that arrives on all of them. Slash is claimed nowhere else in this file, so this has
   exactly one owner. It lives here rather than in the global handler because the panel has to
   open in draw mode too, and that handler bails on the draw bar: a user who has lost track of
   which mode they are in is exactly who needs to read this.

   Registered before the global handler on purpose, for the same reason. Escape, the toggle and
   Tab all have to be taken here first: the global handler sends Escape to leave fullscreen, and
   the draw mode handler sits on this same node, where stopping propagation would not reach it
   and only stopping immediate propagation does. */
const keysOverlay = document.getElementById('keys-overlay');
const keysFocus   = document.getElementById('keys-focus');
const keysList    = document.getElementById('keys-list');
const keysClose   = document.getElementById('keys-close');
// Everything Tab can reach in here, in markup order, and the whole of it: the scrollable list,
// which is focusable so the keyboard can scroll it, and the Close button. The focus target is
// tabindex -1 and is therefore not a third stop.
const keysStops = [keysList, keysClose];

function openHelp() {
    helpOpen = true;
    keysOverlay.classList.add('open');
    applyBodyCursor();  // the edge cursors would be claiming a resize the panel cannot start
    // On the static line at the top of the content, not on the Close button: a dialog that takes
    // focus on its own dismiss control announces "Close, button" and the list behind it is never
    // read at all.
    keysFocus.focus();
}

function closeHelp() {
    // Focus goes back to the body, deliberately. It is not handed to a control in the chrome,
    // because the focusin guard blurs anything focused inside a hidden #hud, so a control handed
    // focus there is bounced straight back out again and the owner is left nowhere. The blur is
    // taken here, while the panel is still up, rather than left to display:none, so the target
    // is a decision rather than a side effect.
    if (keysOverlay.contains(document.activeElement)) document.activeElement.blur();
    helpOpen = false;
    keysOverlay.classList.remove('open');
    applyBodyCursor();
}

keysClose.addEventListener('click', closeHelp);
// A press anywhere in the overlay stops here, so it never reaches the document mousedown
// handlers behind it: no resize from an edge, no window drag from the stage, nothing drawn on the
// canvas. Only a click on the dim itself dismisses, because a press on the panel is a press on a
// dialog and not an outside click.
keysOverlay.addEventListener('mousedown', e => e.stopPropagation());
keysOverlay.addEventListener('click', e => {
    if (e.target === keysOverlay) closeHelp();
});

document.addEventListener('keydown', e => {
    const isToggle = e.code === 'Slash';
    if (helpOpen) {
        // Escape closes the panel and is consumed here. Left to the global handler it would leave
        // fullscreen instead, with the panel still up, and that is WCAG 2.1.2 No Keyboard Trap:
        // focus is held while the way out of it does not work.
        if (e.code === 'Escape') {
            e.preventDefault();
            e.stopImmediatePropagation();
            closeHelp();
            return;
        }
        // The toggle again closes, and the repeat guard is load-bearing in both directions: a
        // held key repeats keydown, which would otherwise strobe the panel open and shut. It is
        // kept on a bare key for that reason rather than dropped with the modifier: autorepeat
        // repeats whatever is held, chord or no chord.
        if (isToggle && !e.repeat) {
            e.preventDefault();
            e.stopImmediatePropagation();
            closeHelp();
            return;
        }
        // Tab and Shift+Tab cycle the two stops and never leave the panel. Without this a Tab from
        // here walks straight out into the chrome behind the overlay, which is the difference
        // between a modal dialog and a covered one.
        if (e.code === 'Tab') {
            e.preventDefault();
            e.stopImmediatePropagation();
            const i = keysStops.indexOf(document.activeElement);
            const n = keysStops.length;
            const next = i < 0 ? (e.shiftKey ? n - 1 : 0) : (i + (e.shiftKey ? -1 : 1) + n) % n;
            keysStops[next].focus();
            return;
        }
        // Everything else stops here as well, which is what makes aria-modal="true" true rather
        // than decorative: Space must not pause the video and the arrows must not seek it while a
        // panel is sitting over it.
        e.stopImmediatePropagation();
        return;
    }
    if (isToggle && !e.repeat) {
        e.preventDefault();
        openHelp();
    }
});

document.addEventListener('keydown', e => {
    // The shortcuts panel owns the keyboard while it is up, so nothing below may act. The
    // listener above already stops the key before it arrives; this is the same boolean written
    // next to the handler it protects, and one boolean is the only thing that decides.
    if (helpOpen) return;
    // Bail out early in draw mode — draw mode handles its own keys
    if (drawbar.classList.contains('open')) return;
    // H toggles the chrome, fullscreen or not. Bare, and left bare: WebView2 holds back a fixed
    // set of its own browser chords and never delivers those to the page, which is what made the
    // Ctrl+H this file once used dead. Other chords do arrive, plain Ctrl+letter among them, so
    // this is a choice rather than a limit, and no modifier is tested anywhere below.
    if (e.code === 'KeyH') {
        e.preventDefault();
        toggleControls();
        return;
    }
    // C turns on fill mode, which reshapes the window to the video rather than changing how
    // the picture is drawn. Bare for the same reason as H, and handed to the circle tool by the
    // early return above while the draw bar is open, as with L and H.
    if (e.code === 'KeyC') {
        e.preventDefault();
        toggleFill();
        return;
    }
    // [ and ] are the rate, one step down and one step up. Bare, and by physical key: e.code is
    // layout-independent where e.key is not, and a shifted character arrives as a different e.key
    // altogether, so a test on e.key would put these two keys somewhere else on a non-US layout.
    // Nothing else in this file claims BracketLeft or BracketRight, so the two keys have exactly
    // one owner each, and the early return above has already handed them to nothing while the
    // draw bar is open.
    if (e.code === 'BracketLeft') {
        e.preventDefault();
        stepRate(-1);
        return;
    }
    if (e.code === 'BracketRight') {
        e.preventDefault();
        stepRate(1);
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
    else if (k === 'delete') {
        // Delete removes the one shape that is picked up, and nothing else: no other key in this
        // file claims Delete, and the picked-up shape only exists while the mouse button is held
        // on it, so there is nothing to delete when it is null and nothing is prevented.
        if (selShape) {
            saveDrawState();
            const idx = shapes.indexOf(selShape);
            if (idx >= 0) { shapes.splice(idx, 1); renderAll(); }
            selShape = null;
            e.preventDefault();
        }
    }
    else if (k === 'backspace') {
        // Backspace clears every shape, and the default is cancelled whether or not there is
        // anything to clear, because the key now means something in this mode.
        //
        // Bare, and the Ctrl+Backspace it replaced is gone: WebView2 reserves that chord and
        // never delivers it to the page, so the old branch was correct and the key never
        // arrived. The metaKey half went with it, because Windows has no Meta key to press and
        // leaving it would keep half a condition that cannot fire on the platform this build
        // ships to.
        e.preventDefault();
        // clearDrawCanvas saves the undo snapshot first, so clearing stays undoable. Nothing to
        // clear means no snapshot, or the undo stack fills with empty frames.
        if (shapes.length > 0) clearDrawCanvas();
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
use std::time::Duration;
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

// Where the answer comes from, and the one address the shell is ever asked to open. Both are
// constants so the page is never given a URL at all: it asks the window to open the release page and
// the window already knows which page that is.
const RELEASES_API: &str = "https://api.github.com/repos/Babar-Meet/Dr.Player/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/Babar-Meet/Dr.Player/releases/latest";

// A metadata GET to one host, and five seconds is generous for that while still being a ceiling:
// reqwest's timeout covers connect, handshake and body read as one deadline, so a network that has
// gone quiet cannot leave a request alive behind the video. Nothing here is retried.
const UPDATE_TIMEOUT: Duration = Duration::from_secs(5);

// Dr.Player also ships inside PDEA's Electron app, as resources/dr-player.exe, and PDEA updates as
// a unit with an updater of its own. A bundled copy compiles in this same version number, so
// without this it would tell people a newer Dr.Player exists that they cannot install without
// updating PDEA first: a promise nothing in that copy can keep. So a bundled copy skips the check
// entirely rather than answering it wrongly.
//
// The test is a sibling app.asar, because that is what a packaged Electron app looks like from the
// inside and this executable can see its own neighbours. PDEA has to change nothing for it to be
// true, which is the reason for this and not another: PDEA is a separate codebase on its own
// release cycle, and an argument the two of them could disagree about is one that eventually would.
// Reading the directory rather than anything passed in also means there is no channel between them
// at all, and a standalone copy sitting in a folder that happens to hold an app.asar is not a shape
// Electron produces.
//
// DRPLAYER_EMBEDDED forces the answer either way without a rebuild, which is what makes the bundled
// behaviour testable on a machine where nothing is bundled: any value other than 0, no, false or off
// means embedded and skips the check, those four mean standalone and run it whatever the directory
// says, and unset or empty leaves the answer to the probe above.
fn is_embedded_copy() -> bool {
    match std::env::var("DRPLAYER_EMBEDDED") {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "" | "0" | "no" | "false" | "off"
        ),
        Err(_) => beside_an_electron_bundle(),
    }
}

// Every failure answers no rather than guessing: the cost of being wrong about a standalone copy is
// one prompt about a release that does exist, and the cost of the other way round is a bundled copy
// that has gone quiet about every release from here on.
fn beside_an_electron_bundle() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Some(dir) = exe.parent() else {
        return false;
    };
    dir.join("app.asar").exists()
}

// MAJOR.MINOR.PATCH, and nothing else is accepted. A leading v is dropped first because that is how
// the tags in this repository are written. Anything that will not parse into three numbers answers
// None rather than a guess, so a prerelease, a bare "0.4", a tag with a build suffix and a tag that
// is a whole sentence all decline in the same way, and a prompt built on a guess is worse than none.
fn parse_version(tag: &str) -> Option<(u64, u64, u64)> {
    let trimmed = tag.trim();
    let body = trimmed
        .strip_prefix('v')
        .or_else(|| trimmed.strip_prefix('V'))
        .unwrap_or(trimmed);
    let mut parts = body.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

// Newer, and only newer: equal is not newer, so a build of the newest release is never told about
// itself. A version on either side that does not parse means no update, which is the one answer that
// is always safe to give.
fn is_newer_release(latest_tag: &str, current: &str) -> bool {
    match (parse_version(latest_tag), parse_version(current)) {
        (Some(latest), Some(mine)) => latest > mine,
        _ => false,
    }
}

// The one request, and every way it can go wrong is the same answer: no prompt. No network, a DNS
// failure, a TLS failure, a rate limit (GitHub allows about 60 unauthenticated requests an hour per
// IP address, so a limit reached here is usually somebody else's doing), a 404 because no release
// has been published, a body that is not JSON, a missing or non-string tag: all of them return
// false, none of them is shown to anyone, none is logged to a dialog and none of them throws. The
// caller does not wait for this and nothing in the window depends on it finishing.
async fn newer_release_published() -> bool {
    let client = match reqwest::Client::builder()
        .timeout(UPDATE_TIMEOUT)
        // GitHub answers a request with no User-Agent with 403, which without this would be a
        // silent no-prompt on every launch, forever.
        .user_agent(concat!("Dr.Player/", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };
    let response = match client.get(RELEASES_API).send().await {
        Ok(response) => response,
        Err(_) => return false,
    };
    if !response.status().is_success() {
        return false;
    }
    let body = match response.text().await {
        Ok(body) => body,
        Err(_) => return false,
    };
    let json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(_) => return false,
    };
    match json.get("tag_name").and_then(|tag| tag.as_str()) {
        Some(tag) => is_newer_release(tag, env!("CARGO_PKG_VERSION")),
        None => false,
    }
}

// The one Windows API this file calls directly, declared rather than pulled in through a bindings
// crate because it is a single call in a program that already links shell32 through everything else
// on this platform, and the crate would weigh more than the rest of this dependency added together.
#[cfg(target_os = "windows")]
#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: *mut core::ffi::c_void,
        operation: *const u16,
        file: *const u16,
        parameters: *const u16,
        directory: *const u16,
        show: i32,
    ) -> *mut core::ffi::c_void;
}

// The shell's own mechanism, and not this window's: ShellExecuteW with the open verb hands the
// address to whatever the owner has registered as their browser, which is the same handoff PDEA
// makes with shell.openExternal on this machine. It must not go through the webview, because
// navigating the webview would take the video away from under the person watching it.
#[cfg(target_os = "windows")]
fn open_in_default_browser(url: &str) {
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }
    const SW_SHOWNORMAL: i32 = 1;
    let verb = wide("open");
    let file = wide(url);
    // SAFETY: the signature is ShellExecuteW's, the two pointers it reads are NUL-terminated UTF-16
    // built here and alive across the call, the three that are null are documented as null, and
    // nothing is read back. A return of 32 or below is the shell declining, which this app has no
    // surface to report and does not: a prompt that cannot open a browser says nothing rather than
    // getting in the way of the picture.
    let _ = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };
}

// The same handoff off Windows, through the launcher each of those platforms ships, so this file
// still builds where the app is built for them. The Windows path is the one that ships.
#[cfg(not(target_os = "windows"))]
fn open_in_default_browser(url: &str) {
    let launcher = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(launcher).arg(url).spawn();
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
    // Cloned before the IPC handler takes the original below, because the answer to the update
    // check has to reach the event loop from off it: the request is made by a task, and the task
    // must not be the one holding the window up.
    let update_proxy = proxy.clone();

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
    // The update prompt's hook, installed as an initialization script so that it exists before any
    // page script does. The request below is fired once the window is up, and an answer can arrive
    // before the page has finished parsing; the flag is what such an answer waits on, because
    // without it a prompt that arrived early would be an evaluate_script into a document that is
    // not there yet and would be lost for the rest of the session. It carries no data, only the
    // fact that an answer came, and nothing else in the page can set it.
    let update_init = "window.showUpdateAvailable = function () { if (window.paintUpdatePill) window.paintUpdatePill(); else window.__drUpdatePending = true; };";

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
        .with_initialization_script(update_init)
        .with_ipc_handler(move |msg| {
            let _ = proxy.send_event(msg.body().to_string());
        })
        .build()?;

    // Once per launch, after the window and the webview are up, so the file server, the video and
    // the chrome are all running before anything leaves the machine, and once is the whole of the
    // frequency: GitHub allows about 60 unauthenticated API requests an hour per IP address, and a
    // player that asked more often would spend that budget on one person watching one window.
    // Nothing waits for the answer, the task runs on the runtime the local file server already
    // uses, and a copy bundled inside PDEA never reaches this line at all.
    if !is_embedded_copy() {
        tokio::spawn(async move {
            if newer_release_published().await {
                let _ = update_proxy.send_event("update_available".to_string());
            }
        });
    }

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
                } else if msg == "update_available" {
                    // The page may not have parsed yet, which is what the hook's flag is for.
                    let _ = webview.evaluate_script("window.showUpdateAvailable();");
                } else if msg == "open_release_page" {
                    // The shell, never the webview: the video stays exactly where it is.
                    open_in_default_browser(RELEASES_PAGE);
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