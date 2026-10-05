; Dr.Player installer. Installs into the current user's own profile, so no
; administrator rights are needed. No drive letter appears below: ".." is the
; repo root and every input path hangs off it.
;
; dr-player.exe takes one required argument, the video to play (src/main.rs:
; `struct Args { path: String }`). Started with no argument it exits with code 2
; before opening a window, so nothing here starts it bare: there is no shortcut to
; the exe, and the finish page does not start the app either. What the finish page
; offers instead is the one place a user can pick Dr.Player as the program that
; opens their videos, which is Windows' own Default Apps page rather than anything
; in this installer. Every entry point that does start the app hands it a real
; path, which is what the open command registered below does.
Unicode true

; makensis reads the script from the script's own folder, whatever directory
; the command was typed in, so ".." is the repo root either way. Anchoring on
; ${__FILEDIR__} instead does not work: with a relative script name it is
; relative to that folder again ("installer\DrPlayer.nsi" yields "installer"),
; and the doubled path does not resolve. /NOCD breaks ".." as well.
!cd ".."

!define APP_NAME     "Dr.Player"
; The version is supplied on the command line and never written here. Cargo.toml is the single
; source of truth, so a number typed into this script is a second copy of it and a second copy is
; how this file and the crate drifted apart once already. The default below is a deliberately
; wrong sentinel rather than a plausible one, so a bare `makensis installer\DrPlayer.nsi` builds an
; installer whose DisplayVersion says it is unset instead of quietly shipping a lie. A release is
; cut with the version read out of Cargo.toml, typed here once, from the repo root:
;
;   makensis "-DAPP_VERSION=2.1.1" installer\DrPlayer.nsi
;
; The quotes are load-bearing in PowerShell, which splits an unquoted -DAPP_VERSION=2.1.0 at the
; dots and hands makensis APP_VERSION=2 and a second argument of .1.0.
;
!ifndef APP_VERSION
  !define APP_VERSION "0.0.0-UNSET"
!endif
!define APP_PUBLISHER "Babariya Meet"
!define APP_EXE      "dr-player.exe"
!define UNINSTKEY    "Software\Microsoft\Windows\CurrentVersion\Uninstall\DrPlayer"

; One ProgID behind all four extensions, and version-independent on purpose: a later release
; re-registers this same name, so a user who has already picked Dr.Player keeps a live handler
; to resolve to, instead of the shell seeing one handler vanish and another appear.
!define ASSOC_PROGID "Dr.Player.Video"

; The name registered under RegisteredApplications is ${APP_NAME}, and the finish-page link
; below reaches that same application by its URI-escaped form. Windows keys the deep-link
; argument to the registered name exactly, so the two have to move together, and both are
; written once, here: Dr%2EPlayer is Dr.Player with the dot escaped as %2E.
!define DEFAULTAPPS_LINK "ms-settings:defaultapps?registeredAppUser=Dr%2EPlayer"

!define MUI_ICON   "resources\icon.ico"
!define MUI_UNICON "resources\icon.ico"

!include "MUI2.nsh"
; WinCore.nsh and Integration.nsh are the pair NSIS's own per-user example includes to register
; an association. On the NSIS installed here (3.12) NotifyShell_AssocChanged is defined in
; Integration.nsh rather than in WinCore.nsh. It is a macro over System::Call
; 'SHELL32::SHChangeNotify' with SHCNE_ASSOCCHANGED, and the notification is not optional:
; without it the shell may go on not noticing the new association until after a reboot.
!include "WinCore.nsh"
!include "Integration.nsh"
; LogicLib is here only for the two comparisons in .onVerifyInstDir below. It ships with NSIS in
; the same Include folder as MUI2.nsh, so it is not an extra dependency on this machine or on
; anybody building from source.
!include "LogicLib.nsh"

Name "${APP_NAME}"
OutFile "dist-installer\DrPlayer-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\Dr.Player"
InstallDirRegKey HKCU "${UNINSTKEY}" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma

