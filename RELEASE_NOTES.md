Dr.Player v2.0.0

- Nothing in this release changed but the version number: no code, no dependency and no behaviour,
  so there is nothing here that has not already been described under v1.0.0
- It exists so that the update check added in v1.0.0 has something newer to report. A copy still
  stamped 1.0.0 compares equal to the old tag rather than behind it, and equal is not newer, so the
  chip would never appear for anyone already sitting on 1.0.0

Dr.Player v1.0.0

Window and Controls
- H hides every on-screen control and brings them back, from anywhere outside draw mode. It is the
  bare key, not Ctrl+H: WebView2 holds back a fixed set of its own chords and never delivers them
  to the page, which is what made the Ctrl+H this once used dead
- With the chrome hidden there is no title bar left to grab, so a press on the stage drags the
  window. Edge resize still wins over it, and it never fires in draw mode
- Hiding the chrome no longer hides the mouse pointer. It used to apply `cursor: none` along with
  the chrome, leaving no way to tell where the pointer was on the video; that rule is gone and the
  edge resize cursors are back
- The eye button in the top bar pins the chrome off, and the auto-hide timer no longer brings it
  back behind your back
- The shortcuts panel owns the keyboard while it is open, so nothing else fires behind it, and a
  Tab can no longer land in a control at opacity 0
- Hide and fill are inline SVG rather than the characters they were, and each swaps two variants so
  the state is in the icon and not only in the behaviour
- A help button opens the shortcuts panel

Fill Mode
- C, or the fill button, reshapes the window to the video's own shape, so the picture covers the
  window edge to edge with no bars, nothing cropped and nothing stretched. It changes the window,
  never the picture
- It is shape-aware. A resize drag corrects the axis the drag is not driving, so a gesture ends on
  the video's ratio whichever way it went; a corner follows the axis that moved less, so the axis
  the cursor led is the one kept
- When the window is reshaped from outside, fill mode gives way rather than arguing: no corrective
  resize is posted, the window is left where Windows put it, and the button stops claiming a mode
  that is no longer happening, so the picture goes back to fitting inside the window with the bars
  that fall

Window Sizing and Small Windows
- Window sizing is no longer clamped. A resize is refused only when a dimension is not a finite
  number of at least 1, so the window goes where the drag takes it
- A compact layout, because a window with no minimum size can be squeezed on both axes. Under 849px
  wide the buttons on the right of the bottom row go whole rather than half, keeping play/pause,
  both time readouts and the whole track, so a mouse-only viewer can still pause and scrub; what
  goes is on the keyboard anyway. Under 600px the title capsule, the update chip, hide, fill and
  minimize go, leaving the help button, the pen, the cross and the drag handle. Under 328px the
  digits drop to 10px so the track stays worth pressing. The bottom row is never hidden for height,
  because it is the only way to pause and scrub; a window shorter than 89px loses the title capsule
  instead, so the drag band stops lying over the row
- The rate field is fixed at six characters, so the group is the same width holding 0.0625 as
  holding 16.00 and no value a keyboard can produce widens the row

Bottom Chrome
- Flat chrome with no blur: the bottom bar is opaque and its chips carry no glass, so it does not
  wash the video out
- The timeline moved to a full-width strip with the time readouts bracketing it, and play/pause
  sits immediately beside it, so the control you reach for most is next to the thing it moves

Playback Speed
- Playback speed is a typed field rather than a row of stops, across the engine's own 0.0625 to 16
  range rather than a round subset of it. Typing clamps or refuses on Enter and on blur, and the
  field never ends up holding a number that is not playing
- [ and ] step the rate, the field's arrows step it, PageUp and PageDown step it a page, and Home
  and End are left to the caret. Every key inside the field stops propagating first, so typing
  0.0625 does not fire a frame step on the period in it

Keyboard
- A shortcuts panel on / or the help button, listing every binding the player and the draw bar
  actually have
- The player handler switches on e.code rather than e.key, so the bindings are the same on a
  non-US layout
- Escape leaves draw mode. The E key that used to do it is gone
- Backspace clears every shape on its own, and stays undoable. The Ctrl+Backspace it replaced was
  correct and unreachable, because WebView2 never delivers that chord to the page

Installer
- An NSIS installer for Windows, installing into the current user's own profile so no
  administrator rights are needed. Nothing installed can start the app on its own, because it needs
  a video file named on the command line, so the wizard points at a how-to text instead of a Run
  button
- The installer's version is passed in on the command line from Cargo.toml rather than written into
  the script, which is how the crate and the installer drifted apart before

Update Check
- One HTTPS GET per launch asks GitHub for the latest release. If it is newer, a chip appears in
  the top bar and its button opens the releases page in the default browser, so the video never
  leaves the window
- A copy bundled inside PDEA skips the request entirely rather than promising an update it cannot
  deliver, decided by a sibling app.asar so PDEA has to change nothing
- Every failure answers no. The cost of being wrong about a standalone copy is one prompt about a
  release that does exist; the cost of the other way round is a bundled copy that has gone quiet
- DRPLAYER_EMBEDDED forces the answer either way without a rebuild

Known
- No automated test suite. The earlier Vitest/jsdom suite was deleted; coverage is manual, through
  TESTING.md and the sample videos in TESTING_videos/
- The text tool is unreachable: no toolbar button and no key selects it, so text annotations cannot
  be created from the UI
