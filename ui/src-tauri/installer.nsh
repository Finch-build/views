; What the installer must do for the Engine, which it knows only as a file.

; Asks a running Engine to quit, so that its executable can be replaced or removed.
!macro StopEngine
  nsExec::Exec 'taskkill /im views.exe'
  Sleep 500
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro StopEngine
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro StopEngine
  ; The autostart toggle's entry would otherwise outlive the program it starts.
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "views"
!macroend