; The Run checkbox is not the app. Started bare dr-player.exe exits 2, so nothing here may
; offer it, and what is left for the user to do is choose Dr.Player as their handler once, which
; is Windows' own Default Apps page and no part of this installer. Windows 11 and later honour
; the registeredAppUser argument and land on Dr.Player; Windows 10 ignores the query string and
; opens the page itself, which is an acceptable degradation of a convenience link.
;
; MUI_FINISHPAGE_RUN is defined without a value because what it names here is a function, not a
; program path: the value would be handed to Exec inside a pair of quotes of MUI's own, and a
; ms-settings URI is neither an exe nor something Exec can resolve.
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_FUNCTION MUI_OpenDefaultApps
!define MUI_FINISHPAGE_RUN_TEXT "Open Default Apps, where I can make ${APP_NAME} the default video player"
; Unchecked by default, so nothing opens unless the user asks for it. MUI 2 spells this switch
; MUI_FINISHPAGE_RUN_NOTCHECKED and defaults to ticked; MUI 1's name for the same thing was
; MUI_FINISHPAGE_RUN_CHECKED. The name that exists is the one that works.
!define MUI_FINISHPAGE_RUN_NOTCHECKED

!define MUI_FINISHPAGE_TEXT_LARGE
; The registration is the optional section, the user can untick it on the components page, and this
; text cannot know which they did. So it states the consequence conditionally rather than asserting
; an association that may not be there, and the Default Apps checkbox below still opens Windows'
; own page either way: if the section was unticked, Dr.Player is not in that page's list,
; which is a harmless trip to a screen the user can back out of, and the alternative,
; page-callback machinery to make one sentence of a static string depend on a runtime flag,
; is not worth what it buys.
!define MUI_FINISHPAGE_TEXT "Installed in:$\r$\n$INSTDIR$\r$\n$\r$\nIf you left the file associations selected, ${APP_NAME} now appears in Open with for MP4, MOV and WebM files, so a video opens in it from Explorer's Open with menu or from any application that offers one.$\r$\n$\r$\nWindows does not let an installer change which program you already use, so double-clicking keeps working the way it does now until you choose ${APP_NAME} once under Settings > Default apps."

Function MUI_OpenDefaultApps
  ; ExecShell is what hands a URI to the shell's protocol handler; CreateProcess, which is what
  ; Exec uses, would look for a program called ms-settings: and find none.
  ExecShell open "${DEFAULTAPPS_LINK}"
FunctionEnd

; One supported NSIS hook for this, documented at 4.7.2.1.10, and the alternative the manual
; offers beside it, PageEx directory with DirVerify leave and GetInstDirError, is more machinery
; than one path shape is worth. Abort is what makes the directory page refuse the path and leave
; the user on the page with their own text still in the box to correct, which is the whole of
; MUI's part in this.
;
; Why the install directory is refused at all: the association written below puts this exact path
; into the command the shell runs for four file types, so an $INSTDIR that is not on a local fixed
; drive means the party controlling that location chooses which binary plays the user's videos,
; and it runs with this user's own credentials. AllowRootDirInstall is deliberately not set
; anywhere in this script, which is NSIS's safe default and which already refuses C:\ and
; \\Server\Share; what it says nothing about is \\Server\Share\Dr.Player, which the directory
; page's edit box accepts exactly as typed. That is the case closed here, together with a
; drive-relative path such as C:Programs, which is not a directory SetOutPath can create but
; which resolves against the per-drive current directory of whichever process acts on it, so
; the string written into the registry at install time and the path CreateProcess resolves at
; launch time need not be the same string.
;
; There is no call into the Win32 API here and no FileFunc.nsh, because a drive letter followed
; by a colon and a separator is the only shape accepted and two characters of the path say so.
Function .onVerifyInstDir
  ${If} $INSTDIR == ""
    Abort
  ${EndIf}
  ; Characters 2 and 3, which is where the shape of an absolute local path is decided. A UNC path
  ; has a separator there where the colon belongs; a drive-relative path such as C:Programs has
  ; its third character where the separator belongs; a relative path has neither. Reading two
  ; characters from offset 1 rather than the first two is what makes this one test cover all three.
  StrCpy $1 $INSTDIR 3
  StrCpy $2 $1 2 1
  ${If} $2 != ":\"
    Abort
  ${EndIf}
