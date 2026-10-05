Two notes on reading this file, because it is a record of what shipped rather than a list of changes.

How these releases are identified: every version named below is the version Cargo.toml carried at the
commit that cut it, and nothing else. This repository carries one git tag, v0.3.0, and its name never
matched the crate: Cargo.toml reads 0.1.0 at that commit and at every commit before 1.0.0. There is
no tag for v1.0.0, v2.0.0, v2.1.0 or v2.1.1, so nothing here can be checked against one, and whether
a GitHub release exists for any version in this file is not visible from this repository. The tag the
update check compares against is the tag name in GitHub's releases API response, not a tag in this
tree.

Corrections: where an entry was wrong and has since been fixed, the fix is recorded in place with the
commit that changed it, rather than made silently. Two entries have been corrected in this revision:
the v2.1.1 opening paragraph, and two sentences in v2.1.0. The v0.2.0 and v0.3.0 sections are
reconstructions, and they got there differently. v0.2.0 did have an entry here, written by 5653afa
and deleted wholesale by 75e9ab8, which replaced this file with a single v1.0.0 heading. v0.3.0
shipped with no entry at all, and none has been added since. Each section says which of the two it is
at the top.

Dr.Player v2.1.1

This is a correction release, not a feature release. No code changed: every fix below is in the
installer around the app, and none of them is a new capability. The binary and the installer were
both rebuilt rather than reused, because the version is frozen into the executable at compile time by
env!("CARGO_PKG_VERSION"): a version bump with no rebuild leaves the app asking GitHub for updates
against a number its own release has already moved past, and offering the user an update to the
version they were already running. Six behaviour changes are listed below, and two exist because
v2.1.0 shipped a claim it did not keep: the per-extension default it wrote, and the finish page that
said the association was there; a third does so if the v2.1.0 entry's claim that the how-to text file
was gone counts as one.

What this record cannot show you about that paragraph: the 2.1.0-stamped binary the first v2.1.1
installer is reported to have been cut around is not kept in this repository, and no commit, tag or
artefact here records it, so that part is a report and not something a reader can check. And the
paragraph postdates the build it describes. This entry first said the opposite, that the binary was
the same v2.1.0 build. The version above was committed a few minutes after the binary and installer in
target\ and dist-installer\ were written (c36aab3), an ordering read off those files' timestamps at
the time: both paths are gitignored, so no commit records it.

Why a fresh install was needed rather than an upgrade: v2.1.0's installer wrote the per-extension
fallback default for all four types, so installing it on a machine that already had a video player
replaced that program's fallback with Dr.Player.Video and kept no copy of what was there. On a
machine where VLC is installed, .mp4, .m4v, .mov and .webm all read VLC.mp4, VLC.m4v, VLC.mov and
VLC.webm, and v2.1.0 put Dr.Player.Video over all four. The uninstaller then deliberately left
that value naming a ProgID it had just deleted, and nothing shipped to put VLC back, so
install-then-uninstall left four file types with a fallback pointing at nothing and no error at
either end. A machine that has already run v2.1.0 has those four values overwritten and this
release does not put them back, because doing it safely means reading each extension's current
value and restoring it only while it is still ours, which is a guarded restore and is not in this
release. The four VLC values were still in place on the machine where this release was cut, untouched,
and the values VLC keeps beside them read WMP11.AssocFile.MP4 and WMP11.AssocFile.MOV. That is one
machine's registry and cannot be re-measured from this repository, so read it as a report about that
machine rather than as a fact about any machine.

- The per-extension (Default) is no longer written at all, and neither are Content Type or
  PerceivedType beside it. Windows consults that value only when no valid UserChoice exists for
  the type, so on any machine where the user has ever chosen a handler it did nothing at all; on
  the machines where it would apply, a fresh profile or a new image, it overwrote another
  program's fallback with no copy kept. OpenWithProgids, which is what actually puts Dr.Player in
  the Open with list, is unchanged and is the whole of what the section writes now
- Uninstalling removes the WebView2 profile directory, recursively, so it can finish. The installer
  removes $INSTDIR\<exe>.WebView2 by that name, which is where the profile was seen to land beside the
  exe on first run; wry picks the location by default and this repository cannot confirm it, so take
  the path itself as the installer's assumption rather than as a verified location. On the machine
  this release was cut on that directory held 193 files and about 8.5 MB, and the same figure is
  repeated in the installer's own comments, which is a second copy of one measurement rather than a
  second one. No version of the installer had ever deleted it. RMDir only removes an empty directory,
  so uninstall used to delete the exe, delete the uninstaller, delete the Add/Remove Programs entry,
  and leave the install directory standing with that profile in it and no way back
