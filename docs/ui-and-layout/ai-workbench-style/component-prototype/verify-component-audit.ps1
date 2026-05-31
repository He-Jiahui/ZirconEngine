param(
  [string]$Reference = "..\..\workbench.png",
  [string]$Candidate = ".\_screenshots\workbench-1672x941-final.png",
  [string[]]$Only = @(),
  [int]$Step = 2
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$base = Split-Path -Parent $MyInvocation.MyCommand.Path
$referencePath = [IO.Path]::GetFullPath((Join-Path $base $Reference))
$candidatePath = [IO.Path]::GetFullPath((Join-Path $base $Candidate))

$regions = [ordered]@{
  "transform-title" = @(1268, 192, 404, 36)
  "transform-position" = @(1268, 228, 404, 34)
  "transform-rotation" = @(1268, 262, 404, 34)
  "transform-scale" = @(1268, 296, 404, 28)

  "controls-buttons" = @(72, 530, 210, 210)
  "controls-buttons-title" = @(72, 530, 210, 34)
  "controls-buttons-row-1" = @(72, 564, 210, 44)
  "controls-buttons-row-2" = @(72, 608, 210, 44)
  "controls-buttons-row-3" = @(72, 642, 210, 54)
  "controls-buttons-row-4" = @(72, 696, 210, 44)
  "controls-icon-buttons" = @(282, 530, 205, 210)
  "controls-inputs" = @(487, 530, 210, 210)
  "controls-inputs-title" = @(487, 530, 210, 34)
  "controls-inputs-fields" = @(487, 564, 210, 110)
  "controls-inputs-bottom" = @(487, 674, 210, 66)
  "controls-checks" = @(697, 530, 152, 210)
  "controls-sliders" = @(849, 530, 278, 210)
  "controls-labs" = @(1127, 530, 141, 210)
  "controls-labs-title" = @(1127, 530, 141, 36)
  "controls-labs-tabs" = @(1127, 566, 141, 42)
  "controls-labs-segment-title" = @(1127, 608, 141, 34)
  "controls-labs-segment" = @(1127, 642, 141, 42)
  "controls-labs-switch" = @(1127, 684, 141, 56)

  "table-title" = @(72, 740, 520, 36)
  "table-head" = @(72, 776, 520, 30)
  "table-row-1" = @(72, 806, 520, 27)
  "table-row-2-selected" = @(72, 833, 520, 27)
  "table-row-3" = @(72, 860, 520, 34)
  "table-row-3-name" = @(72, 860, 128, 34)
  "table-row-3-type" = @(200, 860, 112, 34)
  "table-row-3-size" = @(312, 860, 106, 34)
  "table-row-3-modified" = @(418, 860, 132, 34)

  "renderer-title" = @(1268, 324, 404, 36)
  "renderer-resources" = @(1268, 360, 404, 70)
  "renderer-mesh-row" = @(1268, 360, 404, 34)
  "renderer-material-row" = @(1268, 394, 404, 36)
  "renderer-lighting" = @(1268, 430, 404, 78)
  "renderer-button" = @(1268, 508, 404, 41)

  "status-left" = @(0, 895, 560, 46)
  "status-ready" = @(0, 895, 112, 46)
  "status-no-errors" = @(112, 895, 118, 46)
  "status-warnings" = @(230, 895, 146, 46)
  "status-messages" = @(376, 895, 184, 46)
  "status-middle" = @(560, 895, 560, 46)
  "status-right" = @(1120, 895, 552, 46)

  "alerts-stack" = @(592, 740, 410, 154)
  "tooltip-area" = @(1002, 740, 160, 154)
  "toast-area" = @(1120, 844, 300, 42)
}

function Measure-Region {
  param(
    [System.Drawing.Bitmap]$ReferenceBitmap,
    [System.Drawing.Bitmap]$CandidateBitmap,
    [string]$Name,
    [int[]]$Rect,
    [int]$SampleStep
  )

  $sum = 0.0
  $max = 0
  $count = 0
  $refR = 0.0
  $refG = 0.0
  $refB = 0.0
  $candR = 0.0
  $candG = 0.0
  $candB = 0.0

  for ($y = $Rect[1]; $y -lt ($Rect[1] + $Rect[3]); $y += $SampleStep) {
    for ($x = $Rect[0]; $x -lt ($Rect[0] + $Rect[2]); $x += $SampleStep) {
      $a = $ReferenceBitmap.GetPixel($x, $y)
      $b = $CandidateBitmap.GetPixel($x, $y)
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

  [pscustomobject]@{
    Region = $Name
    AvgRgbDelta = [Math]::Round($sum / $count, 2)
    MaxRgbDelta = $max
    Samples = $count
    RefRgb = ("{0},{1},{2}" -f [Math]::Round($refR / $count, 1), [Math]::Round($refG / $count, 1), [Math]::Round($refB / $count, 1))
    CandidateRgb = ("{0},{1},{2}" -f [Math]::Round($candR / $count, 1), [Math]::Round($candG / $count, 1), [Math]::Round($candB / $count, 1))
  }
}

$ref = [System.Drawing.Bitmap]::FromFile($referencePath)
$cand = [System.Drawing.Bitmap]::FromFile($candidatePath)

try {
  if ($ref.Width -ne $cand.Width -or $ref.Height -ne $cand.Height) {
    throw "Image dimensions differ: reference=$($ref.Width)x$($ref.Height), candidate=$($cand.Width)x$($cand.Height)"
  }

  foreach ($entry in $regions.GetEnumerator()) {
    if ($Only.Count -gt 0 -and $Only -notcontains $entry.Key) {
      continue
    }

    Measure-Region -ReferenceBitmap $ref -CandidateBitmap $cand -Name $entry.Key -Rect $entry.Value -SampleStep $Step
  }
}
finally {
  $ref.Dispose()
  $cand.Dispose()
}
