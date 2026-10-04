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
;   makensis "-DAPP_VERSION=2.1.0" installer\DrPlayer.nsi
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
!define MUI_FINISHPAGE_TEXT "Installed in:$\r$\n$INSTDIR$\r$\n$\r$\n${APP_NAME} now appears in Open with for MP4, MOV and WebM files, so a video opens in it from Explorer's Open with menu or from any application that offers one.$\r$\n$\r$\nWindows does not let an installer change which program you already use, so double-clicking keeps working the way it does now until you choose ${APP_NAME} once under Settings > Default apps."

Function MUI_OpenDefaultApps
  ; ExecShell is what hands a URI to the shell's protocol handler; CreateProcess, which is what
  ; Exec uses, would look for a program called ms-settings: and find none.
  ExecShell open "${DEFAULTAPPS_LINK}"
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
; The (Default) value here is a fallback and not the default: Windows 10 and 11 consult it only
; when no valid UserChoice exists for the type, so on any machine where the user has ever
; chosen a handler it changes nothing at all. That is also why it is left in place on uninstall.
; Its value is the machines where it does apply, a fresh profile or a new image, where nothing
; else would resolve the type to Dr.Player. Content Type and PerceivedType are the shell's own
; hints about what kind of file this is, and are exactly as inert beside it as it is.
!macro AssocRegisterExtension ext mimetype
  ; OpenWithProgids is the list that puts Dr.Player in the Open with menu and in the Open with
  ; dialog: one value named by the ProgID, holding an empty string, which is the form Microsoft
  ; specifies. This line, not the one below it, is the whole reason the section is worth having.
  WriteRegStr HKCU "Software\Classes\${ext}\OpenWithProgids" "${ASSOC_PROGID}" ""
  WriteRegStr HKCU "Software\Classes\${ext}" "" "${ASSOC_PROGID}"
  WriteRegStr HKCU "Software\Classes\${ext}" "Content Type" "${mimetype}"
  WriteRegStr HKCU "Software\Classes\${ext}" "PerceivedType" "video"
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

  ; Deliberately not deleted: HKCU\Software\Classes\${ext} itself, and the (Default) value on
  ; it, written above. Microsoft's uninstall guidance is explicit that an application which took
  ; ownership of a file type should leave that value in place, because it cannot tell whether
  ; another program has overwritten it since, and removing it would hand the type to whatever
  ; the shell falls back to instead.
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
  ; engine cannot open comes up as a black window titled "Error loading video" on exactly the
  ; files users double-click most.
  !insertmacro AssocRegisterExtension ".mp4"  "video/mp4"
  !insertmacro AssocRegisterExtension ".m4v"  "video/x-m4v"
  !insertmacro AssocRegisterExtension ".mov"  "video/quicktime"
  !insertmacro AssocRegisterExtension ".webm" "video/webm"

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

  ; The shell's record of the toast it shows once a program has registered for a type. One
  ; value, under our own name.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\ApplicationAssociationToasts" "${APP_NAME}"

  !insertmacro AssocUnregisterExtension ".mp4"
  !insertmacro AssocUnregisterExtension ".m4v"
  !insertmacro AssocUnregisterExtension ".mov"
  !insertmacro AssocUnregisterExtension ".webm"

  ; Deliberately not touched: MuiCache, AppCompatFlags\Compatibility Assistant\Store and
  ; Search\JumplistData. Each is a shared shell blob holding an entry for every program that has
  ; run on this machine, ours among all the others, so the only correct operation there is a
  ; value-level delete of our own name, and the cost of getting it wrong is another program's
  ; entry. What is left behind is cosmetic and does not put Dr.Player back in the Open with
  ; list, which is decided by the OpenWithProgids values removed above. This is a deferral, not
  ; an oversight: those three value deletes are a later release, once the residue has been seen
  ; on a real machine and the value names written there are known rather than guessed.
  ${NotifyShell_AssocChanged}

  ; v2.0.0 shipped a how-to text and two shortcuts pointing at it. Nothing installs either of
  ; them now and neither name is a define above any more, but a copy installed by 2.0.0 still
  ; has all three, and this uninstaller is the only thing that will ever clean them up. Hence
  ; the file name spelled out here instead of carried as a define for a file nothing writes.
  Delete "$DESKTOP\${APP_NAME} - How to open a video.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\How to open a video.lnk"

  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  ; /REBOOTOK, because a running dr-player.exe holds its own file open
  Delete /REBOOTOK "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\How to open a video.txt"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir /REBOOTOK "$INSTDIR"
  DeleteRegKey HKCU "${UNINSTKEY}"
SectionEnd
