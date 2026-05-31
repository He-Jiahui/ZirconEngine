param(
  [string]$Reference = "..\..\workbench.png",
  [string]$Candidate = ".\_screenshots\workbench-1672x941-final.png",
  [string[]]$Only = @(),
  [int]$Step = 4
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$base = Split-Path -Parent $MyInvocation.MyCommand.Path
$referencePath = [IO.Path]::GetFullPath((Join-Path $base $Reference))
$candidatePath = [IO.Path]::GetFullPath((Join-Path $base $Candidate))

$regions = [ordered]@{
  "topbar" = @(0, 0, 1672, 60)
  "rail" = @(0, 60, 72, 835)
  "scene-tree" = @(72, 60, 332, 428)
  "viewport" = @(404, 60, 864, 428)
  "inspector" = @(1268, 60, 404, 489)
  "inspector-tabs" = @(1268, 60, 404, 43)
  "inspector-object" = @(1268, 103, 404, 89)
  "inspector-transform" = @(1268, 192, 404, 132)
  "inspector-renderer" = @(1268, 324, 404, 225)
  "components" = @(72, 488, 1196, 406)
  "components-controls" = @(72, 530, 1196, 210)
  "components-table" = @(72, 740, 520, 154)
  "components-alerts" = @(592, 740, 410, 154)
  "components-alert-stack" = @(592, 740, 410, 154)
  "components-tooltip" = @(1002, 740, 160, 154)
  "components-toast" = @(1120, 844, 300, 42)
  "components-right" = @(1268, 549, 404, 345)
  "statusbar" = @(0, 895, 1672, 46)
}

$ref = [System.Drawing.Bitmap]::FromFile($referencePath)
$cand = [System.Drawing.Bitmap]::FromFile($candidatePath)
$results = [System.Collections.Generic.List[object]]::new()

try {
  if ($ref.Width -ne $cand.Width -or $ref.Height -ne $cand.Height) {
    throw "Image dimensions differ: reference=$($ref.Width)x$($ref.Height), candidate=$($cand.Width)x$($cand.Height)"
  }

  foreach ($entry in $regions.GetEnumerator()) {
    if ($Only.Count -gt 0 -and $Only -notcontains $entry.Key) {
      continue
    }

    $r = $entry.Value
    $sum = 0.0
    $refR = 0.0
    $refG = 0.0
    $refB = 0.0
    $candR = 0.0
    $candG = 0.0
    $candB = 0.0
    $max = 0
    $count = 0

    for ($y = $r[1]; $y -lt ($r[1] + $r[3]); $y += $Step) {
      for ($x = $r[0]; $x -lt ($r[0] + $r[2]); $x += $Step) {
        $a = $ref.GetPixel($x, $y)
        $b = $cand.GetPixel($x, $y)
        $delta = [Math]::Abs($a.R - $b.R) + [Math]::Abs($a.G - $b.G) + [Math]::Abs($a.B - $b.B)
        $sum += $delta
        $refR += $a.R
        $refG += $a.G
        $refB += $a.B
        $candR += $b.R
        $candG += $b.G
        $candB += $b.B
        if ($delta -gt $max) { $max = $delta }
        $count++
      }
    }

    $grade = if ($entry.Key -eq "viewport") {
      if (($sum / $count) -le 12) { "ok-raster" } else { "needs-work" }
    }
    elseif (($sum / $count) -le 25) { "close" }
    elseif (($sum / $count) -le 38) { "rough" }
    else { "needs-work" }

    $null = $results.Add([pscustomobject]@{
      Region = $entry.Key
      AvgRgbDelta = [Math]::Round($sum / $count, 2)
      MaxRgbDelta = $max
      Samples = $count
      RefRgb = ("{0},{1},{2}" -f [Math]::Round($refR / $count, 1), [Math]::Round($refG / $count, 1), [Math]::Round($refB / $count, 1))
      CandidateRgb = ("{0},{1},{2}" -f [Math]::Round($candR / $count, 1), [Math]::Round($candG / $count, 1), [Math]::Round($candB / $count, 1))
      Status = $grade
    })
  }

  $results
}
finally {
  $ref.Dispose()
  $cand.Dispose()
}
