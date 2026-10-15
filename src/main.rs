#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
mod waveform; // the seek bar's audio decode; it is deliberately not in the HTML below, and says why in its own header
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
    /* visibility is in this list and not only opacity, and it is here rather than on the one rule
       that hides this bar for a hold because the base rule is the only state that still has to fade
       back IN: an untransitioned property is a step, so leaving it out would mean the fade out was
       honoured and the fade in was not, or the other way round, and the one that is a step is the
       one a person would notice as the bar popping in. */
    transition: opacity 0.25s, visibility 0.25s;
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

/* ==================== TEMPORARY SPEED: BADGE AND PANEL ==================== */
/* One chip and one panel, and the chip is the seventh item in .winctrl rather than a thing of its
   own, because it is the same control the other six are and nothing about it earns a different row:
   the same 28 of height, the same #303036 fill, the same #5a5a63 edge, the same #3a3a41 hover and
   the same 0.15s, all of them inherited from .wbtn and none of them restated here. It sits
   immediately after the question mark because that button is the one that says what the keys are,
   and a chip that only means anything to somebody who already knows Space is a key belongs beside
   it and nowhere else on this bar.

   It rides the 600px switch below with the update prompt rather than carrying a switch of its own,
   so every threshold on this bar stays exactly where it is and no existing button moves. Above the
   switch it is 60 more of fixed width than nothing, which is the same trade the prompt above
   already makes and the same trade the caption's own overflow and ellipsis exist for. Below the
   switch it is out of the row entirely and the hold still works, because Space reads this speed
   from script and never from the chip: the chip is how the speed is chosen, not how it is applied.

   A pill and not another 28 circle, and that is the one thing here that is not copied from .wbtn.
   Four characters do not fit inside a circle, which is the whole reason the update prompt above is
   28 tall with a 7 radius and a word in it. The radius is 14 rather than 7 because 14 is half of
   28, and a 50% radius on a box wider than it is tall is the ellipse the .winctrl flex: none note
   describes at line 83. min-width 52 is the widest label this control can be given on its own
   terms: "2.5x" is four characters at the 12px the prompt uses, about 6.7 each, so 27 of text,
   9 of padding a side and 1 of border a side, which is 47, and 52 is that plus the few pixels
   another machine's font metrics are worth. Every label the slider can produce is inside that
   floor, so none of them can move the chip, and that is the whole reason for the floor: a badge
   that resized as its digits changed would nudge the buttons on its left every time a hold began.

   position: relative is this button's own line and it is load-bearing rather than tidiness.
   .winctrl is a static box, so an absolutely positioned panel with no positioned ancestor is
   placed against #topbar and comes out under the wrong control entirely. Anchored to the badge it
   lines its right edge up with the badge's own, so the two read as one control rather than as
   something that opened over there. */
.fast-anchor {
    position: relative;
    display: inline-flex;
    align-items: center;
}

#bfast {
    position: relative;
    width: auto;
    min-width: 52px;
    height: 28px;
    padding: 0 10px;
    border-radius: 14px;
    font-size: 12px;
    font-weight: 600;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #303036;
    border: 1px solid #5a5a63;
    color: #e8e8ea;
    cursor: pointer;
    transition: all 0.15s;
}
#bfast:hover {
    background: #3a3a41;
    border-color: #6b6b74;
    color: #ffffff;
}
#bfast[aria-expanded="true"] {
    background: #44444e;
    border-color: #8a8a96;
    color: #ffffff;
}

/* Apple-style Premium Dark Glassmorphic Speed Control Panel */
.fast-panel {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    z-index: 100;
    display: none;
    flex-direction: column;
    gap: 12px;
    width: 224px;
    max-height: calc(100vh - 64px);
    overflow-y: auto;
    scrollbar-width: none;
    padding: 14px 16px;
    background: rgba(20, 20, 26, 0.94);
    backdrop-filter: blur(28px) saturate(180%);
    -webkit-backdrop-filter: blur(28px) saturate(180%);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 14px;
    box-shadow: 0 16px 40px -8px rgba(0, 0, 0, 0.7), 0 0 0 1px rgba(255, 255, 255, 0.05);
    color: #e8e8ea;
    font-size: 12px;
    text-align: left;
    pointer-events: auto;
    transform-origin: top right;
    animation: fastPanelIn 0.18s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}
@keyframes fastPanelIn {
    0% {
        opacity: 0;
        transform: translateY(-6px) scale(0.97);
    }
    100% {
        opacity: 1;
        transform: translateY(0) scale(1);
    }
}
.fast-panel::-webkit-scrollbar {
    display: none;
}
.fast-panel.open {
    display: flex;
}

/* Header */
.fast-panel-header {
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.fast-panel-label {
    display: block;
    font-size: 13px;
    font-weight: 600;
    color: #ffffff;
    line-height: 1.2;
}
.fast-panel-sub {
    color: rgba(255, 255, 255, 0.45);
    font-size: 11px;
    font-weight: 400;
    line-height: 1.3;
}

/* Speed Value Input (Full Row Apple Glassmorphic Card) */
.fast-input-row {
    width: 100%;
    height: 38px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 12px;
    gap: 4px;
    box-sizing: border-box;
    transition: all 0.15s ease;
}
.fast-input-row:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.2);
}
.fast-input-row:focus-within {
    background: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.4);
    box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.12);
}
.fast-input {
    width: 100%;
    height: 100%;
    border: none;
    background: transparent;
    color: #ffffff;
    font-size: 20px;
    font-weight: 600;
    text-align: center;
    font-family: 'SF Mono', 'Consolas', -apple-system, BlinkMacSystemFont, monospace;
    outline: none;
    user-select: text;
    -webkit-user-select: text;
    padding: 0;
    margin: 0;
}
.fast-value-unit {
    font-size: 15px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.55);
    user-select: none;
    pointer-events: none;
}

/* Slider Section */
.fast-slider-wrap {
    display: flex;
    flex-direction: column;
    gap: 5px;
}
.fast-slider-labels {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: rgba(255, 255, 255, 0.45);
    font-weight: 500;
    padding: 0 1px;
}
.fast-slider-center {
    color: rgba(255, 255, 255, 0.3);
    font-size: 9.5px;
}
.fast-slider-track-wrap {
    position: relative;
    display: flex;
    align-items: center;
    height: 16px;
}
.fast-slider {
    width: 100%;
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    margin: 0;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 2px;
    cursor: pointer;
    outline: none;
    transition: background 0.15s ease;
}
.fast-slider::-webkit-slider-runnable-track {
    height: 4px;
    border-radius: 2px;
    background: transparent;
}
.fast-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    margin-top: -5px;
    border-radius: 50%;
    background: #ffffff;
    border: 1px solid rgba(0, 0, 0, 0.15);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
    cursor: pointer;
    transition: transform 0.12s ease;
}
.fast-slider::-webkit-slider-thumb:hover {
    transform: scale(1.15);
}

/* Quick Presets Section */
.fast-presets-wrap {
    display: flex;
    flex-direction: column;
    gap: 5px;
    margin-top: 2px;
}
.fast-presets-label {
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.6px;
    color: rgba(255, 255, 255, 0.4);
    text-transform: uppercase;
}
.fast-presets {
    display: flex;
    gap: 5px;
}
.fast-preset {
    flex: 1 1 0;
    min-width: 0;
    height: 26px;
    padding: 0;
    border-radius: 7px;
    font-size: 11.5px;
    font-weight: 500;
    background: #303036;
    border: 1px solid #5a5a63;
    color: #e8e8ea;
    cursor: pointer;
    transition: all 0.15s;
}
.fast-preset:hover {
    background: #3a3a41;
    border-color: #6b6b74;
    color: #ffffff;
}
.fast-preset.sel {
    background: rgba(255, 255, 255, 0.24);
    border-color: rgba(255, 255, 255, 0.45);
    color: #ffffff;
    font-weight: 600;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
}

/* ==================== TEMPORARY SPEED: YOUTUBE-STYLE SPEED PILL ==================== */
.fast-pill {
    position: fixed;
    top: 18px;
    left: 50%;
    transform: translateX(-50%) scale(0.92);
    z-index: 50;
    display: flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 10px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 12px;
    background: rgba(18, 18, 22, 0.88);
    backdrop-filter: blur(14px);
    -webkit-backdrop-filter: blur(14px);
    color: #ffffff;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.2px;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    visibility: hidden;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.45);
    transition: opacity 0.12s cubic-bezier(0.16, 1, 0.3, 1),
                transform 0.12s cubic-bezier(0.16, 1, 0.3, 1),
                visibility 0.12s;
}
.fast-pill.show {
    opacity: 1;
    visibility: visible;
    transform: translateX(-50%) scale(1);
}
.fast-pill-label {
    color: #ffffff;
    font-weight: 700;
    font-size: 12px;
}

/* ==================== WHAT A HOLD DOES TO THE CHROME ==================== */
/* This feature's own flag on the body, and deliberately NOT hudLock. hudLock is a person having
   pressed H because they want a clean screen, and a hold is not that: they are different claims with
   different owners. Writing a hold into hudLock would make the hold the author of the owner's
   decision, and clearing it on the way out would silently spend an H they pressed back on them.
   Worse, it breaks the case that matters most: a hold begun while H is already on would clear a lock
   the hold never set, and the chrome would come back up over somebody who had asked for it not to.

   So the hold neither reads hudLock nor writes it, in either direction. An H pressed while a key is
   down still toggles hudLock exactly as it would otherwise; it simply has nothing to show for it
   until the key comes up, and then the state it set is the state on screen. That is the difference
   between being swallowed and merely unseen, and only one of them is a bug.

   hudPin is not spent either, for the same reason and one more: it is the show/hide system's own
   bookkeeping about the three second timer, meaning raised by a key or a pointer and not yet given
   back, and it is not a switch a change of playback speed has any business turning.

   The class is the thing that matters, and it is the same shape as body.drawmode and body.fillmode
   around it, which is what makes "the chrome is out of the way" a fact about the tree rather than a
   thing a reader has to remember. showUI reads the class and not the fastActive variable, for a
   reason that is about this file rather than about the feature: showUI runs on the very first line
   of the chrome block, long before the temporary speed block below has been parsed, and a let
   declared there would still be in its temporal dead zone at that call. A class has no such edge.

   The three properties are body.drawmode's, with visibility in place of its display: none, and the
   difference is the fade. display: none is right for draw mode, which is a mode the owner stays in;
   a hold lasts a moment and the chrome coming back is the point, so opacity is what carries the
   0.25s both bars already declare for it. visibility rides along so a bar that has faded out is out
   of the tab order and out of the accessibility tree as well as off the screen, which opacity alone
   does not do, and pointer-events so the quarter of a second it spends leaving cannot swallow a
   press meant for the picture. All three are !important for the reason the drawmode rule is: the
   guarantee that a hold can always put the chrome back has to not depend on a reader working out
   which declaration wins. */
body.fastmode #topbar,
body.fastmode #hud,
body.fastmode .fast-panel {
    opacity: 0 !important;
    visibility: hidden !important;
    pointer-events: none !important;
}

/* ==================== RESUME POSITION PROMPT ==================== */
/* A chip over the picture, and deliberately not a dialog: no backdrop, no modal, nothing taken away
   from the keyboard, and nothing about it stops the video, which starts at 0:00 and plays whether
   the owner reaches for this or not. The chip is one .btn and the panel around it is a position and
   a stacking order and nothing more: no padding, no fill, no edge, no radius, so the panel's box
   is the button's box and the button is the whole of what is drawn. One control is one edge, and a
   frame around a single button is a second thing on the picture to read before the one thing the
   chip has to say.

   The panel still pins it, 16 from the left and 56 up, which clears the 41 that #hud occupies along
   the bottom and the 8px band above the bottom edge that belongs to the resize handler, so a press
   anywhere on it cannot resize the window or reach the row.

   Opaque #e00 with a #e00 edge, for the reason the rate note and the draw bar declare their
   colours instead of washing: over an arbitrary frame there is no determinate background to read
   against, so the fill, the label and the boundary all come from declared values. The shadow is on
   the button rather than the panel because a shadow is the one thing here drawn outside the chip's
   own edge, and a panel that paints nothing has no edge to draw outside of: left there it would
   have been a shadow around nothing, and the button against the video is the one thing that needs
   a lift.

   There is one action and it is the whole of the chip, so the saved time is the label on that
   button instead of a note beside it: "Continue from 00:12:34" says what the press will do and
   leaves no second answer sitting next to it to be chosen against. The time is fmt(), the
   formatter the two readouts on the bottom row already use, rather than a shorter format of its
   own, because a number in this window is spelled one way. The button is a .btn, so it is in the
   Tab order and takes Enter and Space with no key binding of its own, and it is the same chip with
   the app's own red on it rather than a new colour: #fff on #e00 is 4.5:1 and #fff on the #b00 it
   darkens to is 6.8:1, so the label holds its contrast on the way in and on the way down, which
   the lighter red a hover usually reaches for does not. */