- The uninstaller now deletes the shell's association-toast values under the names that key
  actually uses, `<object>_.<ext>`, for both objects a Dr.Player registration produces: the
  Dr.Player.Video ProgID and Applications\dr-player.exe. It was deleting a value named
  Dr.Player, and that key has never held one, so the line had never removed anything
- An install directory on a network share is now refused. AllowRootDirInstall was already refusing
  the share itself, \\Server\Share, but not a subdirectory of it, and a typed
  \\Server\Share\Dr.Player would have put a binary this user does not control into the handler
  for four file types. The refusal is a .onVerifyInstDir callback, which also rejects a
  drive-relative path such as C:Programs
- Installing now deletes v2.0.0's How to open a video.txt and the two shortcuts that pointed at
  it. They were deleted by the uninstaller only, so a machine upgraded from v2.0.0 still had all
  three on disk and still had the Win+R instruction on its own Start Menu after installing v2.1.0
- The finish page no longer says the association exists. It says that if you left the file
  associations selected, Dr.Player now appears in Open with, because the section that creates it
  is one you can untick
- Two comments that were wrong have been corrected rather than removed. The uninstaller no longer
  claims the shell residue "does not put Dr.Player back in the Open with list": on a machine that
  had already used Dr.Player it does. On the machine this was observed on, .mp4 held i =
  dr-player.exe, .mov held c and .webm held d in Explorer\FileExts\<ext>\OpenWithList, alongside
  VLC, mpv, AfterFX, Premiere, Brave and Photos. Those are positional letters on a shared
  most-recently-used list, so they are a snapshot of one machine's recent use rather than a fixed fact
  about any machine, and removing one shifts the rest. They are left alone, and a user who wants one
  gone clears it from Explorer's own Open with list

Dr.Player v2.1.0

File Associations
- Dr.Player now appears in the Open with list for .mp4, .m4v, .mov and .webm, and in Settings >
  Default apps, as soon as the associations section runs. Installing it does not make it the default,
  so double-clicking a video still opens whatever already opens .mp4 until you choose Dr.Player once,
  in Windows' own UI. Four types and no more, because those are the ones the embedded engine decodes
  with no bundled codec. A type it cannot decode does not fail politely: the window opens and the
  title capsule in the top bar reads "Error loading video" in place of the file's name, and the close
  button beside it closes the window. Nothing is drawn on the stage, the OS window title stays
  Dr.Player, the window is undecorated so there is no system title bar to read, and the chrome
  auto-hides after three seconds, so the message is not left standing. That is why .avi, .wmv, .flv,
  .mpeg and .ogv are absent rather than merely untested
- The registration goes under HKCU\Software\Classes, per user and with no administrator rights,
  which is the only scope an unelevated installer can honestly claim. One version-independent
  ProgID, Dr.Player.Video, sits behind all four, so a later release re-registers the same name and
  a user who has already chosen Dr.Player keeps a working handler instead of losing one to a
  handler that appeared and disappeared
- The installer will not make itself the default and does not claim to. Windows keeps that choice
  in Explorer\FileExts\<ext>\UserChoice, which is obfuscated, hash-protected and blocked from
  writes by a filter driver, so no installer may take a file type away from the program someone
  already uses. Dr.Player sets AllowSilentDefaultTakeOver instead, which is Microsoft's way of
  saying the same thing from the other end: stay in the list, keep out of the decision. Somebody
  who already had another video player keeps it and gains Dr.Player beside it. That was true of
  the UserChoice layer and false of the layer below it, which v2.1.0 wrote by mistake: see the
  v2.1.1 section above
- Becoming the default is one choice, made once, in Windows' own UI: Settings > Default apps, or
  right-click a video, Open with, Choose another app, then dr-player.exe. The finish page offers
  to open that screen, unchecked, and Windows 10 opens the plain page where Windows 11 opens
  Dr.Player's own
- Uninstalling took back the ProgID key and the application key, whole and recursively, so nothing
  was left pointing at an exe that was gone, plus Dr.Player's own value in each of the four
  OpenWithProgids lists and under RegisteredApplications. It did not take back the per-extension
  default value it had written, which on uninstall then named a ProgID that no longer existed;
  that write, and its claim to take back exactly what it added, were both corrected in v2.1.1

Installer
- The how-to text file is gone, with the Start Menu and desktop shortcuts that pointed at it and
  the finish-page checkbox that opened it. Nothing shipped tells you to run a command line any
  more, because nothing shipped needs one: the app still takes exactly one argument, and the
  association supplies it
