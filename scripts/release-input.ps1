param([string]$WindowTitle,[string]$Action='shortcut',[string]$Text='',[long]$WindowHandle=0)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -Path "$PSScriptRoot\ReleaseInput.cs"
if($WindowHandle -or $WindowTitle){
 if($WindowHandle){$window=[System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]$WindowHandle)}else{
 $windows=[System.Windows.Automation.AutomationElement]::RootElement.FindAll([System.Windows.Automation.TreeScope]::Children,[System.Windows.Automation.Condition]::TrueCondition)
 $window=$windows | Where-Object { $_.Current.Name -like "*$WindowTitle*" } | Select-Object -First 1
 }
 if(!$window){throw "Test window not found: $WindowTitle"}
 [ReleaseInput]::Focus($window.Current.NativeWindowHandle)
 Start-Sleep -Milliseconds 250
 if([ReleaseInput]::Foreground() -ne $window.Current.NativeWindowHandle){throw "Native foreground activation failed"}
}
switch($Action){
 'shortcut' {[ReleaseInput]::Keys(0x20,$true,$true,$false)}
 'search' {[ReleaseInput]::Keys(0x53,$false,$false,$true)}
 'address' {[ReleaseInput]::Keys(0x4c,$true,$false,$false);[ReleaseInput]::Text($Text);$window.Current.NativeWindowHandle}
 'addressValue' {
  $edit=$window.FindFirst([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,'Address and search bar')))
  if(!$edit){throw 'Chrome address bar was not found'}
  ([System.Windows.Automation.ValuePattern]$edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)).Current.Value
 }
}