.resume-panel {
    position: fixed;
    left: 16px;
    bottom: 56px;
    display: none;
    z-index: 25;
}
.resume-panel.show {
    display: block;
}
.resume-go {
    background: #e00;
    border-color: #e00;
    color: #fff;
    box-shadow: 0 6px 20px rgba(0,0,0,0.55);
}
.resume-go:hover, .resume-go:active {
    background: #b00;
    border-color: #b00;
    color: #fff;
}
/* The focus ring the rate field and the shortcuts panel already use, and load-bearing here rather
   than tidy: the panel takes no focus of its own, so this one button is the only way into it, and
   a keyboard user has to be able to see where they have landed once they are on it. */
.resume-go:focus {
    outline: 2px solid #e00;
    outline-offset: 2px;
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
    /* visibility for the same reason and with the same consequence as the top bar's: it is what
       takes a faded-out bar out of the tab order and out of the accessibility tree, and it only
       fades if it is in the transition list. */
    transition: opacity 0.25s, visibility 0.25s;
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
   is its own left edge wherever that lands. The waveform canvas is positioned against this same box
   with no inset of any kind, so it spans exactly the same rectangle and the mapping above it is
   unchanged by it being there. */
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
/* A thin center line, 2px on the #111114 backing: 35% white composites to #646466, the 3:1
   a control has to reach against the surface it sits on. Thin on purpose — it is the exact spine
   the waveform below grows around, so decoded waves appear in place with no layout shift and no
   swap between two different-looking bars. Always visible: the canvas behind only ever adds the
   audio texture, never replaces this line. */
.seek-track {
    width: 100%;
    height: 2px;
    background: rgba(255,255,255,0.35);
    border-radius: 1px;
    position: relative;
    transition: height 0.12s;
    z-index: 2;
}
.seekbar-container:hover .seek-track {
    height: 3px;
}
/* The played portion: a white fill pinned to the left edge, its width written from script on every
   timeupdate. This is the one element that says where in the file the picture is, and it answers
   whether or not any audio has been decoded yet. */
.seek-fill {
    position: absolute;
    left: 0; top: 0; bottom: 0;
    width: 0%;
    background: #fff;
    border-radius: 2px;
    pointer-events: none;
}
/* The playhead: a vertical tick on the same center as everything else, 18px tall in a 28px
   row so it clears the tallest wave column without touching the buttons beside it. Always
   visible rather than hover-only, because the position has to read at a glance on a file whose
   waveform has not arrived yet. 2px wide like the line it stands on, so played-vs-unplayed reads
   from the fill and the waves on either side of it rather than from a dot sitting on top. */
.seek-tick {
    position: absolute;
    top: 50%;
    left: 0%;
    width: 2px; height: 18px;
    border-radius: 1px;
    background: #fff;
    transform: translate(-50%, -50%);
    pointer-events: none;
    box-shadow: 0 1px 4px rgba(0,0,0,0.6);
}
/* The waveform texture behind the progress line, drawn once batches arrive and filling in from the
   left while the decode runs. 24 tall against the container's 28px floor, so the tallest column
   keeps 2px of the row's own backing above and below it and the 41px row never moves.

   Behind the track (z-index 1 under the track's 2) rather than in place of it: the line, the fill
   and the knob above are what say where the picture is, and the canvas only ever adds what the
   audio looks like. Its background is transparent — clearRect, never a fill — so where nothing has
   been decoded yet the plain bar simply shows through, and the bar is complete from the first
   frame rather than only once the decode finishes.

   Absolute with both horizontal offsets declared so its left edge is the container's left edge,
   which is what the seek mapping below measures against. width: auto is stated because a canvas
   carries an intrinsic 300 by 150, and a 300px-wide flex item in this row would push every button
   beside it along. pointer-events: none keeps the press on the container and its 28px hit area. */
#seek-wave {
    position: absolute;
    left: 0; right: 0;
    top: 50%;
    transform: translateY(-50%);
    width: auto;
    height: 24px;
    display: none;
    pointer-events: none;
    z-index: 1;
}
#seekbar-container.waveform #seek-wave { display: block; }

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
   focus indicator and #e00 on this field's own #1c1c20 is 4.1:1.

   This class is not only the bottom row's field. The temporary speed panel's big value takes it too,
   and it takes it rather than a field of its own so the fill, the edge, the monospace, the centring
   and the focus border are one declaration rather than two that have to be kept in step. What that
   block overrides is only its own size: the width, because the 6ch floor above exists to stop the
   bottom row's layout moving as the digits change and that block has no row to move, and the height,
   padding, radius and font size, because it is the panel's big value rather than a readout in a
   41px row. Nothing about what the field looks like is decided twice. */
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

/* The footer carries the way out that is not on the list itself, so the panel can be closed from
   where the eye already is instead of only from the keyboard. */
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
   up here and 347 on the 0.4 the caption does not claim is 868: that, not 600, is the width at
   which the caption is given its whole 60% again. Between 600 and 868 it is up to 107 narrower
   than it was, which is the trade its own overflow and ellipsis exist for, and dismissing the
   prompt hands every one of those pixels back, because a display: none item is out of the row and
   the flex line is laid out again without it.

   The temporary speed chip rides this rule as well, and for the same reason and on the same
   grounds: it is a seventh item on this same row of this same arithmetic, and a display: none item
   costs the row nothing above the switch and is out of it below, so neither threshold moves and no
   existing button loses a pixel. Above the switch it is 60 of fixed width, which is the same trade
   the prompt above is already paying and the same one the caption's ellipsis is already standing
   ready for. Below the switch it is gone and the hold it sets up still works, because Space reads
   that speed from script and never from the chip: the chip is how the speed is chosen, not how it
   is applied. */
@media (max-width: 600px) {
    .title-capsule, #update-pill, #bhide, #bfill, #bmin, #bfast, .fast-anchor { display: none; }
    .drag-handle { inset: 8px 8px 0 8px; }
}
/* Help is the last thing to go on this bar, and it is the one control that says what the keys are:
   a window too narrow for the caption has already lost the file name, and this is the control that
   puts the shortcuts back within reach. It has its own switch rather than riding the one above
   because it is not on the same arithmetic: with the caption gone, what is left up here is this
   button, the pen and the cross, which is 3 x 28 plus 2 x 8, so 100, and 100 plus the bar's own 32
   of padding is 132, which is the narrowest window that can hold all three without pushing the last
   one off the right edge. The temporary speed chip is not in that count because the 600px rule
   above has already taken it, and that is the whole reason it rides that rule rather than carrying
   one of its own: a seventh item that only appeared at this width would need a third switch and a
   second set of arithmetic. Below it the pen and the cross stay, because those two are the only way
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

<!-- Resume position, and it is display: none until the window tells the page this file was left
     part-watched, which nothing in this file can decide: the saved seconds arrive from the window
     with the video URL, and the offer itself waits for the metadata, because every one of the
     thresholds is a comparison against the duration. Sits over the picture, takes no focus when it
     appears (a panel that grabbed the keyboard would turn the first Space into a resume rather than
     a pause) and stops both mousedown and keydown, for the reasons its own script gives. One
     button and one action, with the saved time in its own label: there is no second control here to
     press, so nothing on this chip is a way of refusing it. -->
<div class="resume-panel" id="resume" role="group" aria-label="Resume playback">
    <button class="btn resume-go" id="resume-go" type="button">Continue from 00:00:00</button>
</div>

<!-- What a hold of Space leaves on screen, and the only thing it leaves: one pill carrying the speed
     while the top bar, the bottom row and the resume chip are all out of the way. It is a sibling of
     both of those and a child of neither, because both are what goes away, and it is fixed at the
     top centre of the window because that is where YouTube puts it and where the eye already goes
     for it.

     It takes no focus, takes no press and is out of the accessibility tree for as long as this page
     is open, and the reasons for all three are at length in the stylesheet and beside enterFast: it
     is a reflection of a key that is physically down rather than an answer to a question, and it
     has no control in it to answer with. The number is not withheld from anybody: the field in the
     bottom row and the chip in the top bar both state it and both can be reached.

     The string inside is written by setTempRate with everything else that reads tempRate, so it
     cannot go stale against the chip that shows the same number, and the class on the element is
     the only other thing written to it, by the two halves of the hold. -->
<div class="fast-pill" id="fast-pill" aria-hidden="true"><span id="fast-pill-label">2x</span></div>

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
         nothing to the button's name, so title is what a screen reader reads.

         The seventh item in this row is not a window button and is counted separately for that
         reason: the temporary speed chip and the panel it opens, described at length in the
         stylesheet and in the script below. It sits here, between the question mark and the eye,
         and not down on the bottom row beside the persistent number, because the two are different
         speeds and putting them side by side on one bar is the mistake the whole pair exists to
         prevent. Nothing on the bottom row is moved, hidden or removed by any of it. -->
    <div class="winctrl">
        <button class="wbtn" id="bhelp" title="Keyboard shortcuts (/)">?</button>
        <!-- The temporary speed, which is the speed Space gives for as long as it is held and only
             for that long. The chip is a chip because it is the only control that sets that number,
             and it shows the number rather than the word HOLD because a chip that had to change
             what it says depending on whether a key was down would be a different chip in each of
             its two states, and two chips is what a bar must not have.

             The label inside is aria-hidden and the accessible name is on the button instead,
             because "2x" is what a screen reader would otherwise announce for this control and
             "2x" says nothing about what it is a control for. The script writes that name with the
             current number in it, so the name cannot go stale against the chip it describes, and it
             ends with the chip's own text so the two agree for SC 2.5.3.

             The panel is a sibling of the chip rather than a child of it, anchored to it from below
             in the stylesheet, and inside #topbar so the bar's own three-second fade and draw
             mode's display: none take it away with the chip it belongs to. It reads top to bottom
             as what this speed is, what it is now, how to move it and the speeds a hold is for,
             which is the order YouTube's November 2024 panel reads in and not the tall stack of rows
             that panel replaced. The big number IS the field rather than sitting beside a small
             one, so clicking the value is the way to type a new one and there is no second box to
             find. Four blocks, every one of which applies the moment it is touched: no OK, because
             there is nothing to confirm, and no cancel, because there is nothing left to discard.
             Escape and a press anywhere outside are the whole of the way out. -->
        <div class="fast-anchor">
            <button class="wbtn" id="bfast" title="Temporary speed" aria-expanded="false"
                    aria-label="Temporary speed: 2x"><span id="fast-badge-label" aria-hidden="true">2x</span></button>
            <div class="fast-panel" id="fast-panel" role="group" aria-labelledby="fast-panel-label">
                <div class="fast-panel-header">
                    <label class="fast-panel-label" id="fast-panel-label" for="fast-input">Hold Speed</label>
                    <p class="fast-panel-sub">Active while holding screen or Space</p>
                </div>

                <!-- Speed Value Input (Full Row) -->
                <div class="fast-input-row">
                    <input class="fast-input" id="fast-input" type="text" role="spinbutton"
                           aria-valuemin="0.0625" aria-valuemax="16" aria-valuenow="2" aria-valuetext="2.00 times"
                           value="2.00" inputmode="decimal" spellcheck="false" autocomplete="off">
                    <span class="fast-value-unit">x</span>
                </div>

                <!-- Slider with labels -->
                <div class="fast-slider-wrap">
                    <div class="fast-slider-labels">
                        <span>0.06x</span>
                        <span class="fast-slider-center">Drag to adjust</span>
                        <span>16x</span>
                    </div>
                    <div class="fast-slider-track-wrap">
                        <input class="fast-slider" id="fast-slider" type="range" min="0.0625" max="16" step="0.05" value="2"
                               aria-label="Temporary speed slider">
                    </div>
                </div>

                <!-- Quick Presets -->
                <div class="fast-presets-wrap">
                    <span class="fast-presets-label">QUICK PRESETS</span>
                    <div class="fast-presets">
                        <button class="fast-preset" type="button" data-rate="1.5" aria-pressed="false">1.5x</button>
                        <button class="fast-preset sel" type="button" data-rate="2" aria-pressed="true">2x</button>
                        <button class="fast-preset" type="button" data-rate="3" aria-pressed="false">3x</button>
                        <button class="fast-preset" type="button" data-rate="4" aria-pressed="false">4x</button>
                    </div>
                </div>
            </div>
        </div>
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
            <!-- The plain line is always in the DOM and always visible: fill and tick say where
                 the picture is from the first frame, waveform or not. The canvas behind it only
                 ever adds the audio texture, so neither is named, focusable or given a role. -->
            <div class="seek-track" id="seek-track">
                <div class="seek-fill" id="seek-fill"></div>
                <div class="seek-tick" id="seek-tick"></div>
            </div>
            <canvas id="seek-wave"></canvas>
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
     because the owner read "Fullscreen on or off" twice over as two separate things to look up.

     Space is printed twice, once in Playback and once in Speed, and it is the one place that rule
     above cannot be followed: it is one key reaching two different actions depending on how long it
     is down, and a single row could only ever state one of them. Printing only the tap would leave
     the hold undiscoverable in the one place in this app that exists to list the keys. -->
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
            <li class="keys-row">
              <span class="keys-desc">Hold to play fast, and back on release</span>
              <span class="keys-keys"><kbd class="keys-key">Space</kbd></span>
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
    // The temporary speed panel goes down with the bar it is anchored to, by the same one call and
    // for the same reason the resume chip gets from hideResumeWithChrome. It is not cosmetic: the
    // panel is inside #topbar, so the bar's own fade already takes it off screen, and an item that
    // is invisible while still remembering that it is open comes back on the next mouse move still
    // open, which would spend the chip's next press closing something nobody can see.
    closeFast();
    topbar.classList.remove('show');
    hud.classList.remove('show');
    renderHideIcon();
    applyBodyCursor();
}
function showUI() {
    // body.fastmode is in this list beside body.drawmode and for the same reason, and it is the
    // class rather than a variable because this function is called on the first line of the chrome
    // block, long before the temporary speed block has been parsed and a let declared there would
    // still be in its temporal dead zone here. While a key is held the chrome is out of the way
    // whatever this returns, so the refusal is about not disturbing the state behind it: nothing
    // during a hold may raise the chrome or re-arm the timer that would take it down, and the hold
    // restores the state it found rather than writing one.
    if (hudLock || hudPin || window.spaceDown || window.pointerDown || document.body.classList.contains('drawmode') || document.body.classList.contains('fastmode')) return;
    showControls();
    clearTimeout(hideT);
    // While video is playing, auto-hide chrome after 3 seconds of idle. While paused, keep controls visible.
    if (!vid.paused) {
        hideT = setTimeout(() => {
            if (hud.contains(document.activeElement)) { showUI(); return; }
            hideControls();
        }, 3000);
    }
}
// H hides the chrome and brings it back pinned
// The chip rides the explicit toggle only, so the three second idle timer in showUI cannot take it
// down with the chrome; see the note on hideResumeWithChrome for why the two are not the same hide.
function toggleControls() {
    if (topbar.classList.contains('show') || hud.classList.contains('show')) {
        hudLock = true;
        hideControls();
        hideResumeWithChrome();
    } else {
        hudLock = false;
        hudPin = true;
        showControls();
        showResumeWithChrome();
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
    window.ipc.postMessage('fill_mode:off');
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
    // Which way it went, told to the window because the window remembers this window's geometry and
    // a shape fill mode chose for itself is the video's shape rather than a size the owner picked.
    // Both writers of fillMode above are the only two, so this cannot disagree with the icon.
    window.ipc.postMessage(fillMode ? 'fill_mode:on' : 'fill_mode:off');
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
    if (e.code === 'Escape' || e.code === 'Space') return;
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

// A second explicit hide of its own rather than a caller of toggleControls, so the chip is taken
// down and brought back here as well. The chip's call is the same one for the same reason, and
// showUI is still what raises the chrome rather than showControls: this button does not pin.
bhide.addEventListener('click', () => {
    hudLock = !hudLock;
    if (hudLock) { hideControls(); hideResumeWithChrome(); }
    else { showUI(); showResumeWithChrome(); }
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

/* ====================== MOVE THE WINDOW OR HOLD FOR SPEED ====================== */
document.addEventListener('mousedown', (e) => {
    beginPointerPress(e);
});