FunctionEnd

!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; Everything about one extension, in one place, so a fifth type is one more call at each end
; rather than a second copy of any of it.
;
; What this macro used to end with, the extension's own (Default) beside Content Type and
; PerceivedType, is gone, and no backup-and-restore mechanism replaces it. The reason is not
; caution. Windows consults the extension (Default) only when no valid UserChoice exists for the
; type, so on any machine where the user has ever chosen a handler for it, which is every machine
; that has another video player installed, the write changes nothing at all. The machines it would
; apply to are a fresh profile or a new image, and there it overwrites whatever program's
; fallback was already on the key and keeps no copy of it: this machine's .mp4 reads VLC.mp4,
; with a VLC.backup of WMP11.AssocFile.MP4, and Dr.Player would have put Dr.Player.Video over the
; VLC.mp4, put the VLC.mp4 back nowhere, and left four types resolving to a ProgID that the
; uninstaller has just deleted. Overwriting another program's fallback with nothing kept to put
; it back is a worse outcome than not writing the value, so the value is not written. Content
; Type and PerceivedType are the shell's own hints about what kind of file this is, and they are
; exactly as inert beside a (Default) that nothing consults as they are without it.
;
; ${mimetype} went with the writes that used it, so the calls below are one argument each again
; and there is nothing left in this file that needs a MIME type.
;
; OpenWithProgids is what actually puts Dr.Player in the Open with list and in the Open with
; dialog, so that one value is the whole of what this macro writes and the whole of the reason
; the section above is worth having.
!macro AssocRegisterExtension ext
  ; One value named by the ProgID, holding an empty string, which is the form Microsoft specifies.
  WriteRegStr HKCU "Software\Classes\${ext}\OpenWithProgids" "${ASSOC_PROGID}" ""
!macroend

!macro AssocUnregisterExtension ext
  ; Our value out of the list, our key only, with the /ifempty guard Microsoft uses: the key
  ; itself goes only while nothing else is left in it, so another program's entry survives.
  DeleteRegValue HKCU "Software\Classes\${ext}\OpenWithProgids" "${ASSOC_PROGID}"
  DeleteRegKey /ifempty HKCU "Software\Classes\${ext}\OpenWithProgids"
  ; The shell keeps a second copy of the same membership under FileExts, and leaving that one
  ; behind is the single commonest way to end up with a ghost entry in the Open with list.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\${ext}\OpenWithProgids" "${ASSOC_PROGID}"
  DeleteRegKey /ifempty HKCU "Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\${ext}\OpenWithProgids"

  ; The shell's record of the toast it shows once a program has registered for a type. The value
  ; names there are <object>_.<ext>, and a Dr.Player registration produces two objects, so both
  ; shapes are deleted: the ProgID this script registers, and the Applications\dr-player.exe key
  ; the same registration writes underneath it. Measured read-only on this machine, which holds a
  ; Dr.Player registration, the key carries Applications\dr-player.exe_.mp4, _.mov and _.webm among
  ; a thousand-odd entries belonging to other programs, and not one value named Dr.Player.Video, or
  ; Dr.Player, which is the name v2.1.0's uninstaller asked for and could never have matched. A
  ; DeleteRegValue naming a value that is not there is a no-op, so both shapes go and the cost of
  ; the one this shell never wrote is nothing.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\ApplicationAssociationToasts" "${ASSOC_PROGID}_.${ext}"
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\ApplicationAssociationToasts" "Applications\${APP_EXE}_.${ext}"

  ; Deliberately not deleted: HKCU\Software\Classes\${ext} itself, and whatever (Default) value is
  ; on it. Nothing this script writes is on that key any more, so there is no value of ours here to
  ; leave in place or to take away, and Microsoft's uninstall guidance about not removing a value
  ; another program may have taken over since has nothing left to apply to. A machine that ran
  ; v2.1.0 does still carry Dr.Player.Video on these four keys, and that is residue of the write
  ; removed above rather than something this uninstaller put there; removing it here would need the
  ; guard the research document describes, reading the current value and removing it only while it
  ; is still ours, because deleting it unguarded would delete whatever program claimed the type in
  ; the meantime. That guard is a later release with a verified backfill, not this one.
