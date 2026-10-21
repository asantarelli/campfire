; This file is part of Campfire.
; Copyright (c) Pandula Péter 2017-2026.
; https://github.com/pandulapeter/campfire
;
; This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
; If a copy of the MPL was not distributed with this file, You can obtain one at
; https://mozilla.org/MPL/2.0/.

; The installer of the Windows desktop build: it copies the folder `createDistributable` writes (the app and the Java it
; runs on) to the user's own programs folder, with a Start menu entry, an optional shortcut on the desktop and an
; uninstaller. It needs no administrator rights. The library is never touched by it: it lives in the user's AppData,
; which an uninstall leaves for whoever wants the songs back.
;
; Built by build.ps1, which passes /DVERSION, /DSOURCE_DIR, /DICON and /DOUTFILE.

Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"

!ifndef VERSION
  !error "Falta /DVERSION"
!endif
!ifndef SOURCE_DIR
  !error "Falta /DSOURCE_DIR"
!endif
!ifndef ICON
  !error "Falta /DICON"
!endif
!ifndef OUTFILE
  !error "Falta /DOUTFILE"
!endif

!define APP_NAME "SDCancionero"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_NAME}"

Name "${APP_NAME}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\${APP_NAME}"
InstallDirRegKey HKCU "Software\${APP_NAME}" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
BrandingText "${APP_NAME} ${VERSION}"

VIProductVersion "${VERSION}.0"
VIAddVersionKey /LANG=3082 "ProductName" "${APP_NAME}"
VIAddVersionKey /LANG=3082 "FileDescription" "Instalador de ${APP_NAME}"
VIAddVersionKey /LANG=3082 "FileVersion" "${VERSION}"
VIAddVersionKey /LANG=3082 "ProductVersion" "${VERSION}"
VIAddVersionKey /LANG=3082 "LegalCopyright" "Copyright (c) Pandula Péter 2017-2026, MPL-2.0"

!define MUI_ICON "${ICON}"
!define MUI_UNICON "${ICON}"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_NAME}.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Abrir ${APP_NAME}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "Spanish"

; The uninstaller removes the folder the installer was pointed at, so it is only ever pointed at one named after the app:
; a "C:\" typed into the page would otherwise be what an uninstall empties.
Function .onVerifyInstDir
  StrLen $0 "${APP_NAME}"
  StrLen $1 "$INSTDIR"
  ${If} $1 < $0
    Abort
  ${EndIf}
  IntOp $2 $1 - $0
  StrCpy $3 "$INSTDIR" "" $2
  ${If} $3 != "${APP_NAME}"
    Abort
  ${EndIf}
FunctionEnd

Section "${APP_NAME}" SecApp
  SectionIn RO
  SetOutPath "$INSTDIR"
  File /r "${SOURCE_DIR}\*.*"
  WriteUninstaller "$INSTDIR\Desinstalar.exe"

  CreateDirectory "$SMPROGRAMS\${APP_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_NAME}.exe"
  CreateShortcut "$SMPROGRAMS\${APP_NAME}\Desinstalar ${APP_NAME}.lnk" "$INSTDIR\Desinstalar.exe"

  WriteRegStr HKCU "Software\${APP_NAME}" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${APP_NAME}.exe"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Desinstalar.exe"'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section "Acceso directo en el escritorio" SecDesktop
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_NAME}.exe"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecApp} "La aplicación y el Java que necesita. Tus canciones no se tocan nunca."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Agrega un icono de ${APP_NAME} al escritorio."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  Delete "$DESKTOP\${APP_NAME}.lnk"
  RMDir /r "$SMPROGRAMS\${APP_NAME}"
  RMDir /r "$INSTDIR"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  DeleteRegKey HKCU "Software\${APP_NAME}"
SectionEnd