document.addEventListener('mousemove', (e) => {
    if (pointerDown && !fastActive) {
        const dx = e.clientX - pointerStartX;
        const dy = e.clientY - pointerStartY;
        if (Math.hypot(dx, dy) > 7) {
            pointerMoved = true;
            clearTimeout(pointerHoldTimer);
            pointerHoldTimer = null;
            // With chrome hidden, dragging moves the window
            if (!topbar.classList.contains('show') && !hud.classList.contains('show')) {
                pointerDown = false;
                window.pointerDown = false;
                window.ipc.postMessage('drag_window');
            }
        }
    }
});

document.addEventListener('mouseup', (e) => {
    resizing = false;
    resizeDir = '';
    resizePending = false;
    endPointerPress(e);
});

window.addEventListener('blur', () => {
    resizing = false;
    resizePending = false;
    seeking = false;
    vDrag = false;
    dragData = null;
    drawing = false;
    selShape = null;
    endSpacePress();
    endPointerPress();
});

/* ====================== WINDOW BUTTONS ====================== */
document.getElementById('bmin').onclick = () => window.ipc.postMessage('minimize');
// Where I left off is written before the close goes out, because the close is what takes the page
// with it: this is the last moment the position is known to anything. Both messages are posted in
// this order and the first is never dropped on account of the second, so a video quit halfway
// through is still a video that opens offering to continue.
document.getElementById('bcls').onclick = () => {
    saveResumePosition();
    window.ipc.postMessage('close');
};
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
/* The bar is one control and it has always been one: the same 28px target, the same container
   rect, the same 0 to 1 mapping, the same two readouts. What is drawn inside it changed, not what
   it does. The mousedown stays on the container rather than moving onto the canvas, because a 24px
   canvas in a 28px box is 2px of dead strip top and bottom and the target would silently drop to
   exactly WCAG 2.2's 24 by 24 floor with no slack. */
const seekContainer = document.getElementById('seekbar-container');
const seekTrackEl = document.getElementById('seek-track');
const seekFillEl = document.getElementById('seek-fill');
const seekTickEl = document.getElementById('seek-tick');
const waveCanvas = document.getElementById('seek-wave');
const waveCtx = waveCanvas.getContext('2d');
const tcEl = document.getElementById('tc');
const trEl = document.getElementById('tr');
let seeking = false;

function updateSeekbar() {
    if (!vid.duration) return;
    // The bar's own position, answered here and nowhere else: a white fill and a tick, moved on
    // every timeupdate whether or not any audio has been decoded yet. This is what makes the bar
    // a working seek bar on a 3-hour file whose decode is still running.
    const pos = Math.max(0, Math.min(1, vid.currentTime / vid.duration));
    tcEl.textContent = fmt(vid.currentTime);
    trEl.textContent = '-' + fmt(vid.duration - vid.currentTime);
    seekFillEl.style.width = (pos * 100) + '%';
    seekTickEl.style.left = (pos * 100) + '%';
    paintWaveform();
}
vid.addEventListener('timeupdate', updateSeekbar);
vid.addEventListener('loadedmetadata', updateSeekbar);