!macroend

Section "Dr.Player program files (required)" SecApp
  SetOutPath "$INSTDIR"
  File "target\release\${APP_EXE}"
  File "resources\icon.ico"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk" "$INSTDIR\Uninstall.exe"

  WriteRegStr HKCU "${UNINSTKEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "${UNINSTKEY}" "Publisher" "${APP_PUBLISHER}"
  WriteRegStr HKCU "${UNINSTKEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTKEY}" "DisplayIcon" "$INSTDIR\${APP_EXE}"
  WriteRegStr HKCU "${UNINSTKEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKCU "${UNINSTKEY}" "NoModify" "1"
  WriteRegStr HKCU "${UNINSTKEY}" "NoRepair" "1"

  ; v2.0.0 shipped a how-to text file in the install directory and two shortcuts pointing at it,
  ; one on the desktop and one in this Start Menu folder, and the text told its reader to press
  ; Win+R and type a command line. No v2.x installer writes any of the three and, until this
  ; release, no v2.x installer deleted them either, so a machine upgraded from v2.0.0 still had
  ; all three on disk after installing v2.1.0, still had a Win+R instruction on its own Start
  ; Menu telling it how to start the app by hand, and found out they were gone only when it ran
  ; the uninstaller. Deleting them here, in the section that runs on every install including an
  ; upgrade over an older one, is what actually clears them. The uninstaller below keeps its own
  ; copy of the same three lines for the copy that was installed by v2.0.0 and never upgraded past
  ; it, and deleting a file that is not there is harmless, which is why neither copy needs a test.
  Delete "$INSTDIR\How to open a video.txt"
  Delete "$DESKTOP\${APP_NAME} - How to open a video.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\How to open a video.lnk"
SectionEnd

; What the section below deliberately never writes. Each of these is a line somebody will one day
; be tempted to add, and every one of them is the wrong move:
;
;   * Anything under HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\<ext>\
;     UserChoice, Hash included. Windows blocks writes there with the UCPD.sys filter driver,
;     ignores a Hash that does not validate, and resets the key with an "An app default was
;     reset" notice when one appears to. There is no version of this that is not hostile, and
;     the claim this installer makes is that it never takes a file type away from the program
;     the user already chose. That claim is only worth making while this line stays absent.
;   * Anything under HKLM. This installer is RequestExecutionLevel user, and per-user is the
;     scope Windows requires for a per-user association regardless.
;   * OpenWithList on any extension. Microsoft's own table says "Do not use": it was for exe
;     applications before Windows XP and OpenWithProgids supersedes it, which is what is
;     written instead.
;   * ShellNew under the ProgID. Dr.Player creates no files, so it offers no New-item verb.
;   * CurVer. One installed copy and no side-by-side versions to choose between, so there is
;     nothing for it to point at.
;   * assoc and ftype. They need administrator rights this installer does not have and not
;     want, they write machine-scope keys, and neither can express OpenWithProgids,
;     SupportedTypes, Capabilities or RegisteredApplications, which are the four things that
;     actually decide anything.
;
; Being a candidate and being the default are different achievements, and Windows only ever
; grants the second one to the user. Nothing below changes what any extension opens in today.

