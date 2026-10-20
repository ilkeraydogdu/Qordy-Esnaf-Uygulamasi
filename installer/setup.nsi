; ==============================================================================
; QORDY Esnaf POS v3.0 - Professional Windows Enterprise Installer (NSIS)
; Pofuduk Dijital Medya ve Yazılım Limited Şirketi
; ==============================================================================

Unicode true
!include "MUI2.nsh"
!include "FileFunc.nsh"

; Define Application Info
!define PRODUCT_NAME "QORDY Esnaf POS"
!define PRODUCT_VERSION "3.0.0"
!define PRODUCT_PUBLISHER "Pofuduk Dijital Medya ve Yazılım Ltd. Şti."
!define PRODUCT_WEB_SITE "https://qordy.com"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\qordy_pos_rs.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKLM"
!define PRODUCT_EXE "qordy_pos_rs.exe"

; General Settings
Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
OutFile "..\dist\installer\QORDY_Esnaf_POS_v3.0.0_Setup.exe"
InstallDir "$PROGRAMFILES64\QORDY\EsnafPOS"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
ShowInstDetails show
ShowUnInstDetails show
RequestExecutionLevel admin

; Modern UI Interface Configuration
!define MUI_ABORTWARNING
!define MUI_ICON "..\assets\images\app.ico"
!define MUI_UNICON "..\assets\images\app.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\${PRODUCT_EXE}"
!define MUI_FINISHPAGE_RUN_TEXT "QORDY Esnaf POS Uygulamasını Şimdi Başlat"
!insertmacro MUI_PAGE_FINISH

; Uninstaller Pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; Languages
!insertmacro MUI_LANGUAGE "Turkish"
!insertmacro MUI_LANGUAGE "English"

Section "MainSection" SEC01
  SetOutPath "$INSTDIR"
  SetOverwrite ifnewer

  ; Main Executable
  File "..\target\x86_64-pc-windows-gnu\release\${PRODUCT_EXE}"

  ; Assets
  SetOutPath "$INSTDIR\assets"
  File "..\assets\index.html"
  File "..\assets\customer.html"

  SetOutPath "$INSTDIR\assets\css"
  File "..\assets\css\style.css"
  File "..\assets\css\customer.css"

  SetOutPath "$INSTDIR\assets\js"
  File "..\assets\js\api.js"
  File "..\assets\js\app.js"
  File "..\assets\js\auth.js"
  File "..\assets\js\cart.js"
  File "..\assets\js\customer.js"
  File "..\assets\js\terminal.js"
  File "..\assets\js\views.js"

  SetOutPath "$INSTDIR\assets\images"
  File "..\assets\images\app.ico"
  File "..\assets\images\favicon.ico"
  File "..\assets\images\favicon.png"
  File "..\assets\images\logo.webp"
  File "..\assets\images\logo_light.webp"
  File "..\assets\images\qordy_app_icon.png"
  File "..\assets\images\pofuduk-logo.svg"

  ; Create Desktop and Start Menu Shortcuts
  SetOutPath "$INSTDIR"
  CreateDirectory "$SMPROGRAMS\QORDY"
  CreateShortcut "$SMPROGRAMS\QORDY\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\assets\images\app.ico" 0
  CreateShortcut "$SMPROGRAMS\QORDY\Kaldır.lnk" "$INSTDIR\uninst.exe"
  CreateShortcut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_EXE}" "" "$INSTDIR\assets\images\app.ico" 0
SectionEnd

Section -Post
  WriteUninstaller "$INSTDIR\uninst.exe"
  WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\${PRODUCT_EXE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\${PRODUCT_EXE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
SectionEnd

Section Uninstall
  Delete "$DESKTOP\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\QORDY\${PRODUCT_NAME}.lnk"
  Delete "$SMPROGRAMS\QORDY\Kaldır.lnk"
  RMDir "$SMPROGRAMS\QORDY"

  Delete "$INSTDIR\${PRODUCT_EXE}"
  Delete "$INSTDIR\uninst.exe"
  RMDir /r "$INSTDIR\assets"
  RMDir "$INSTDIR"

  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
  SetAutoClose true
SectionEnd