function seekFromEvent(e) {
    // The guard updateSeekbar has always had, and this needed before it did not: before the
    // waveform the target was a 4px line that was hard to hit by accident, and now it is a 24px
    // one that is not. vid.duration is NaN until loadedmetadata, and writing currentTime = NaN
    // puts the element into a state no later seek recovers it from.
    if (!vid.duration) return;
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

/* ====================== WAVEFORM ====================== */
/* The peaks of the file's own first audio track, decoded in Rust on a thread that is started only
   after this page says it is ready and arriving here in batches of 512 at 20ms a column. Four
   pieces of state and nothing else: the columns, the bucket width they are measured in, the
   decoded length of the track, and the loudest sample seen so far.

   That last one is the normalisation, and it is a deliberate trade. Peaks arrive as linear
   amplitudes in 0 to 1, which is exactly what the decoder hands back, and dividing by the running
   maximum is what makes a quiet recording read as a quiet recording with a shape rather than as an
   absent bar. What it costs is that two files shown by the same player each show their own
   maximum, so a quiet file looks as tall as a loud one. The peaks keep full resolution either way;
   only the display is relative to the file. The other cost is visible for about a second: the
   maximum grows as batches land, so the bar can rescale once when a louder sample arrives. On real
   content the first batch almost always carries one, so it settles immediately.

   Below the quiet floor there is nothing to normalise against and every column is drawn at the
   minimum instead. A bucket at 0.001 of full scale is a hundredth of a pixel and rounds to
   nothing, so a file that is silence rather than a recording would otherwise draw an empty bar,
   and a flat thin line says what is actually there. */
const WAVE_CSS_HEIGHT = 24;      // the height: 24px in the #seek-wave rule above
const WAVE_MIN_HALF_PX = 1;      // one device pixel: silence, and not yet decoded
const WAVE_QUIET_FLOOR = 0.002;  // under this the file is silence, not a quiet recording
// Played bars are the fill's own white and unplayed bars are the track's own 35% white, so the
// waves carry position the same way the line does: left of the tick white, right of it grey. Same
// center, same tick, same mapping — the decoded region is the thin line grown tall, not a second
// control overlaid on the first.
const WAVE_PLAYED = '#ffffff';
const WAVE_UNPLAYED = 'rgba(255,255,255,0.35)';

let wavePeaks = [];
let waveBucket = 0;
let waveDuration = 0;
let waveMax = 0;

function paintWaveform() {
    // Nothing decoded yet, or nothing to decode into: the plain bar below is already complete, so
    // there is nothing to add and this costs one test. That is the whole of what a file with no
    // audio costs, four times a second.
    //
    // vid.duration is NaN until loadedmetadata, and for a file the engine refuses to play it stays
    // NaN for the whole session, because onerror only rewrites the title. The guard is therefore
    // also what keeps the canvas hidden on the three fixtures in exactly that state
    // (sample-5s.avi, sample-15s.avi, test1-matroskaconformance.mkv): peaks without a duration is
    // a canvas with no time axis, and an empty 28px gap is what the bar must never become.
    if (!waveBucket || !wavePeaks.length || !vid.duration) return;

    // Measured on every call rather than cached behind a resize listener, because this is the one
    // canvas in this file that has to be right on a display that is not at 100%: a change of
    // devicePixelRatio fires no event that all three webview engines agree on, and re-reading one
    // rectangle is cheaper than the media-query bookkeeping a listener would be. The rectangle is
    // the container's and not the canvas's, so it is the same one the click and the drag measure.
    const cssW = seekContainer.getBoundingClientRect().width;
    if (!(cssW > 0)) return;
    const dpr = window.devicePixelRatio || 1;
    const bw = Math.round(cssW * dpr);
    const bh = Math.round(WAVE_CSS_HEIGHT * dpr);
    if (waveCanvas.width !== bw || waveCanvas.height !== bh) {
        waveCanvas.width = bw;
        waveCanvas.height = bh;
    }
    const W = waveCanvas.width;
    const H = waveCanvas.height;

    // Device pixels throughout, because fillRect paints into the backing store and no scale is
    // applied to this canvas's context. At DPR 1 that is floor(24/2) - 1 = 11, so a full column is
    // 22 of the 24 with a pixel of air above and below it.
    const mid = Math.floor(H / 2);
    const maxHalf = Math.max(WAVE_MIN_HALF_PX, mid - 1);
    const minHalf = WAVE_MIN_HALF_PX;

    // Where the decode has got to, measured against what is actually known: while it is still
    // running that is simply how far it has got, and afterwards it is the decoded length. Columns
    // past it are left transparent rather than drawn flat, so the bar fills in from the left over
    // the following seconds and the undecoded right side is the plain line, not an absence.
    const received = wavePeaks.length;
    const decoded = received * waveBucket;
    const audioEnd = waveDuration > 0 ? Math.min(waveDuration, decoded) : decoded;

    const quiet = waveMax < WAVE_QUIET_FLOOR;
    const duration = vid.duration;
    const progressSeconds = vid.currentTime;

    seekContainer.classList.add('waveform');
    waveCtx.clearRect(0, 0, W, H);
    for (let x = 0; x < W; x++) {
        const t0 = (x / W) * duration;
        if (t0 >= audioEnd) continue;
        const t1 = ((x + 1) / W) * duration;
        // Max over the buckets this column covers, never the mean and never the last: the mean
        // drops a transient that lands in one bucket of five, and the last depends on how the
        // rounding fell.
        const i0 = Math.floor(t0 / waveBucket);
        const i1 = Math.max(i0 + 1, Math.ceil(t1 / waveBucket));
        const stop = Math.min(i1, received);
        let peak = 0;
        for (let i = i0; i < stop; i++) {
            if (wavePeaks[i] > peak) peak = wavePeaks[i];
        }
        // Past what has arrived is the decode not having reached this time yet; the continue above
        // already left it transparent.
        if (i0 >= received) continue;
        const value = (quiet || waveMax <= 0) ? 0 : peak / waveMax;
        let hh = Math.round(value * maxHalf);
        if (hh < minHalf) hh = minHalf;
        if (hh > maxHalf) hh = maxHalf;
        // The played and unplayed colours split between two columns rather than across one, so the
        // tick at the same x always lands on the change instead of half a pixel into it.
        waveCtx.fillStyle = t0 < progressSeconds ? WAVE_PLAYED : WAVE_UNPLAYED;
        waveCtx.fillRect(x, mid - hh, 1, hh * 2);
    }
}

window.setWaveform = rec => {
    if (rec.failed) {
        // A decode that could not finish leaves the plain bar exactly as it was and forgets what
        // it had: half a waveform measured against the video's clock claims audio where there is
        // none. This is also the one path a file with no audio, an unreadable track and a
        // container this build cannot demux all arrive at, and none of them is anything to show.
        wavePeaks = []; waveBucket = 0; waveDuration = 0; waveMax = 0;
        seekContainer.classList.remove('waveform');
        return;
    }
    // The bucket width is stated by the sender rather than assumed here, so there is one number
    // and one place it is written down. The track's length is read whenever a positive one
    // arrives: the first Meta carries what the container claims and the last carries what was
    // decoded, and the decoded one wins.
    if (typeof rec.bucketSeconds === 'number' && rec.bucketSeconds > 0) waveBucket = rec.bucketSeconds;
    if (typeof rec.durationSeconds === 'number' && rec.durationSeconds > 0) waveDuration = rec.durationSeconds;
    if (Array.isArray(rec.peaks)) {
        const start = typeof rec.start === 'number' && rec.start >= 0 ? rec.start : wavePeaks.length;
        for (let i = 0; i < rec.peaks.length; i++) {
            const p = rec.peaks[i];
            wavePeaks[start + i] = p;
            if (p > waveMax) waveMax = p;
        }
    }
    // Shown on arrival, progressively: the canvas is what knows whether there is anything to paint
    // into yet, and a batch that arrived before loadedmetadata waits for the duration.
    paintWaveform();
};

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

/* ====================== TEMPORARY SPEED: HOLD SPACE ====================== */
/* Two speeds, both of which stay on screen and neither of which replaces the other. The number on
   the bottom row is the speed in effect when nothing is held, and it is the field this file
   already had: it is not moved, not hidden, not renamed, and nothing in this block writes to it
   directly at all. This is the other one, the speed while Space is physically down, and the only
   thing that ever applies it is a key that is currently down.

   WHY THE HOLD WAITS A QUARTER SECOND AND DOES NOT ACT ON KEYDOWN. The delay is the whole design
   and it is not a number anybody chose for taste. One key has two meanings here and they are told
   apart by how long it is down, which means the app has to know, at the moment it would have to act,
   whether this is going to turn out to be a tap or a hold. Acting on keydown cannot: a press and a
   release are the same two events and by the time the first one arrives nobody knows whether the
   second is coming, so acting then takes play/pause away outright. Acting on keyup alone is worse
   and in the opposite direction: the picture would sit at 1.00 for a quarter of a second on every
   play and every pause, so the speed control would flicker on the most-used key in the app and
   every pause would look like a glitch. Between the two, the timer is what lets a tap stay exactly
   the tap it has always been: nothing at all is written to the video until the timer fires, so a
   key that goes up again inside it leaves the rate and the paused flag untouched and the keyup
   path below does precisely what the Space case did before there was a timer here.

   And the two cannot both have happened, which is the property that makes either of them safe. By
   the time the callback runs, a release has not happened, so the keyup path will see fast mode up
   and do the restore and nothing else; by the time a release arrives without fast mode up, the
   timer either has not fired or never will, so the restore cannot be needed. A tap never flickers
   the speed because a tap never reaches the video at all, and a hold never pauses because a hold
   never reaches the toggle.

   The value itself. 250ms is under the 300 to 500 that a deliberate press measures and far over
   the roughly 50ms autorepeat delay, which is what makes a held key unmistakably a hold rather
   than a run of short taps. It is also short enough that nobody describes the feature as laggy,
   and long enough to separate the two gestures by the only signal a keyboard has. Three quarters
   of a second was considered and rejected: nobody watches a quarter of a second of 4x in their
   head as fast, and a threshold that long reads to the hand as no threshold at all.

   WHY FAST MODE IS TORN DOWN ON BLUR. Because the worst outcome in this feature is not a wrong
   speed, it is a stuck one. A user who holds Space and alt-tabs releases the key into another
   window, and this page never hears about it: the timer has already fired, fast mode is entered,
   and the only thing in the whole file that could undo it is the keyup that is never going to
   come. The picture then stays at the temporary speed, and keeps playing, and the bottom row's
   field reads that speed because applyRate renders it, and there is nothing on screen to say why.
   The user closes the window and comes back to a video running four times too fast. So the restore
   is wired to the window blur handler as well as to the keyup, in the same place the six other
   half-finished gestures are dropped, and by the same one call, which also covers the softer
   version: a blur inside the quarter second is a tap that never completes, so the timer is cleared
   and the keyup that arrives on the way back finds wasDown false and does nothing at all instead
   of pausing a picture nobody touched.

   That call is from the blur handler and not from here because here is too late: this block is
   the end of the file's script and the blur handler is registered long before it, so a function
   declared below would not exist when the listener was registered if declarations did not hoist.
   They do, and endSpacePress is one of them, which is the only reason a function defined here can
   be called from there at all. Nothing about the gesture depends on the window being there when
   the key goes up. */
const HOLD_MS = 250;
const fastBtn    = document.getElementById('bfast');
const fastPanel  = document.getElementById('fast-panel');
const fastInput  = document.getElementById('fast-input');
const fastSlider = document.getElementById('fast-slider');
const fastLabel  = document.getElementById('fast-badge-label');
const fastPill   = document.getElementById('fast-pill');
const fastPillLabel = document.getElementById('fast-pill-label');
const fastMinus  = document.getElementById('fast-minus');
const fastPlus   = document.getElementById('fast-plus');
// The four preset buttons, read once. They are the only writers of a whole number besides the field
// and they all carry the value they set, so the markup is the table and there is no second list here
// to keep in step with it. setTempRate is what marks which one is current, and it compares against
// the same number rather than against the text, so 2 typed by hand and 2 pressed here are the same
// answer to "is this one chosen".
const fastPresets = Array.from(document.querySelectorAll('.fast-preset'));

// The slider spans the full range from RATE_MIN (0.0625x) to RATE_MAX (16x).
const FAST_SLIDER_MIN = 0.0625;
const FAST_SLIDER_MAX = 16;

// The four preset speeds are in the markup on each button's data-rate (1.5x, 2x, 3x, 4x)
// and provide instant one-click selection.

// The temporary speed itself.
let tempRate = 2;
let fastOpen = false;
// The press flags. Also exposed on window so showUI can check them before TDZ.
window.spaceDown = false;
window.pointerDown = false;
let spaceDown = false;
let holdTimer = null;
// Fast mode's own three facts, all written on the way in and all read only on the way out.
let fastActive = false;
let fastSavedRate = 1;
let fastWasPaused = true;

// The chip reads 2x and 2.5x where fmtRate reads 2.00 and 2.50, because fmtRate is the bottom row's
// formatter and the bottom row holds a six-character field while this is a 28-tall chip: a whole
// number prints bare and everything else prints at the four decimals the 0.0625 floor needs and no
// further. toFixed(4) then parseFloat, so trailing zeros are dropped rather than rounded twice, and
// 2.5 comes out as 2.5 rather than 2.50.
function fmtFast(v) {
    return (Number.isInteger(v) ? String(v) : String(parseFloat(v.toFixed(4)))) + 'x';
}

function updateFastSliderFill() {
    if (!fastSlider) return;
    const min = FAST_SLIDER_MIN;
    const max = FAST_SLIDER_MAX;
    const val = Math.min(max, Math.max(min, tempRate));
    const pct = ((val - min) / (max - min)) * 100;
    fastSlider.style.background = `linear-gradient(to right, rgba(255,255,255,0.7) 0%, rgba(255,255,255,0.7) ${pct}%, rgba(255,255,255,0.2) ${pct}%, rgba(255,255,255,0.2) 100%)`;
}

function setTempRate(v, writeField) {
    if (!Number.isFinite(v)) return false;
    tempRate = Math.min(RATE_MAX, Math.max(RATE_MIN, v));
    fastLabel.textContent = fmtFast(tempRate);
    fastBtn.setAttribute('aria-label', 'Temporary speed: ' + fmtFast(tempRate));
    fastPillLabel.textContent = fmtFast(tempRate);
    fastPresets.forEach(btn => {
        const chosen = Number(btn.dataset.rate) === tempRate;
        btn.classList.toggle('sel', chosen);
        btn.setAttribute('aria-pressed', chosen ? 'true' : 'false');
    });
    if (writeField) {
        fastInput.value = fmtRate(tempRate);
        fastInput.setAttribute('aria-valuenow', String(tempRate));
        fastInput.setAttribute('aria-valuetext', fmtRate(tempRate) + ' times');
    }
    if (document.activeElement !== fastSlider) {
        fastSlider.value = String(Math.min(FAST_SLIDER_MAX, Math.max(FAST_SLIDER_MIN, tempRate)));
    }
    updateFastSliderFill();
    if (window.ipc && window.ipc.postMessage) {
        window.ipc.postMessage('temp_rate:' + tempRate);
    }
    return true;
}

window.setStoredTempRate = (r) => {
    if (typeof r === 'number' && Number.isFinite(r) && r >= RATE_MIN && r <= RATE_MAX) {
        setTempRate(r, true);
    }
};

// The three states this control must not answer in, each of which already has an early return
// somewhere else in this file and each of which is copied here rather than reinvented. helpOpen is
// the one boolean the shortcuts panel owns, and it is modal and takes the whole keyboard while it
// is up. The draw bar takes the top bar away with display: none, so the chip cannot be pressed at
// all there, and its own handler at the foot of this file takes the keyboard as well. The bottom
// row's rate field is the third: the keystrokes that would arrive here never get that far, because
// that field stops propagation on its own keydown, which is the same stop the panel above makes and
// the reason a space typed into either field is a space and not a pause.
function fastBlocked() {
    if (helpOpen) return true;
    if (drawbar.classList.contains('open')) return true;
    if (document.activeElement === rateInput) return true;
    return false;
}

function openFast() {
    if (fastOpen) return;
    fastOpen = true;
    fastPanel.classList.add('open');
    fastBtn.setAttribute('aria-expanded', 'true');
    setTempRate(tempRate, true);
    // The chrome's own clock, re-armed from the control that opens a panel over the picture. The
    // panel lives in #topbar, which fades after three seconds, so a panel left up by the idle timer
    // would be a panel nothing can see.
    showUI();
}

function closeFast() {
    if (!fastOpen) return;
    fastOpen = false;
    fastPanel.classList.remove('open');
    fastBtn.setAttribute('aria-expanded', 'false');
    // The same call closeHelp makes and for the same reason: a panel taken off screen with the
    // keyboard still sitting in it is the incident this app has already had once, and the guard is
    // cheap because nothing is focused on the way in. This is the only place a focus can be let go
    // of, which is the other half of never taking one.
    if (fastPanel.contains(document.activeElement)) document.activeElement.blur();
}

fastBtn.addEventListener('click', () => {
    if (fastBlocked()) return;
    if (fastOpen) closeFast();
    else openFast();
});

// A press anywhere else dismisses, on mousedown rather than on click for two reasons: the press
// that opens the panel must not also be the press that closes it, and a press that begins on the
// picture has to end this before the picture's own handlers below see it. The chip is passed over
// because its click toggles, and without this a press on the chip would close the panel here and
// then open it again in the handler above, which is a chip that cannot be shut.
document.addEventListener('mousedown', e => {
    if (!fastOpen) return;
    if (fastPanel.contains(e.target) || fastBtn.contains(e.target)) return;
    closeFast();
});

// Escape, from wherever the keyboard is. Registered here, which is earlier in this script than both
// the shortcuts panel's own keydown and the global player handler, because both of those can claim
// Escape and the global one spends it on leaving fullscreen. A panel up is a frontmost thing, and
// the shortcuts panel says the same about itself; this is answered first and consumed here so the
// panel cannot be left up with the window going fullscreen underneath it.
//
// helpOpen is refused here, and being refused here rather than left to openHelp alone is the point:
// this listener is registered first, so a panel that was somehow still open would swallow the Escape
// that the shortcuts panel consumes to close itself, and the owner would be sitting in a modal
// dialog whose way out had stopped working. That is WCAG 2.1.2 No Keyboard Trap, the same fault
// the handler it would have starved is written to prevent.
document.addEventListener('keydown', e => {
    if (!fastOpen || helpOpen || e.code !== 'Escape') return;
    e.preventDefault();
    e.stopImmediatePropagation();
    closeFast();
});

// One handler for the whole panel rather than one per control, so the stop that matters is made in
// exactly one place: the player handler at the foot of this file switches on e.code with no focus
// guard at all, so without it every keystroke aimed at this panel fires a player shortcut instead
// of reaching it. c, h, l, m, f, F11, the brackets, comma and period, the arrows, the slash: all of
// them are in reach of the hands that are here to type a number. keyup is stopped for the same
// reason and it matters more here than anywhere else in this file, because the keyup of a Space is
// what ends a hold and what toggles play/pause when there was no hold.
//
// It is on the panel and not on the field, the slider or any of the five buttons, and that is what
// keeps the rule true as controls are added to it: a keydown from any of them bubbles here, so the
// two step buttons, the slider and the four presets are covered by the same one stop as the field
// rather than each needing a copy of it. The one thing a control of its own still has to do is
// cancel the browser's own default, which is what the field's arrow handler below is for.
//
// Escape closes rather than reaching the document handler above, because this stop is unconditional
// and the key never gets that far. showUI is in here because the chrome's three seconds run from
// the last thing that happened, and a panel somebody is still typing in is not idle. That is the
// same rule as the re-check on #hud's focus in the timer below, reached the other way round: this
// block changes nothing about showUI itself.
fastPanel.addEventListener('keydown', e => {
    e.stopPropagation();
    showUI();
    if (e.code === 'Escape') {
        e.preventDefault();
        closeFast();
    }
});
fastPanel.addEventListener('keyup', e => e.stopPropagation());

// The slider applies on input rather than on change, which is the same "as it is moved" rule the
// typing path has. There is no commit step to wait for and therefore no reason for the chip and the
// thumb to disagree for the length of a drag, and change would fire late enough on some engines to
// make the thumb look stuck.
fastSlider.addEventListener('input', () => setTempRate(Number(fastSlider.value), true));

// Typing applies on input as well, which is the sharpest part of it and the reason there is no OK
// button on this panel: "2.5" applies as 2, then as 2 again for the trailing period, then as 2.5,
// and the chip reads 2x for the two intermediate states of a number that is halfway through being
// typed. That is the requirement, not a flaw: a speed that only moved on Enter would be a control
// with a save step, and a save step is what a cancel button would then need.
//
// Anything that is not a number writes nothing at all, and the empty case is caught before the
// parse rather than after it, because Number('') is 0 and 0 is a real rate here: the one rate that
// stands the picture still without pausing it. A refused value is left exactly as typed rather than
// being overwritten under the hand, and the panel puts the last good one back when it closes.
fastInput.addEventListener('input', () => {
    const raw = fastInput.value.trim();
    const v = raw === '' ? NaN : Number(raw);
    if (raw === '' || !Number.isFinite(v)) return;
    setTempRate(v, false);
});

// The two step buttons and the field's own arrow keys, which are the same gesture in four places,
// so they are one function. The grid is RATE_STEP, the bottom row's, because this is the same
// quantity on the same quarter grid and a second step size for it would be a number to keep in
// step with the other one for no reason anybody could state. One step off the grid rather than one
// added to the last value, which is stepRate's rule and for stepRate's reason: a value typed off
// the grid snaps to its nearest point and the rest of the grid is not dragged along with it, so a
// 2.33 typed by hand steps down to 2.00 and never to 2.08. setTempRate clamps, so a step off either
// end of the grid stops at the bound rather than asking the engine for a rate it refuses.
//
// Arrow keys are prevented here for the reason the bottom row's field prevents them: the player
// handler at the foot of this file takes the arrows for seeking and volume with no focus guard of
// its own, so without this a step from the keyboard would also move the picture and change the
// sound. The panel's own keydown listener stops the same key from ever reaching that handler, but
// stopping it there does not stop the engine's own caret behaviour on a text field, so the default
// is cancelled here as well.
function stepTemp(dir) {
    setTempRate((Math.round(tempRate / RATE_STEP) + dir) * RATE_STEP, true);
}
if (fastMinus) fastMinus.addEventListener('click', () => stepTemp(-1));
if (fastPlus) fastPlus.addEventListener('click', () => stepTemp(1));
fastInput.addEventListener('keydown', e => {
    if (e.code === 'Enter') { e.preventDefault(); fastInput.blur(); }
    else if (e.code === 'ArrowUp' || e.code === 'ArrowRight') { e.preventDefault(); stepTemp(1); }
    else if (e.code === 'ArrowDown' || e.code === 'ArrowLeft') { e.preventDefault(); stepTemp(-1); }
});
fastInput.addEventListener('blur', () => setTempRate(tempRate, true));
// The presets are four presses of the same function, and the markup carries each value, so there is
// no per-button code and no table to keep in step with the four buttons.
fastPresets.forEach(btn => {
    btn.addEventListener('click', () => setTempRate(Number(btn.dataset.rate), true));
});

// Entering fast mode.
function enterFast() {
    if (fastActive) return;
    closeFast();
    fastActive = true;
    fastSavedRate = rate;
    fastWasPaused = vid.paused;
    applyRate(Math.min(RATE_MAX, Math.max(RATE_MIN, tempRate)));
    if (fastWasPaused) vid.play().catch(() => {});
    hideControls();
    hideResumeWithChrome();
    document.body.classList.add('fastmode');
    fastPill.classList.add('show');
}

// Leaving fast mode. Restores persistent rate, hides pill and keeps UI hidden.
function leaveFast() {
    if (!fastActive) return;
    fastActive = false;
    applyRate(fastSavedRate);
    if (fastWasPaused) vid.pause();
    document.body.classList.remove('fastmode');
    fastPill.classList.remove('show');
    hideControls();
    hideResumeWithChrome();
}

// Pointer (mouse / touch) press & hold to fast-forward like YouTube
let pointerDown = false;
let pointerHoldTimer = null;
let pointerStartX = 0, pointerStartY = 0;
let pointerMoved = false;

function beginPointerPress(e) {
    if (e.button !== 0) return;
    if (resizing) return;
    if (drawbar.classList.contains('open')) return;
    if (textDialog.style.display !== 'none') return;
    if (fastOpen) return;
    if (e.target.closest('#topbar, #hud, #resume, #keys-overlay, .draw-btn, input, button, textarea, [role="button"]')) return;

    pointerDown = true;
    window.pointerDown = true;
    pointerMoved = false;
    pointerStartX = e.clientX;
    pointerStartY = e.clientY;

    clearTimeout(pointerHoldTimer);
    pointerHoldTimer = setTimeout(() => {
        pointerHoldTimer = null;
        enterFast();
    }, HOLD_MS);
}

function endPointerPress(e) {
    const wasDown = pointerDown;
    const wasMoved = pointerMoved;
    pointerDown = false;
    window.pointerDown = false;
    pointerMoved = false;
    clearTimeout(pointerHoldTimer);
    pointerHoldTimer = null;

    if (fastActive) {
        leaveFast();
        return;
    }
    if (!wasDown || wasMoved) return;

    // Short tap / click on the video area toggles play/pause (like YouTube)
    if (e && e.target && (e.target.id === 'stage' || e.target.id === 'v' || e.target.id === 'draw' || e.target === document.body)) {
        vid.paused ? vid.play().catch(()=>{}) : vid.pause();
    }
}

// Space press & hold.
function beginSpacePress() {
    spaceDown = true;
    window.spaceDown = true;
    clearTimeout(holdTimer);
    holdTimer = setTimeout(() => {
        holdTimer = null;
        enterFast();
    }, HOLD_MS);
}

// Space release. If fast mode was active, leaves fast mode and keeps UI hidden until mouse moves.
function endSpacePress() {
    const wasDown = spaceDown;
    spaceDown = false;
    window.spaceDown = false;
    clearTimeout(holdTimer);
    holdTimer = null;
    if (fastActive) {
        leaveFast();
        return;
    }
    if (!wasDown) return;
    vid.paused ? vid.play() : vid.pause();
}

// The keyup, on document rather than on the window or on any control, because the key can be
// released over anything at all: over the picture, over the rate field, over another application.
// A handler on a control would miss every release that landed somewhere else, and every release
// that landed somewhere else is the one this feature cannot afford to miss.
document.addEventListener('keyup', e => {
    if (e.code !== 'Space') return;
    endSpacePress();
});

setTempRate(tempRate, true);

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
/* One frame is 1/60 of a second on a 60fps file and nothing else. Every step in this app used to be
   that constant: two buttons here, two in the draw toolbar, and the comma and period keys firing
   this pair. On 24fps source a press moved two fifths of a frame and on 30fps half of one, so a
   person walking a shot down frame by frame landed between frames and marked the wrong one.

   The file's own rate is measured instead. requestVideoFrameCallback is handed every frame the
   engine presents with the media time it presented and how many frames have been presented in all,
   so the interval between two of those is the file's frame interval, whatever it was shot at. It is
   the one API WebView2 (Chromium 83 and up), WKWebView (Safari 15.4 and up) and current WebKitGTK
   all carry.

   Measurement falls back rather than failing, and the fallback is the old fixed step rather than a
   second measurement. getVideoPlaybackQuality().totalVideoFrames was the obvious second source and
   is not one: it counts the frames the element has CREATED, not the ones the engine PRESENTED, so
   dividing elapsed media time by it does not give a frame interval. The two counters drift apart
   whenever frames are decoded ahead of being shown, and on one variable-frame-rate clip the same
   file measured 20.0000ms on one poll and 15.0000ms on the next. Polled every 500ms beside a
   working requestVideoFrameCallback it was worse than that, because two measurers compete and the
   coarser one landing last overwrites the better value. So it is not used as a measurer at all,
   here or anywhere else in this file.

   Which leaves one source and one honest fallback. Where requestVideoFrameCallback is not there,
   or its callback never arrives because nothing is being presented, the step is
   FRAME_INTERVAL_FALLBACK: the worst case for this feature is the behaviour it replaced, which is
   at least right on a 60fps source, rather than a number that looks measured and is not.

   What this is not, and no comment in this file should ever say otherwise: the video element is not
   asked to seek to a frame and does not promise to land on one. currentTime is a position in
   seconds and the engine decodes from whatever frame is nearest, so this moves by the file's own
   frame interval and lands on the boundary it was aimed at. */
const FRAME_INTERVAL_FALLBACK = 1 / 60;
// The band a measurement has to land in to be used at all: below one frame in 240 or above one in
// 10 there is no source this app will ever be handed, and a division that came out at either of
// those ends is a bad sample rather than a rare frame rate.
const FRAME_INTERVAL_MIN = 1 / 240;
const FRAME_INTERVAL_MAX = 1 / 10;
// The rates files are actually recorded at, and how close a measurement has to be to one of them to
// be taken as that one exactly. The band is two per cent of the interval it is compared against,
// which at 25fps is 0.0008s: a file measuring 24.99fps lands 0.000016s from 1/25 and is read as
// exactly 1/25. Snapping is worth it because the nominal rate is the frame and the measurement is
// the engine's rounding of it, so the exact interval is the closer of the two.
const FRAME_RATES = [24, 25, 30, 50, 60, 120];
const FRAME_RATE_TOLERANCE = 0.02;

let frameInterval = FRAME_INTERVAL_FALLBACK;
let frameCallbackLast = null;

// A sample is only worth anything once it is inside the band above; outside it the value is dropped
// and the one already held stays, which is the fallback if it never got started.
function adoptFrameInterval(seconds) {
    if (!Number.isFinite(seconds) || seconds < FRAME_INTERVAL_MIN || seconds > FRAME_INTERVAL_MAX) return;
    for (const rate of FRAME_RATES) {
        const exact = 1 / rate;
        if (Math.abs(seconds - exact) <= exact * FRAME_RATE_TOLERANCE) {
            frameInterval = exact;
            return;
        }
    }
    frameInterval = seconds;
}

// What the callback carries: mediaTime is the frame's own timestamp in seconds and presentedFrames
// is how many frames have been shown, so their differences against the last callback's are the frame
// interval and nothing else. Both can arrive unusable on the first frame, which is why each is
// checked rather than the pair divided.
function sampleFrameCallback(metadata) {
    const media = metadata.mediaTime;
    const frames = metadata.presentedFrames;
    const last = frameCallbackLast;
    frameCallbackLast = { media, frames };
    if (!last || !Number.isFinite(media) || !Number.isFinite(frames)) return;
    const shown = frames - last.frames;
    const elapsed = media - last.media;
    if (shown > 0 && elapsed > 0) adoptFrameInterval(elapsed / shown);
}

// Started once and left running: the callback re-registers itself every frame it is handed and
// simply stops being called while nothing is presented, which is the same as having nothing to
// measure. Where the API is absent, or it never fires, frameInterval is never written and the step
// stays on FRAME_INTERVAL_FALLBACK — which is why there is no timer beside this one any more.
function watchFrameRate() {
    if (typeof vid.requestVideoFrameCallback === 'function') {
        const onFrame = (now, metadata) => {
            sampleFrameCallback(metadata);
            vid.requestVideoFrameCallback(onFrame);
        };
        vid.requestVideoFrameCallback(onFrame);
    }
}

// A seek is a jump in the media time the next sample is measured against, so the measurement window
// starts over after one. The interval already measured is kept rather than reset: it belongs to the
// file, not to where in the file the window happens to be standing.
function forgetFrameSamples() {
    frameCallbackLast = null;
}
vid.addEventListener('seeking', forgetFrameSamples);
vid.addEventListener('loadedmetadata', () => {
    forgetFrameSamples();
    // A new file measures its own rate, so the last one's is not carried into it.
    frameInterval = FRAME_INTERVAL_FALLBACK;
});

// One frame, in the direction asked for. The video is paused first, as it always was, so a step is
// a look at a still frame rather than a jump inside a moving picture. Both ends are clamped to the
// file the same way the two seek steps clamp, and nothing here can move past 0:00 or past the end.
function stepFrame(dir) {
    vid.pause();
    const target = vid.currentTime + dir * frameInterval;
    vid.currentTime = Math.max(0, Math.min(vid.duration || 0, target));
}

document.getElementById('bprev').onclick = () => stepFrame(-1);
document.getElementById('bnext').onclick = () => stepFrame(1);
watchFrameRate();

/* ====================== RESUME PLAYBACK POSITION ====================== */
/* The video starts at 0:00 and plays, exactly as it did before any of this existed. What is new is a
   chip over the picture offering to carry on from where this file was last left, and it is an offer
   and not a decision: the chip carries one button, so doing nothing is a complete answer on its own
   and the ten seconds are how long that answer is left to stand. Nothing about the panel touches
   playback.

   The two thresholds are refusals, and the second one is the unusual half of this feature. Under
   RESUME_MIN_SECONDS there is no position worth resuming to and a prompt offering four seconds is
   noise over the opening. Past RESUME_END_FRACTION the owner has watched the thing, and somebody
   opening it again is asking to watch it afresh: the usual "not within fifteen seconds of the end"
   rule is the wrong tool here, because it hides the last few seconds from the person who has already
   seen the ending, and 80 to 90 per cent is exactly the band the owner wants whole again.

   Nothing is decided before the metadata lands, because all three refusals compare against the
   duration and a file that has not reported one has nothing to compare with. A saved position
   beyond the duration is the third refusal and it covers a file that has been replaced or truncated
   since it was written; the window also refuses to offer one whose file is no longer the same size,
   so a name that now belongs to different video inherits nothing.

   The time in the button's label is fmt(), the formatter the two readouts on the bottom row already
   use, rather than a shorter one of its own: a number in this window is spelled one way. */
const RESUME_MIN_SECONDS = 5;
const RESUME_END_FRACTION = 0.9;
// Ten seconds, and what runs out here takes the panel away and changes nothing else. The other
// project's version of this prompt counts down in the open and lands in the same place, and the
// number is one that has been read rather than guessed: an offer left up longer than this stops
// being an offer. Ten seconds is also long enough for a person who glanced at the chip and looked
// away to read to still be within it when they came back, which is why timing out cannot be read as
// a refusal. See the note under hideResumePanel.
const RESUME_TIMEOUT_MS = 10000;
// Pauses inside half a second of the last one written say nothing new, and a held frame-step key is
// a pause every time: without this, walking along a file frame by frame would put a disk write on
// every keypress, which is the thing the no-timeupdate rule exists to avoid.
const RESUME_WRITE_EPSILON = 0.5;

const resumePanel = document.getElementById('resume');
const resumeGo    = document.getElementById('resume-go');
let resumeSaved = null;
let resumeTimer = null;
let resumeWritten = 0;
let resumeSuppressed = false;

// Takes the chip off the screen and stops its timer, and that is the entire effect: no seek, no
// pause, no resume_clear, nothing written. It is not a dismissal and nothing in this file treats it
// as one, because the panel has no gesture that means no. A person who opens a long lecture, sees
// the chip, looks away to read for thirty seconds and comes back has not turned it down, and a
// timeout that fired while they were away is not a decision they made — it is the absence of one,
// which is why the stored position stays and the video is left exactly where it was playing.
//
// It also drops the suppression flag, which is what makes this the end of the offer on every path
// that reaches it: the timer answering, the button being pressed, the picture being clicked. Each
// of those has decided the chip is finished, and an unhide afterwards must not read that as the
// chip having been taken away and put it back.
function hideResumePanel() {
    clearTimeout(resumeTimer);
    resumeTimer = null;
    resumeSuppressed = false;
    resumePanel.classList.remove('show');
}

// The chip going down with the chrome, and the one hiding of it in this file that is not the end of
// the offer. It hangs on the explicit hide and nothing else — H, and the eye button, which is a
// second toggle of its own and not a caller of that one — and deliberately not on hideControls,
// because hideControls also runs on the three second idle timer. Hiding the chip there would cut a
// ten second window down to three and the offer would be a flash rather than something to read.
// Someone who pressed H asked for a clean screen, not to be told their place was gone, so the chip
// is still theirs to have back.
//
// The timer is left running on purpose, and that is the owner's instruction rather than a detail.
// The ten seconds are counted from the moment the chip appeared, so four seconds of hidden chrome
// buys four seconds less reading time and not four more: a chip still down after the tenth second
// has expired and stays down for good. The flag is only set when the chip was actually up, so an
// offer that has not been made yet is never marked as one that was taken away.
function hideResumeWithChrome() {
    if (!resumePanel.classList.contains('show')) return;
    resumePanel.classList.remove('show');
    resumeSuppressed = true;
}

// The other half of that, on the explicit unhide and from nowhere else. A null timer is a window
// that has closed, and hideResumePanel clears the flag on every path that closes one, so this cannot
// put back a chip whose ten seconds were already up no matter how long the chrome stayed down.
//
// showUI is kept out of this path even though it runs on every mouse move and every key and is what
// brings the chrome back. Wiring it in there would raise the chip over a screen the owner had just
// cleaned because the pointer moved a centimetre, which is exactly the thing the explicit toggle
// exists to distinguish. Leaving it out is what makes the chip an answer to the toggle rather than
// an answer to activity.
function showResumeWithChrome() {
    if (!resumeSuppressed) return;
    resumeSuppressed = false;
    if (resumeTimer === null) return;
    resumePanel.classList.add('show');
}

// The one decision, and it happens again from both sides: the metadata can land before the window
// has answered and the window's answer can land before the metadata, so whichever arrives second is
// what puts the panel up. One offer per file is all there is.
function maybeOfferResume() {
    if (!resumeSaved || resumePanel.classList.contains('show')) return;
    const duration = vid.duration;
    if (!Number.isFinite(duration) || duration <= 0) return;
    if (resumeSaved < RESUME_MIN_SECONDS) return;
    if (resumeSaved > duration) return;
    if (resumeSaved > duration * RESUME_END_FRACTION) return;
    resumeGo.textContent = 'Continue from ' + fmt(resumeSaved);
    resumePanel.classList.add('show');
    resumeTimer = setTimeout(hideResumePanel, RESUME_TIMEOUT_MS);
}

// The window's answer to whether this file was left part-watched. It arrives with the video URL on
// DOMContentLoaded and is a number or null; nothing here can ask for it and nothing asks twice.
window.setResumePosition = (seconds) => {
    resumeSaved = typeof seconds === 'number' && Number.isFinite(seconds) && seconds > 0 ? seconds : null;
    maybeOfferResume();
};

// Written on pause, on close and nowhere else. timeupdate fires several times a second and writing
// on it would mean constant disk writes; a pause is a decision, and a close is the end of the file's
// turn. One thing refuses it outright, and it is a refusal of the offer rather than of the writing:
// a video on repeat is not a "where I left off" moment, and the loop toggle is off at every launch,
// so the only way to get here with one on is to turn it on. Nothing else is gated on the offer,
// because there is no gesture left that means no — with the chip gone down to one button, the only
// path in this page that clears storage is the ended handler below, and that one is the file
// finishing rather than a person declining.
function saveResumePosition() {
    if (loopEnabled) return;
    const t = vid.currentTime;
    if (!Number.isFinite(t) || t <= 0) return;
    if (Math.abs(t - resumeWritten) < RESUME_WRITE_EPSILON) return;
    resumeWritten = t;
    window.ipc.postMessage(`resume_save:${t}`);
}

resumeGo.addEventListener('click', () => {
    const target = resumeSaved;
    hideResumePanel();
    if (!target) return;
    // Playback is never stopped, so there is nothing to start again: this only moves the picture,
    // and the position stays stored because it is still where the video is.
    vid.currentTime = Math.min(Math.max(0, target), vid.duration || target);
});
// The other way the chip goes away without being answered, and it behaves exactly like the timeout:
// hide, and change nothing else. A press on the picture means this person is watching, not that they
// have been asked and said no, so it neither seeks to 0:00 nor clears what is stored, and there is no
// other gesture in this file that means no for it to be mistaken for.
// Escape is not a third way out and is not made one here. It is owned in every state, and each owner
// is the frontmost thing there is: the shortcuts panel up takes it and consumes it there, the draw
// bar up closes it, the speed field focused abandons what was typed into it, and with none of those
// up it asks the window to leave fullscreen. Not one of them is the chip, and none is the controls.
// Each answers the thing it closes, and that is what keeps them off it: the chip is an offer, and
// no key in this file is an answer to an offer.
document.getElementById('stage').addEventListener('click', hideResumePanel);
// Both stops are the same two the rate field and the text dialog already make, for the reasons they
// give. The keydown one is what keeps Space on the focused Continue from resuming and toggling
// playback at once, and what keeps an arrow key aimed at the panel from seeking; the mousedown one
// keeps a press on it from reaching the edge handler or, with the chrome hidden, the window drag.
resumePanel.addEventListener('keydown', (e) => e.stopPropagation());
resumePanel.addEventListener('mousedown', (e) => e.stopPropagation());

vid.addEventListener('loadedmetadata', maybeOfferResume);
vid.addEventListener('pause', saveResumePosition);
vid.addEventListener('ended', () => {
    // Watched to the end is not a place to come back from, so what was stored goes rather than being
    // kept at the final second: a finished file that keeps asking to be resumed is the commonest way
    // this goes wrong. A looping one has not finished at all and keeps nothing either. This is the
    // one and only path that clears storage now that there is no gesture for refusing the offer,
    // which is the right shape for it: the file finishing is a fact about the file, and it is the
    // only answer in this feature that somebody is not answering for themselves.
    if (loopEnabled) return;
    resumeWritten = 0;
    window.ipc.postMessage('resume_clear');
});

document.getElementById('bplay').onclick = () => {
    vid.paused ? vid.play() : vid.pause();
};
vid.addEventListener('play', () => {
    document.getElementById('bplay').textContent = '⏸';
    if (!fastActive && !document.body.classList.contains('fastmode') && !hudLock) {
        showUI();
    }
});
vid.addEventListener('pause', () => {
    document.getElementById('bplay').textContent = '▶';
    if (!fastActive && !document.body.classList.contains('fastmode') && !hudLock) {
        showUI();
    }
});

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
    // The temporary speed panel is taken down on the way in, for the reason the badge toggle is
    // guarded against in its own Escape handler: this overlay is modal, so a panel that was still
    // up behind it would be one Escape away from a shortcut panel that could not close.
    closeFast();
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
    // Space is a tap and a hold and it is told apart by the quarter second rather than here, for
    // the reasons the block that owns it gives at length. Nothing is written to the video on the
    // way down and nothing is drawn: the press is remembered and a timer is set, so a key that
    // comes up again inside that timer reaches a toggle identical to the one this case used to be,
    // and a key that is still down when the timer fires is handled by the callback instead. This is
    // an early return of its own rather than a case in the switch below for the same reason the
    // brackets above are: it is more than one statement.
    //
    // e.repeat is refused, and it is the whole reason this arm was ever wrong: a held key repeats
    // keydown, so the old single line toggled play/pause on every repeat, which at the standard
    // autorepeat rate is thirty times a second. That is fixed here rather than worked around,
    // because a hold is now a gesture and a gesture has exactly one beginning.
    if (e.code === 'Space') {
        e.preventDefault();
        if (e.repeat || spaceDown) return;
        beginSpacePress();
        return;
    }
    switch (e.code) {
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
// The waveform repaints on the same event and for the same reason, and lives here so both canvases
// are told about one width change in one place: it re-aggregates the same fixed 20ms buckets into
// however many columns the bar now has, which is a max over a couple of dozen floats per column.
// Its own paint re-measures the container's rectangle and the device pixel ratio on every call, so
// unlike this one it also self-corrects on a display whose scale has changed.
window.addEventListener('resize', paintWaveform);
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
        stepFrame(parseInt(btn.dataset.dir));
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
// The audio decode starts here and not before: the window pushes its batches into this page with
// an evaluate_script, and one that lands before this document has finished parsing is lost for the
// rest of the session, which is the exposure the update prompt's own pending flag works around.
// This post is the other answer to it. It also keeps the decode off the CPU through page load, so
// the first second of time-to-first-frame is spent on the picture, and it runs during parsing,
// which is after the section above has defined window.setWaveform.
if (window.ipc && window.ipc.postMessage) window.ipc.postMessage('wave_ready');
</script>
</body>
</html>
"##;

use clap::Parser;
use std::path::PathBuf;
use std::time::{Duration, Instant};
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

// One peak batch as one user event, for the same reason the three other messages that carry
// something to the page are Strings on this one channel: evaluate_script cannot be called from
// off the event loop, and this is built on the decode thread where the window and the webview do
// not exist. The prefix is what the arm below matches on and matches no existing message.
//
// The peaks go out at three decimal places and the numbers are the record, so this is the one
// place where rounding happens and it happens on the way to the wire rather than in the decode.
// It costs nothing that can be seen: a 24px canvas is 11 device pixels from the middle to a full
// column, so one pixel is 1/11 = 0.0909 of full scale and 0.001 is about eighty times finer than
// a single pixel. Base64 of bytes would be a sixth of the bytes and would put an encoding
// convention into the interface and a decode step into the page to save 147 KB delivered once.
fn wave_event(msg: &waveform::WaveMsg) -> String {
    let mut fields = serde_json::Map::new();
    match msg {
        waveform::WaveMsg::Meta {
            bucket_seconds,
            duration_seconds,
        } => {
            fields.insert("bucketSeconds".into(), (*bucket_seconds).into());
            fields.insert("durationSeconds".into(), (*duration_seconds).into());
        }
        waveform::WaveMsg::Chunk { start, peaks } => {
            let rounded: Vec<f64> = peaks
                .iter()
                .map(|p| ((*p as f64) * 1000.0).round() / 1000.0)
                .collect();
            fields.insert("start".into(), (*start).into());
            fields.insert("peaks".into(), rounded.into());
        }
        waveform::WaveMsg::Done => {
            fields.insert("done".into(), true.into());
        }
        waveform::WaveMsg::Failed => {
            fields.insert("failed".into(), true.into());
        }
    }
    let body = serde_json::to_string(&serde_json::Value::Object(fields)).unwrap_or_default();
    format!("wave:{body}")
}

// ==================== WHAT THE APP REMEMBERS ====================
// Two things, in one file: where this window was left, and where playback was left in each file
// that has been watched and not finished. One file rather than two because there is one helper
// below and both of them go through it, and a second format would be a second thing to keep
// tolerant of a half-written copy.
//
// It lives in the config directory of the user running it and nowhere else. The install directory
// is owned by the installer and can be read-only, and a file the app cannot write is a file the
// app does not keep. The path is worked out by hand rather than through a directory crate because
// three path joins are not what a dependency is for, and these three are the three the platform
// directories are: %APPDATA% on Windows, ~/Library/Application Support on macOS, and
// $XDG_CONFIG_HOME or ~/.config on Linux.
const STATE_FOLDER: &str = "Dr.Player";
const STATE_FILE: &str = "state.json";
// How many files keep a position. A player that opens a new file every few minutes would otherwise
// write this file for the rest of its life, and the oldest entries are the ones nobody is coming
// back for.
const MAX_SAVED_POSITIONS: usize = 200;

fn appdata_state_path() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        PathBuf::from(std::env::var_os("APPDATA")?)
    } else if cfg!(target_os = "macos") {
        PathBuf::from(std::env::var_os("HOME")?)
            .join("Library")
            .join("Application Support")
    } else {
        match std::env::var_os("XDG_CONFIG_HOME") {
            Some(dir) if !dir.is_empty() => PathBuf::from(dir),
            _ => PathBuf::from(std::env::var_os("HOME")?).join(".config"),
        }
    };
    Some(base.join(STATE_FOLDER).join(STATE_FILE))
}

