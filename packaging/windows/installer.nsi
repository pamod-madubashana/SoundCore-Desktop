; SoundCore-Desktop NSIS installer
; Run from repo root:
;   makensis -DVERSION=x.y.z packaging/windows/installer.nsi
; NSIS resolves relative paths against the script directory (packaging/windows/).
!ifndef VERSION
  !error "VERSION define required: makensis -DVERSION=x.y.z packaging/windows/installer.nsi"
!endif
!include "MUI2.nsh"

!define PRODUCT_NAME "SoundCore Desktop"
!define PRODUCT_NAME_NO_SPACE "SoundCore-Desktop"
!define PRODUCT_PUBLISHER "pamod-madubashana"
!define PRODUCT_UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME_NO_SPACE}"

Name "${PRODUCT_NAME}"
OutFile "SoundCore-Desktop-x64-setup.exe"
InstallDir "$PROGRAMFILES64\${PRODUCT_NAME_NO_SPACE}"
InstallDirRegKey HKLM "Software\${PRODUCT_NAME_NO_SPACE}" "InstallDir"
RequestExecutionLevel admin
Unicode True

!define MUI_ABORTWARNING
!define MUI_ICON "..\..\assets\icons\icon.ico"
!define MUI_UNICON "..\..\assets\icons\icon.ico"

!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Section "Install"
  SetOutPath "$INSTDIR"
  File "..\..\target\release\soundcore-desktop.exe"
  File "..\..\assets\icons\icon.ico"

  WriteRegStr HKLM "Software\${PRODUCT_NAME_NO_SPACE}" "InstallDir" "$INSTDIR"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "DisplayName" "${PRODUCT_NAME}"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\icon.ico"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
  WriteRegStr HKLM "${PRODUCT_UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\Uninstall.exe" /S'
  WriteRegDWORD HKLM "${PRODUCT_UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKLM "${PRODUCT_UNINSTALL_KEY}" "NoRepair" 1

  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
  CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk" "$INSTDIR\soundcore-desktop.exe"
  CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall.lnk" "$INSTDIR\Uninstall.exe"
  CreateShortCut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\soundcore-desktop.exe"
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\soundcore-desktop.exe"
  Delete "$INSTDIR\icon.ico"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall.lnk"
  RMDir "$SMPROGRAMS\${PRODUCT_NAME}"
  Delete "$DESKTOP\${PRODUCT_NAME}.lnk"

  DeleteRegKey HKLM "${PRODUCT_UNINSTALL_KEY}"
  DeleteRegKey HKLM "Software\${PRODUCT_NAME_NO_SPACE}"
SectionEnd
