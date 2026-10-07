param([ValidateSet('all','Word','PowerPoint')][string]$Application='all',[ValidateRange(1,5)][int]$Rounds=1)
$ErrorActionPreference='Stop'
Add-Type -Path "$PSScriptRoot\ReleaseInput.cs"
$results=New-Object System.Collections.Generic.List[object]
function Check($app,$case,$expected,$read) {
 if([ReleaseInput]::Foreground() -ne $script:expectedWindow){throw 'Office foreground changed before sending the shortcut.'}
 [ReleaseInput]::Keys(0x20,$true,$true,$false)
 $until=[DateTime]::UtcNow.AddSeconds(8)
 do { Start-Sleep -Milliseconds 100; $actual=& $read } while ($actual -ne $expected -and [DateTime]::UtcNow -lt $until)
 $passed=$actual -eq $expected
 $results.Add([pscustomobject]@{app=$app;round=$round;case=$case;passed=$passed;expected=$expected;actual=$actual})
 Write-Output "$app / $case : $passed"
 if(!$passed){node "$PSScriptRoot\..\.tools\read-status.mjs"}
}
$word=$null;$doc=$null;$ppt=$null;$presentation=$null
if($Application -ne 'PowerPoint'){try {
 $word=New-Object -ComObject Word.Application
 $word.Visible=$true
 $doc=$word.Documents.Add()
 for($round=1;$round -le $Rounds;$round++){foreach($case in @(@{text='اثممخ';expected='hello';selected=$true;name='selected Persian'},@{text='sghl lk o,';expected='سلام من خو';selected=$true;name='selected partial sentence'},@{text='sghl lk o,';expected='سلام من خو';selected=$false;name='caret partial sentence'})) {
  $doc.Content.Text=$case.text
  $doc.Activate();[ReleaseInput]::Focus($word.ActiveWindow.Hwnd)
  $doc.Range(0,$case.text.Length).Select()
  if([ReleaseInput]::Foreground() -ne $word.ActiveWindow.Hwnd){throw 'Word did not become the foreground window; no shortcut was sent.'}
  if(!$case.selected){[ReleaseInput]::Keys(0x23,$true,$false,$false)}
  Start-Sleep -Milliseconds 800
  $script:expectedWindow=$word.ActiveWindow.Hwnd
  Check 'Word' $case.name $case.expected { $doc.Content.Text.TrimEnd([char]13,[char]10) }
 }}
} catch { $results.Add([pscustomobject]@{app='Word';case='setup';passed=$false;actual=$_.Exception.Message});Write-Output $_.Exception.Message }
finally { try{if($doc){$doc.Close(0)}}catch{}; try{if($word -and $word.Documents.Count -eq 0){$word.Quit()}}catch{} }}
if($Application -ne 'Word'){try {
 $ppt=New-Object -ComObject PowerPoint.Application
 $ppt.Visible=-1
 $presentation=$ppt.Presentations.Add(-1)
 $slide=$presentation.Slides.Add(1,12)
 $shape=$slide.Shapes.AddTextbox(1,80,100,500,200)
 for($round=1;$round -le $Rounds;$round++){foreach($case in @(@{text='اثممخ';expected='hello';selected=$true;name='selected Persian'},@{text='sghl lk o,';expected='سلام من خو';selected=$true;name='selected partial sentence'},@{text='sghl lk o,';expected='سلام من خو';selected=$false;name='caret partial sentence'})) {
  $shape.TextFrame.TextRange.Text=$case.text
  $ppt.ActiveWindow.View.GotoSlide(1)
  [ReleaseInput]::Focus($ppt.HWND)
  $shape.TextFrame.TextRange.Select()
  if(!$case.selected){[ReleaseInput]::Keys(0x23,$true,$false,$false)}
  Start-Sleep -Milliseconds 800
  $script:expectedWindow=$ppt.HWND
  Check 'PowerPoint' $case.name $case.expected { $shape.TextFrame.TextRange.Text.TrimEnd([char]13,[char]10) }
 }}
} catch { $results.Add([pscustomobject]@{app='PowerPoint';case='setup';passed=$false;actual=$_.Exception.Message});Write-Output $_.Exception.Message }
finally {try{if($presentation){$presentation.Saved=-1;$presentation.Close()}}catch{};try{if($ppt -and $ppt.Presentations.Count -eq 0){$ppt.Quit()}}catch{} }}
$results | ConvertTo-Json -Depth 4 | Set-Content -Encoding UTF8 "$PSScriptRoot\..\test-results\office-release.json"
$results | Format-Table -AutoSize
if(@($results | Where-Object {!$_.passed}).Count){exit 1}