fn state_path() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let exe_state = dir.join(STATE_FILE);
            if exe_state.exists() {
                return Some(exe_state);
            }
            if let Some(appdata_state) = appdata_state_path() {
                if appdata_state.exists() {
                    let _ = std::fs::copy(&appdata_state, &exe_state);
                }
            }
            return Some(exe_state);
        }
    }
    appdata_state_path()
}

// Logical pixels throughout, deliberately: a size or an offset remembered in physical pixels lands
// at the wrong size in the wrong place as soon as the display's scale changes, which is exactly the
// time a saved window is read.
#[derive(Clone, Copy)]
struct WindowRect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

// Read once, at launch. Every way this can go wrong answers the same thing, which is nothing
// remembered: no file yet, a file that will not open, a file that is not JSON, a file that is JSON
// but not an object, and a file that is an object with none of the numbers in it. None of them is a
// reason not to open, and none of them is anything the owner did wrong, so none of them is said.
fn read_state() -> (
    Option<WindowRect>,
    serde_json::Map<String, serde_json::Value>,
    Option<f64>,
) {
    let empty = serde_json::Map::new();
    let Some(path) = state_path() else {
        return (None, empty, None);
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return (None, empty, None);
    };
    let Ok(serde_json::Value::Object(root)) = serde_json::from_str::<serde_json::Value>(&text)
    else {
        return (None, empty, None);
    };
    let rect = root.get("window").and_then(remembered_rect);
    let positions = match root.get("positions") {
        Some(serde_json::Value::Object(map)) => map.clone(),
        _ => serde_json::Map::new(),
    };
    let temp_rate = root.get("temp_rate").and_then(serde_json::Value::as_f64);
    (rect, positions, temp_rate)
}