; Selected by default, and still the user's to untick: a Section declared without the /o switch is
; selected when the components page opens, and the user can clear it there. It is not labelled
; "set as default" because it cannot be, and a checkbox with that on it would be a lie the first
; user to read it would report.
Section "Dr.Player in Open with for MP4, MOV and WebM" SecAssoc
  ; The ProgID. FriendlyTypeName is the modern entry and the (Default) on the key itself is
  ; deprecated in its favour, so both carry the same name.
  WriteRegStr HKCU "Software\Classes\${ASSOC_PROGID}" "" "Dr.Player Video"
  WriteRegStr HKCU "Software\Classes\${ASSOC_PROGID}" "FriendlyTypeName" "Dr.Player Video"

  ; The most important single line in this section. Microsoft's definition, verbatim: "Set this
  ; optional entry to signal that Windows should ignore this ProgID when determining a default
  ; handler for a public file type. Regardless of whether this value is set, the ProgID continues
  ; to appear in the OpenWith shortcut menu and dialog." Offered, never imposed, in one value.
  ; It is written with WriteRegBin and no data because the shell reads it for presence rather
  ; than for a payload. Microsoft documents the type as REG_NONE and NSIS can write that too, via
  ; WriteRegNone, as SupportedTypes below shows; this one is an empty value because that is the
  ; shape this script was written to, and both are present-with-no-data as far as the shell is
  ; concerned.
  WriteRegBin HKCU "Software\Classes\${ASSOC_PROGID}" "AllowSilentDefaultTakeOver" ""

  WriteRegStr HKCU "Software\Classes\${ASSOC_PROGID}\DefaultIcon" "" "$INSTDIR\${APP_EXE},0"

  ; Both quotes, and %1 last. This is the line that satisfies `struct Args { path: String }`:
  ; one process, exactly one path, no matter what the file is called or where it sits.
  WriteRegStr HKCU "Software\Classes\${ASSOC_PROGID}\shell\open\command" "" '"$INSTDIR\${APP_EXE}" "%1"'

  ; The four types, and only the four. Each earns its place by being decodable by the embedded
  ; engine with no bundled codec, and over-registering is not a graceful failure: a type the
  ; engine cannot open comes up as a window whose stage says "Error loading video", which closes
  ; on the close button, on exactly the files users double-click most.
  !insertmacro AssocRegisterExtension ".mp4"
  !insertmacro AssocRegisterExtension ".m4v"
  !insertmacro AssocRegisterExtension ".mov"
  !insertmacro AssocRegisterExtension ".webm"

  ; The application key. SupportedTypes is what stops Windows offering dr-player.exe for every
  ; extension on the machine; without it the exe is a candidate for all of them.
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}" "FriendlyAppName" "${APP_NAME}"
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}" "ApplicationCompany" "${APP_PUBLISHER}"
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\shell\open\command" "" '"$INSTDIR\${APP_EXE}" "%1"'
  WriteRegNone HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".mp4"
  WriteRegNone HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".m4v"
  WriteRegNone HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".mov"
  WriteRegNone HKCU "Software\Classes\Applications\${APP_EXE}\SupportedTypes" ".webm"

  ; What puts Dr.Player in Settings > Default apps. ApplicationDescription is required rather
  ; than decorative: "If ApplicationDescription is not provided, the application does not appear
  ; in UI lists of potential default programs."
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\Capabilities" "ApplicationDescription" "Plays one local video file."
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\Capabilities\FileAssociations" ".mp4"  "${ASSOC_PROGID}"
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\Capabilities\FileAssociations" ".m4v"  "${ASSOC_PROGID}"
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\Capabilities\FileAssociations" ".mov"  "${ASSOC_PROGID}"
  WriteRegStr HKCU "Software\Classes\Applications\${APP_EXE}\Capabilities\FileAssociations" ".webm" "${ASSOC_PROGID}"

  ; The registered name is the argument of the finish-page link above, which is why that link is
  ; a define next to this rather than a string typed into a checkbox label.
  WriteRegStr HKCU "Software\RegisteredApplications" "${APP_NAME}" "Software\Classes\Applications\${APP_EXE}\Capabilities"

  ${NotifyShell_AssocChanged}
SectionEnd