- The optional desktop-shortcut section went with it. It existed only to shortcut to that text
- The finish page now says where the app installed and that it is in the Open with list, and offers
  to open Default Apps rather than to start the app. The app exits with code 2 when started with
  no argument, which is why there has never been a shortcut to the exe and still is not one

Corrections to this entry, both made here rather than silently. The bullet about a type the engine
cannot decode was wrong twice. As first written (78bc73a) it said the window comes up titled "Error
loading video" and has to be closed from Task Manager; 0f70eb0 rewrote that to say the stage inside
the window says it and the close button in the top bar closes it. Neither wording was right about the
message, which goes to the title capsule in the top bar and never to the stage, and 0f70eb0 changed no
code, so the Close button that version claimed was missing was already in the build this release
shipped. The first bullet above was also wrong about double-clicking, and this entry contradicted
itself about it two bullets further down, as the installer's own finish page does: the installer
never writes UserChoice, so installing changes nothing about what double-click opens until you choose
Dr.Player yourself.

Dr.Player v2.0.0

- Nothing in this release changed but the version number: no code, no dependency and no behaviour,
  so there is nothing here that has not already been described under v1.0.0
- It exists so that the update check added in v1.0.0 has something newer to report. That check reads
  the tag name out of GitHub's releases API and compares it with the version frozen into the running
  binary, so a copy still stamped 1.0.0 compares equal to the latest tag rather than behind it, and
  equal is not newer, so the chip would never appear for anyone already sitting on 1.0.0. Nothing in
  this repository evidences a GitHub release for that version, and there is no git tag for it either,
  so the premise here is not checkable from here

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

Dr.Player v0.2.0

This release had an entry in this file and it was deleted. 5653afa wrote a v0.2.0 body here, and
75e9ab8 replaced the whole file with a single v1.0.0 heading, taking it with it; nothing since has put
it back. The version number is that deleted entry's own heading: Cargo.toml read 0.1.0 at 5653afa, as
it did at every commit before 75e9ab8, so no crate ever carried 0.2.0. What follows is reconstructed
from the deleted body and from the two commits it covered, and is what those commits say rather than
anything confirmed here: nothing in this file has been tested on macOS or on Linux.

- WKWebView crash fixes, as b638098 describes them and none of which was verified by running the app
  on either platform: cursor mutation deferred through requestAnimationFrame, .click() calls replaced
  by a direct switchTool(), window.prompt() replaced by an in-page text dialog, the wry fullscreen
  feature enabled, and with_initialization_script with a DOMContentLoaded listener in place of
  evaluate_script. All of it is still in src/main.rs, the cursor helper included, which is named for
  the crash it works around
- Error handling: window.onerror and unhandledrejection handlers in the page, and a Rust panic hook in
  main
- Testing infrastructure: a Vitest suite in ui/tests/player.test.js, which the deleted entry counted
  as 73 tests, a figure no longer reproducible from this repository because the file is gone;
  TESTING.md; 13 sample videos in TESTING_videos/; the pull request template under .github/; the
  platform notes in docs/WEBVIEW_QUIRKS.md; and the pre-commit hook in .githooks/. All of it is still
  in the tree except the suite and the package.json and vitest.config.js that ran it, deleted in
  03fcabc, which is why the Known section above says there is no test suite
- d92ec8f: only Escape leaves draw mode, and the e key that used to do it is gone. v1.0.0 records the
  same change under Keyboard; it is repeated here because this is the release it happened in

Dr.Player v0.3.0

This release shipped with no entry in this file at all, and nothing since has added one. It is the
only release here that carries a git tag, v0.3.0, and the tag never matched the crate: Cargo.toml
reads 0.1.0 at that commit. What follows is taken from the tagged commit bde0448 and from the code it
left behind, both of which are still in the tree.

- Autoplay with audio, falling back to muted. The muted and autoplay attributes came off the video
  element and the page starts playback itself: it un-mutes, asks the engine to play, and if that
  promise is rejected, which is what a browser does when it will not start audio unasked, it mutes
  and asks again. On Windows the webview is additionally built with
  --autoplay-policy=no-user-gesture-required, which is the engine's own switch for not blocking audio
  that starts unasked; how often the fallback actually fires is not something this repository shows
- The volume slider stopped being a hover reveal and is simply there: a fixed 60px slider that
  changes height on hover, in place of a zero-width track at zero opacity which appeared only when the
  pointer went over the speaker. It is still one of the controls the compact layout drops under 849px,
  which v1.0.0 describes
- No installer existed yet. installer/DrPlayer.nsi was added later, at 59d8c97
- Neither feature above is described anywhere else in this file: v1.0.0 covers the controls, the keys
  and the layout, and no later entry covers how playback starts