// All four or none: a rectangle missing a side is not a rectangle, and half of one is a position.
fn remembered_rect(value: &serde_json::Value) -> Option<WindowRect> {
    let side = |key: &str| value.get(key).and_then(serde_json::Value::as_f64);
    let (Some(w), Some(h), Some(x), Some(y)) = (side("w"), side("h"), side("x"), side("y")) else {
        return None;
    };
    if !w.is_finite() || !h.is_finite() || !x.is_finite() || !y.is_finite() {
        return None;
    }
    Some(WindowRect { x, y, w, h })
}

// The whole file, from whatever is remembered now, and nothing else in it. Every failure is silent
// on purpose and none of them can end the app: a state file the user cannot write to, a disk that
// is full, a directory that will not be created. The one thing worth doing about any of those is
// carry on as though nothing had been remembered, which is what the next launch does anyway.
fn write_state(
    rect: Option<WindowRect>,
    positions: &serde_json::Map<String, serde_json::Value>,
    temp_rate: Option<f64>,
) {
    let Some(path) = state_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        if std::fs::create_dir_all(dir).is_err() {
            return;
        }
    }
    let mut root = serde_json::Map::new();
    if let Some(rect) = rect {
        root.insert(
            "window".to_string(),
            serde_json::json!({ "x": rect.x, "y": rect.y, "w": rect.w, "h": rect.h }),
        );
    }
    if !positions.is_empty() {
        root.insert(
            "positions".to_string(),
            serde_json::Value::Object(positions.clone()),
        );
    }
    if let Some(rate) = temp_rate {
        if rate.is_finite() && (0.0625..=16.0).contains(&rate) {
            root.insert("temp_rate".to_string(), serde_json::json!(rate));
        }
    }
    let Ok(text) = serde_json::to_string_pretty(&root) else {
        return;
    };
    // Written beside itself and renamed over it. A write cut short by a kill or a full disk then
    // leaves the previous file whole rather than half of a new one, and read_state answers a half
    // file the same way it answers a missing one anyway. This is belt and braces, not the reason
    // that half a file is survivable.
    let staged = path.with_extension("json.new");
    if std::fs::write(&staged, text).is_err() {
        return;
    }
    let _ = std::fs::rename(staged, path);
}

