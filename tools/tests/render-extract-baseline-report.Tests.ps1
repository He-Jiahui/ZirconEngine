# 经 Pester 或直接入口点源所有 render-extract-baseline-report/*.Tests.ps1；目录为空即拒绝继续。
$testDirectory = Join-Path $PSScriptRoot 'render-extract-baseline-report'
$testFiles = @(Get-ChildItem -LiteralPath $testDirectory -File -Filter '*.Tests.ps1' | Sort-Object Name)
if ($testFiles.Count -eq 0) {
    throw "Render-extract baseline report tests are missing under $testDirectory."
}
foreach ($testFile in $testFiles) {
    . $testFile.FullName
}
