; Dr.Player installer. Installs into the current user's own profile, so no
; administrator rights are needed. No drive letter appears below: ".." is the
; repo root and every input path hangs off it.
;
; dr-player.exe takes one required argument, the video to play (src/main.rs:
; `struct Args { path: String }`). Started with no argument it exits with code 2
; before opening a window, so nothing here starts it bare: there is no
; finish-page Run button and no shortcut to the exe. The wizard, the Start Menu
; and the optional desktop shortcut point at the how-to text instead.
Unicode true

; makensis reads the script from the script's own folder, whatever directory
; the command was typed in, so ".." is the repo root either way. Anchoring on
; ${__FILEDIR__} instead does not work: with a relative script name it is
; relative to that folder again ("installer\DrPlayer.nsi" yields "installer"),
; and the doubled path does not resolve. /NOCD breaks ".." as well.
!cd ".."

!define APP_NAME     "Dr.Player"
!define APP_VERSION  "0.1.0"
!define APP_PUBLISHER "Babariya Meet"
!define APP_EXE      "dr-player.exe"
!define APP_HELP     "How to open a video.txt"
!define HELP_LINK    "How to open a video"
!define UNINSTKEY    "Software\Microsoft\Windows\CurrentVersion\Uninstall\DrPlayer"

!define MUI_ICON   "resources\icon.ico"
!define MUI_UNICON "resources\icon.ico"

!include "MUI2.nsh"

Name "${APP_NAME}"
OutFile "dist-installer\DrPlayer-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\Dr.Player"
InstallDirRegKey HKCU "${UNINSTKEY}" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma

; The Run checkbox is deliberately absent: it would start the app with no
; argument. The readme checkbox takes its place and opens a file that works.
!define MUI_FINISHPAGE_TEXT_LARGE
!define MUI_FINISHPAGE_TEXT "Installed in:$\r$\n$INSTDIR$\r$\n$\r$\n${APP_NAME} needs a video file, so nothing installed here can start it on its own. Open that folder in Explorer and drag a video onto ${APP_EXE}, or read ${APP_HELP}."
!define MUI_FINISHPAGE_SHOWREADME "$INSTDIR\${APP_HELP}"
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Show me how to open a video"

!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "Dr.Player program files (required)" SecApp
  SetOutPath "$INSTDIR"
  File "target\release\${APP_EXE}"
  File "resources\icon.ico"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  FileOpen $0 "$INSTDIR\${APP_HELP}" w
  FileWrite $0 '${APP_NAME} ${APP_VERSION}$\r$\n'
  FileWrite $0 '$\r$\n'
  FileWrite $0 '${APP_NAME} plays one video file, named on the command line. It has no file picker and no registered file type, so it cannot be started by double-clicking it.$\r$\n'
  FileWrite $0 '$\r$\n'
  FileWrite $0 'To watch a video, open this folder in Explorer and drag a video file onto ${APP_EXE}.$\r$\n'
  FileWrite $0 '$\r$\n'
  FileWrite $0 'Or press Win+R and run this, with your own video in place of the example:$\r$\n'
  FileWrite $0 '$\r$\n'
  FileWrite $0 '  "$INSTDIR\${APP_EXE}" "my-video.mp4"$\r$\n'
  FileWrite $0 '$\r$\n'
  FileWrite $0 'To uninstall: Start > ${APP_NAME} > Uninstall ${APP_NAME}, or Settings > Apps > Installed apps > ${APP_NAME}.$\r$\n'
  FileClose $0

  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortCut "$SMPROGRAMS\${APP_NAME}\${HELP_LINK}.lnk" "$INSTDIR\${APP_HELP}"
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

Section /o "Desktop shortcut to the how-to text" SecDesktop
  CreateShortCut "$DESKTOP\${APP_NAME} - ${HELP_LINK}.lnk" "$INSTDIR\${APP_HELP}"
SectionEnd

LangString DESC_APP ${LANG_ENGLISH} "${APP_EXE}, its uninstaller, a Start Menu folder and ${APP_HELP}."
LangString DESC_DESKTOP ${LANG_ENGLISH} "Opens ${APP_HELP}. The app itself is not on the desktop: it cannot be started without a video file."

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecApp} $(DESC_APP)
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} $(DESC_DESKTOP)
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  Delete "$DESKTOP\${APP_NAME} - ${HELP_LINK}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\${HELP_LINK}.lnk"
  Delete "$SMPROGRAMS\${APP_NAME}\Uninstall ${APP_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_NAME}"
  ; /REBOOTOK, because a running dr-player.exe holds its own file open
  Delete /REBOOTOK "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\${APP_HELP}"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir /REBOOTOK "$INSTDIR"
  DeleteRegKey HKCU "${UNINSTKEY}"
SectionEnd