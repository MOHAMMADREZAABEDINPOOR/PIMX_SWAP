param([int]$ProcessId, [string]$Executable = "$PSScriptRoot\..\release\PIMXSWAP.exe")
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$created = !$ProcessId
if ($created) { $taskApp = Start-Process -FilePath $Executable -WindowStyle Hidden -PassThru; $ProcessId = $taskApp.Id }
else { $taskApp = Get-Process -Id $ProcessId }
$settingsPath = Join-Path $env:APPDATA 'com.pimxswap.desktop\settings.json'
$existed = Test-Path -LiteralPath $settingsPath
$originalSettings = if ($existed) { [System.IO.File]::ReadAllBytes($settingsPath) } else { $null }
$needsOnboarding = !$existed -or !([System.Text.Encoding]::UTF8.GetString($originalSettings) | ConvertFrom-Json).onboarded
function Find-Control([string]$name, $type) {
    $condition = New-Object System.Windows.Automation.AndCondition(
        (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,$name)),
        (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty,$type)))
    return $script:root.FindAll([System.Windows.Automation.TreeScope]::Descendants,$condition)
}
function Click-Button([string]$name) {
    $controls = Find-Control $name ([System.Windows.Automation.ControlType]::Button)
    if ($controls.Count -eq 0) { throw "Button not found: $name" }
    $control = $controls[$controls.Count - 1]
    $pattern = $null
    if ($control.TryGetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern,[ref]$pattern)) {
        ([System.Windows.Automation.InvokePattern]$pattern).Invoke()
    } else {
        ([System.Windows.Automation.TogglePattern]$control.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern)).Toggle()
    }
    Start-Sleep -Milliseconds 300
}
function Field([string]$id) {
    $condition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::AutomationIdProperty,$id)
    $control = $null
    for ($attempt=0;$attempt -lt 50;$attempt++) {
        $control = $script:root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
        if ($control) {break}; Start-Sleep -Milliseconds 100
    }
    if (!$control) { throw "Field not found: $id" }
    return [System.Windows.Automation.ValuePattern]$control.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
}
try {
    for ($attempt=0;$attempt -lt 30;$attempt++) { $taskApp.Refresh(); if ($taskApp.MainWindowTitle -eq 'PIMXSWAP') {break}; Start-Sleep -Milliseconds 200 }
    $script:root = [System.Windows.Automation.AutomationElement]::FromHandle($taskApp.MainWindowHandle)
    for ($attempt=0;$attempt -lt 30;$attempt++) {
        $readyName = if ($needsOnboarding) {'Continue'} else {'Direct convert'}
        if ((Find-Control $readyName ([System.Windows.Automation.ControlType]::Button)).Count -gt 0) {break}; Start-Sleep -Milliseconds 200
    }
    if ((Find-Control 'Continue' ([System.Windows.Automation.ControlType]::Button)).Count -gt 0) {
        Click-Button 'Continue'; Click-Button 'Continue'; Click-Button 'Start using PIMXSWAP'
    }
    Write-Output 'Checking native conversions…'
    (Field 'input-text').SetValue('sghl'); Start-Sleep -Milliseconds 200
    Click-Button 'Auto Fix'
    for ($attempt=0;$attempt -lt 30;$attempt++) { if ((Field 'output-text').Current.Value -eq ([string][char]0x0633 + [char]0x0644 + [char]0x0627 + [char]0x0645)) {break}; Start-Sleep -Milliseconds 100 }
    if ((Field 'output-text').Current.Value -ne ([string][char]0x0633 + [char]0x0644 + [char]0x0627 + [char]0x0645)) {throw 'Native Auto Fix did not produce the expected Persian word.'}
    Click-Button 'Direct convert'; Click-Button 'Convert text'
    if ((Field 'output-text').Current.Value -ne ([string][char]0x0633 + [char]0x0644 + [char]0x0627 + [char]0x0645)) {throw 'Native direct conversion failed.'}
    Click-Button 'Swap layouts'; Click-Button 'Convert text'
    if ((Field 'output-text').Current.Value -ne 'sghl') {throw 'Native reverse conversion failed.'}
    (Field 'input-text').SetValue('hello world'); Click-Button 'Auto Fix'; Click-Button 'Auto Fix'
    if ((Field 'output-text').Current.Value -ne 'hello world') {throw 'Correct text was changed.'}
    (Field 'input-text').SetValue('xy'); Click-Button 'Auto Fix'
    $expectedPartial = [string][char]0x0637 + [char]0x063A
    if ((Field 'output-text').Current.Value -ne $expectedPartial) {throw 'Configured pair did not convert short unfinished text.'}
    Write-Output 'Checking compact workspace and restore…'
    $previousWidth = $script:root.Current.BoundingRectangle.Width
    $previousInput = (Field 'input-text').Current.Value
    $previousOutput = (Field 'output-text').Current.Value
    Click-Button 'Compact workspace'
    if ($script:root.Current.BoundingRectangle.Width -ge $previousWidth) {throw 'Compact mode did not shrink the native window.'}
    if ((Field 'input-text').Current.Value -ne $previousInput -or (Field 'output-text').Current.Value -ne $previousOutput) {throw 'Compact mode lost editor text.'}
    Click-Button 'Full workspace'
    if ([Math]::Abs($script:root.Current.BoundingRectangle.Width - $previousWidth) -gt 3) {throw 'Full workspace did not restore the native window size.'}
    Write-Output 'Checking close-to-tray and reopen…'
    ([System.Windows.Automation.WindowPattern]$script:root.GetCurrentPattern([System.Windows.Automation.WindowPattern]::Pattern)).Close()
    Start-Sleep -Seconds 1; $taskApp.Refresh()
    if ($taskApp.HasExited -or $taskApp.MainWindowTitle -eq 'PIMXSWAP') {throw 'Closing the editor did not release the editor while keeping the background instance.'}
    $second = Start-Process -FilePath $taskApp.Path -WindowStyle Hidden -PassThru
    for ($attempt=0;$attempt -lt 30;$attempt++) { $taskApp.Refresh(); if ($taskApp.MainWindowTitle -eq 'PIMXSWAP') {break}; Start-Sleep -Milliseconds 200 }
    if ($taskApp.MainWindowTitle -ne 'PIMXSWAP') {throw 'A second launch did not reopen the existing instance.'}
    Write-Output 'Native UI Automation passed: Tauri IPC, Auto Fix, direct conversion, swap, unchanged protection, unfinished text, compact mode, close-to-tray and single-instance reopen.'
} finally {
    if ($created -and !$taskApp.HasExited) { Stop-Process -Id $taskApp.Id }
    if ($created) {
        if ($existed) { [System.IO.File]::WriteAllBytes($settingsPath,$originalSettings) }
        elseif (Test-Path -LiteralPath $settingsPath) { Remove-Item -LiteralPath $settingsPath }
    }
}