fn unix_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

// The position this file was left at, and None for every reason it should not be offered: no entry,
// an entry written for a different file that now answers to the same path, a position that is not a
// number, and a position of zero. The size is the cheap identity a canonical path cannot give, and
// it is compared only when both sides know theirs: a file that cannot be measured is not refused for
// that, it is the one case where a stored position is still worth trusting.
fn remembered_position(
    positions: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    size: Option<u64>,
) -> Option<f64> {
    let entry = positions.get(key)?;
    if let (Some(stored), Some(current)) =
        (entry.get("size").and_then(serde_json::Value::as_u64), size)
    {
        if stored != current {
            return None;
        }
    }
    let seconds = entry.get("t").and_then(serde_json::Value::as_f64)?;
    if seconds.is_finite() && seconds > 0.0 {
        Some(seconds)
    } else {
        None
    }
}

// One file's position, and the cap that keeps the map a map. Eviction is oldest-first by the stamp
// written here, because a JSON object has no order of its own to evict in and alphabetical order is
// not the same thing as how long ago a file was watched.
fn remember_position(
    positions: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    seconds: f64,
    size: Option<u64>,
) {
    let mut entry = serde_json::Map::new();
    entry.insert("t".to_string(), serde_json::json!(seconds));
    if let Some(size) = size {
        entry.insert("size".to_string(), serde_json::json!(size));
    }
    entry.insert("at".to_string(), serde_json::json!(unix_seconds()));
    positions.insert(key.to_string(), serde_json::Value::Object(entry));
    while positions.len() > MAX_SAVED_POSITIONS {
        let oldest = positions
            .iter()
            .min_by_key(|(_, entry)| entry.get("at").and_then(serde_json::Value::as_u64))
            .map(|(key, _)| key.clone());
        match oldest {
            Some(key) => {
                positions.remove(&key);
            }
            None => break,
        }
    }
}

// The size a remembered window opens at. A window may be dragged to any size at all and a very small
// one is a deliberate substitute for picture-in-picture, so nothing in the live resize path is
// touched; this floor is checked once, on the way back in, and its only job is that a window
// restored below it would be too small to see and grab. 200 by 120 is the smallest shape that still
// holds a 48px top bar and a 41px bottom row without the two overlapping, which is the chrome this
// window has no other way to be moved or resized by.
const MIN_RESTORED_WIDTH: f64 = 200.0;
const MIN_RESTORED_HEIGHT: f64 = 120.0;
const DEFAULT_WINDOW_WIDTH: f64 = 1280.0;
const DEFAULT_WINDOW_HEIGHT: f64 = 720.0;

