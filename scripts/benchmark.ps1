param([int]$Seconds = 10, [string]$Executable = "$PSScriptRoot\..\release\PIMXSWAP.exe")
$ErrorActionPreference = 'Stop'
if ($Seconds -lt 2 -or $Seconds -gt 60) { throw 'Seconds must be between 2 and 60.' }
$process = Get-Process -Name PIMXSWAP -ErrorAction SilentlyContinue | Select-Object -First 1
$started = !$process
if ($started) { $process = Start-Process -FilePath $Executable -ArgumentList '--background' -WindowStyle Hidden -PassThru; Start-Sleep -Seconds 2 }
try {
    $process.Refresh(); $before = $process.TotalProcessorTime.TotalSeconds
    Start-Sleep -Seconds $Seconds
    $process.Refresh(); $cpu = ($process.TotalProcessorTime.TotalSeconds - $before) / $Seconds / [Environment]::ProcessorCount * 100
    $all = Get-CimInstance Win32_Process
    $ids = [System.Collections.Generic.HashSet[int]]::new(); [void]$ids.Add($process.Id)
    do { $added = $false; foreach ($item in $all) { if ($ids.Contains([int]$item.ParentProcessId) -and $ids.Add([int]$item.ProcessId)) { $added = $true } } } while ($added)
    $tree = @($ids | ForEach-Object { Get-Process -Id $_ -ErrorAction SilentlyContinue })
    [PSCustomObject]@{ SampleSeconds=$Seconds; NativeCpuPercent=[Math]::Round($cpu,4); NativeWorkingSetMB=[Math]::Round($process.WorkingSet64/1MB,2); NativePrivateMB=[Math]::Round($process.PrivateMemorySize64/1MB,2); ProcessCount=$tree.Count; TotalWorkingSetMB=[Math]::Round(($tree | Measure-Object WorkingSet64 -Sum).Sum/1MB,2); ProcessNames=($tree.ProcessName -join ', ') } | ConvertTo-Json
} finally { if ($started -and !$process.HasExited) { Stop-Process -Id $process.Id } }