LangString DESC_APP ${LANG_ENGLISH} "${APP_EXE}, its uninstaller and a Start Menu folder."
LangString DESC_ASSOC ${LANG_ENGLISH} "Puts Dr.Player in the Open with list for MP4, M4V, MOV and WebM files, and in Settings > Default apps. It cannot make itself the default: Windows does not let an installer take a file type away from the program you already chose. Choosing it there is one click, and the finish page can open that screen for you."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecApp} $(DESC_APP)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecAssoc} $(DESC_ASSOC)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  ; Associations first, so that nothing still points at an exe the lines below delete. Dr.Player's
  ; own keys go whole and recursively, the ProgID and the application key, because half a
  ; registration is worse than none: it is a name in a list that launches something that is gone.
  DeleteRegKey HKCU "Software\Classes\${ASSOC_PROGID}"
  DeleteRegKey HKCU "Software\Classes\Applications\${APP_EXE}"
  DeleteRegValue HKCU "Software\RegisteredApplications" "${APP_NAME}"

  !insertmacro AssocUnregisterExtension ".mp4"
  !insertmacro AssocUnregisterExtension ".m4v"
  !insertmacro AssocUnregisterExtension ".mov"
  !insertmacro AssocUnregisterExtension ".webm"

  ; Deliberately not touched: MuiCache, AppCompatFlags\Compatibility Assistant\Store,
  ; Search\JumplistData and the per-extension OpenWithList. What v2.1.0's version of this comment
  ; said, that the residue left behind "does not put Dr.Player back in the Open with list", was
  ; wrong, and it is worth saying what is true instead of what would have been convenient.
  ;
  ; It does put Dr.Player back in the Open with list, on any machine that had already used it.
  ; Measured read-only on this one: Explorer\FileExts\.mp4|OpenWithList holds i = dr-player.exe,
  ; .mov holds c and .webm holds d, and UserChoiceLatest\ProgId on .mp4 and .mov is
  ; Applications\dr-player.exe. Choosing one of those entries after uninstall offers a program
  ; that is gone.
  ;
  ; It is left there on purpose, and the reason is not caution about MuiCache. OpenWithList's
  ; value names are single positional letters indexed by the MRUList value beside them, and that
  ; one list holds VLC, mpv, AfterFX, Premiere, Brave, Photos and Dr.Player together on this
  ; machine. Removing our letter without also rewriting MRUList shifts every other program's entry
  ; along it, so the one operation an installer must not perform here is the only one that would
  ; clear it. MuiCache, AppCompatFlags and JumplistData take a value named after our own path, so
  ; a value-level delete would be safe in principle, but nothing in this script writes a value
  ; whose name is known rather than guessed, and that is a deferral rather than an oversight.
  ;
  ; What a user does about it is one click, in Explorer's own Open with list, which is the only
  ; place those letters can be removed without shifting somebody else's.
  ${NotifyShell_AssocChanged}

  ; v2.0.0 shipped a how-to text and two shortcuts pointing at it, and the install section above
  ; now deletes all three on every install, so this copy is here for the machine that installed
  ; v2.0.0 and never upgraded. Deleting a file that is not there is harmless, which is why both
  ; copies exist and neither needs a test. Hence the file names spelled out here rather than
  ; carried as defines for files nothing writes.
  Delete "$DESKTOP\${APP_NAME} - How to open a video.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\How to open a video.lnk"

  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  ; /REBOOTOK, because a running dr-player.exe holds its own file open
  Delete /REBOOTOK "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\How to open a video.txt"
  Delete "$INSTDIR\Uninstall.exe"

  ; The WebView2 profile, which is what stopped RMDir below from ever succeeding. webview2-com
  ; creates this directory next to the binary the first time the app runs, no installer version
  ; writes it, and until this release no installer version deleted it either: 193 files and about
  ; 8.5 MB, sixty-odd directories deep, entirely Chromium runtime state and nothing of the user's
  ; own. RMDir removes an empty directory and declines silently on a full one, so v2.1.0 deleted
  ; the exe, deleted Uninstall.exe and deleted the Add/Remove Programs key, and left the install
  ; directory standing with that profile in it and no way back to removing it by hand.
  ;
  ; /r because the tree is deep and no single delete reaches it, /REBOOTOK because a running
  ; dr-player.exe holds its own Cache, Code Cache and LevelDB files open and would otherwise
  ; leave the tree standing anyway. Named as one path rather than a recursive RMDir of $INSTDIR
  ; on purpose: anything else the user has put in the install directory is theirs to keep, and the
  ; RMDir below will simply decline to remove a directory that still has anything in it, which
  ; is the honest outcome rather than deleting files nobody asked it to delete.
  RMDir /r /REBOOTOK "$INSTDIR\${APP_EXE}.WebView2"
  RMDir /REBOOTOK "$INSTDIR"
  DeleteRegKey HKCU "${UNINSTKEY}"
SectionEnd