// The window as it is now, in logical pixels. Nothing is written while fill mode is shaping the
// window, because a shape fill mode chose is the video's shape rather than a size the owner picked,
// and nothing is written while the window is fullscreen, because then the window's size is the
// monitor's. Both refusals keep what is already on file saying what the window looked like when it
// was the owner's own, which is what the next launch has to open.
fn remember_geometry(window: &Window, fill_mode: bool) -> Option<WindowRect> {
    if fill_mode || window.fullscreen().is_some() {
        return None;
    }
    let scale = window.scale_factor();
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }
    let size = window.inner_size();
    let position = window.outer_position().ok()?;
    let rect = WindowRect {
        w: size.width as f64 / scale,
        h: size.height as f64 / scale,
        x: position.x as f64 / scale,
        y: position.y as f64 / scale,
    };
    if is_postable_size(rect.w, rect.h) {
        Some(rect)
    } else {
        None
    }
}

// Whether a rectangle lands somewhere the owner could find it: entirely inside one monitor that is
// plugged in now. tao 0.28 reports each monitor's position and size in the same physical pixels the
// window's own position is reported in and relative to the same origin, which is the top-left of the
// desktop rather than of any one screen, so the two are compared as they stand. What is compared is
// the monitor rectangle and not the work area: tao 0.28's MonitorHandle exposes no work area, and a
// window a few pixels under a taskbar is still a window that can be seen and grabbed, which is the
// whole of what this guard is for.
fn rect_fits_a_monitor(window: &Window, rect: WindowRect, scale: f64) -> bool {
    let left = rect.x * scale;
    let top = rect.y * scale;
    let right = left + rect.w * scale;
    let bottom = top + rect.h * scale;
    window.available_monitors().any(|monitor| {
        let origin = monitor.position();
        let size = monitor.size();
        let monitor_left = origin.x as f64;
        let monitor_top = origin.y as f64;
        left >= monitor_left
            && top >= monitor_top
            && right <= monitor_left + size.width as f64
            && bottom <= monitor_top + size.height as f64
    })
}

// Where the window opens. The remembered rectangle wins when it fits a monitor that is there now;
// otherwise the position is dropped and the window opens centred on the primary monitor, aligned to
// its top-left corner when it is bigger than that monitor so it cannot land half off the screen.
// Either way this is the size chosen above that is kept. A frameless window restored onto a monitor
// that is no longer connected has no title bar to go looking for, which is why some programs will
// not restore a position at all on Windows; this app keeps its position and checks it instead.
fn restore_position(window: &Window, saved: Option<WindowRect>, width: f64, height: f64) {
    let Some(saved) = saved else {
        return;
    };
    let landing = WindowRect {
        w: width,
        h: height,
        x: saved.x,
        y: saved.y,
    };
    if rect_fits_a_monitor(window, landing, window.scale_factor()) {
        window.set_outer_position(tao::dpi::LogicalPosition::new(saved.x, saved.y));
        return;
    }
    let Some(monitor) = window.primary_monitor() else {
        return;
    };
    let scale = monitor.scale_factor();
    let origin = monitor.position();
    let monitor_size = monitor.size();
    let monitor_width = monitor_size.width as f64;
    let monitor_height = monitor_size.height as f64;
    let x = if landing.w * scale <= monitor_width {
        origin.x as f64 + (monitor_width - landing.w * scale) / 2.0
    } else {
        origin.x as f64
    };
    let y = if landing.h * scale <= monitor_height {
        origin.y as f64 + (monitor_height - landing.h * scale) / 2.0
    } else {
        origin.y as f64
    };
    window.set_outer_position(tao::dpi::LogicalPosition::new(x / scale, y / scale));
}

// How long the window has to stop moving before its geometry is written. A drag and a fill-mode
// resize both arrive as one Resized and one Moved per frame of the gesture, so writing on each of
// them would put a disk write inside the drag; the event loop waits for this instead of the other
// way round. Nothing waits on the deadline and nothing in the window depends on it having fired,
// because the same geometry is written again on the way out.
const GEOMETRY_SETTLE: Duration = Duration::from_millis(500);

// The last write of the session, on both exit routes. A shape fill mode is driving, or the window is
// fullscreen, and the geometry on file is better than either of those, so what is remembered stays as
// it is; the positions are written either way because the last thing the page did was say where
// playback was, and this is the moment it is worth having said.
fn save_state_on_exit(
    window: &Window,
    rect: &mut Option<WindowRect>,
    positions: &serde_json::Map<String, serde_json::Value>,
    temp_rate: Option<f64>,
    fill_mode: bool,
) {
    if let Some(remembered) = remember_geometry(window, fill_mode) {
        *rect = Some(remembered);
    }
    write_state(*rect, positions, temp_rate);
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
    // The same clone for the same reason. The waveform's decode runs on its own thread and its
    // batches come back through this proxy, so the thread can never touch the webview and the
    // borrow checker is what proves it: nothing borrowed from main is reachable from a 'static
    // spawn site, because the window and the webview are moved into the loop closure below.
    let wave_proxy = proxy.clone();

    let app_icon = load_icon(include_bytes!("../resources/icon.ico"));

    // What was remembered, read once and then carried by the event loop below. The size it was left
    // at is used only if it passes the same is_postable_size test a posted resize passes, and is at
    // or above the floor; anything else opens at the default instead. The is_postable_size half is
    // the untrusted half: these numbers come out of a file on disk that a text editor or a bad write
    // can put anything in, and a size that is not a size has no business reaching the window. It is
    // the same rule the live resize path uses rather than a second one, and the floor is checked
    // after it rather than instead of it. Nothing in the live path is limited by either: a very small
    // window stays a thing the owner can make, because this guard runs once on the way back in and
    // never on a drag. The position is checked separately below, against the monitors connected now.
    let (mut remembered_rect, mut positions, mut temp_rate) = read_state();
    let file_size = std::fs::metadata(&path).ok().map(|meta| meta.len());
    let position_key = path.to_string_lossy().into_owned();
    let (inner_width, inner_height) = match remembered_rect {
        Some(rect)
            if is_postable_size(rect.w, rect.h)
                && rect.w >= MIN_RESTORED_WIDTH
                && rect.h >= MIN_RESTORED_HEIGHT =>
        {
            (rect.w, rect.h)
        }
        _ => (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
    };

    let window = WindowBuilder::new()
        .with_title("Dr.Player")
        .with_window_icon(app_icon)
        .with_decorations(false)
        .with_resizable(true)
        .with_inner_size(tao::dpi::LogicalSize::new(inner_width, inner_height))
        .build(&event_loop)
        .unwrap();

    restore_position(&window, remembered_rect, inner_width, inner_height);

    let safe_url = serde_json::to_string(&file_url).unwrap();
    let safe_title = serde_json::to_string(&filename).unwrap();
    // Where this file was left, if it was left anywhere worth going back to. null is the common
    // answer and the page puts no panel up for it. Serialized rather than formatted into the script
    // as a literal, so this goes through the same escaping as the URL and the title.
    let safe_resume = match remembered_position(&positions, &position_key, file_size) {
        Some(seconds) => serde_json::to_string(&seconds).unwrap_or_else(|_| "null".to_string()),
        None => "null".to_string(),
    };
    let safe_temp_rate = match temp_rate {
        Some(r) if r.is_finite() && (0.0625..=16.0).contains(&r) => r.to_string(),
        _ => "2".to_string(),
    };
    let init_script = format!(
        "window.addEventListener('DOMContentLoaded',function(){{window.loadVideo({});window.setTitle({});window.setResumePosition({});window.setStoredTempRate({});}});",
        safe_url, safe_title, safe_resume, safe_temp_rate
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

    // The settle timer is the only thing this loop ever waits on, and it is None almost always.
    let mut geometry_due: Option<Instant> = None;
    // Told by the page, because only the page knows: a fill-mode shape is the video's and not the
    // owner's, so nothing about the window's size is remembered while one is on its way.
    let mut fill_mode = false;

    event_loop.run(move |event, _, control_flow| {
        if let Some(due) = geometry_due {
            if Instant::now() >= due {
                geometry_due = None;
                if let Some(remembered) = remember_geometry(&window, fill_mode) {
                    remembered_rect = Some(remembered);
                    write_state(remembered_rect, &positions, temp_rate);
                }
            }
        }
        *control_flow = match geometry_due {
            Some(due) => ControlFlow::WaitUntil(due),
            None => ControlFlow::Wait,
        };
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    save_state_on_exit(&window, &mut remembered_rect, &positions, temp_rate, fill_mode);
                    *control_flow = ControlFlow::Exit;
                }
                // Both arrive once per frame of a gesture, so all they do is push the deadline out.
                // Nothing is written until the window has been still for GEOMETRY_SETTLE, and
                // nothing at all is written while fill mode is driving the shape.
                WindowEvent::Resized(_) | WindowEvent::Moved(_) if !fill_mode => {
                    geometry_due = Some(Instant::now() + GEOMETRY_SETTLE);
                }
                _ => {}
            },
            Event::UserEvent(msg) => {
                if let Some(payload) = msg.strip_prefix("wave:") {
                    // A batch, a Meta or a terminal event, from the decode thread. The body behind
                    // the prefix is already JSON, so it is read back as JSON and written out as
                    // JSON, escaped through serde_json exactly as the URL and the title are at
                    // startup, so a peak can never be read as anything but JSON. A body that does
                    // not parse is not handed to the page at all. The page is a const literal in
                    // this same binary and never navigates, so there is no second implementation
                    // this could disagree with.
                    if let Ok(rec) = serde_json::from_str::<serde_json::Value>(payload) {
                        let escaped = serde_json::to_string(&rec).unwrap_or_else(|_| "{}".to_string());
                        let _ = webview.evaluate_script(&format!("window.setWaveform({escaped});"));
                    }
                } else if msg == "wave_ready" {
                    // The page has parsed and window.setWaveform exists, so the decode can start.
                    // Nothing here is joined: joining would make the close button wait out a full
                    // decode of a 133 MB file, and S8 is a close within the first second, ten
                    // times over. The thread owns the canonical path and this proxy and nothing
                    // else; its file handle dies with it, and every send out of it after the
                    // loop has ended fails and is discarded, which is what makes the detach
                    // safe rather than merely convenient.
                    let path = path.clone();
                    let wave_proxy = wave_proxy.clone();
                    std::thread::spawn(move || {
                        let terminal = waveform::extract(path, |m| {
                            let _ = wave_proxy.send_event(wave_event(&m));
                        });
                        // The terminal message goes out from here rather than from inside the
                        // extraction, so a failure has one shape whichever path produced it.
                        let _ = wave_proxy.send_event(wave_event(&terminal));
                    });
                } else if msg == "close" {
                    save_state_on_exit(&window, &mut remembered_rect, &positions, temp_rate, fill_mode);
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
                } else if msg == "fill_mode:on" {
                    fill_mode = true;
                } else if msg == "fill_mode:off" {
                    fill_mode = false;
                } else if let Some(rate_str) = msg.strip_prefix("temp_rate:") {
                    if let Ok(rate_val) = rate_str.parse::<f64>() {
                        if rate_val.is_finite() && (0.0625..=16.0).contains(&rate_val) {
                            temp_rate = Some(rate_val);
                            write_state(remembered_rect, &positions, temp_rate);
                        }
                    }
                } else if let Some(seconds) = msg.strip_prefix("resume_save:") {
                    // Written the moment the page says where playback is, which is on a pause, on the
                    // close and nowhere else. A number that is not one, or is not past the start, is
                    // dropped rather than stored: it would be a position no panel could ever offer.
                    if let Ok(seconds) = seconds.parse::<f64>() {
                        if seconds.is_finite() && seconds > 0.0 {
                            remember_position(&mut positions, &position_key, seconds, file_size);
                            write_state(remembered_rect, &positions, temp_rate);
                        }
                    }
                } else if msg == "resume_clear" {
                    if positions.remove(&position_key).is_some() {
                        write_state(remembered_rect, &positions, temp_rate);
                    }
                } else if msg.starts_with("resize:") {
                    let parts: Vec<&str> = msg.split(':').collect();
                    if parts.len() == 3 {
                        if let (Ok(w), Ok(h)) = (parts[1].parse::<f64>(), parts[2].parse::<f64>()) {
                            if is_postable_size(w, h) {
                                window.set_inner_size(tao::dpi::LogicalSize::new(w, h));
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    })
}