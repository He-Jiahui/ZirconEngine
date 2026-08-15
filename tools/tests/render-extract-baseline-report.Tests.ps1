$testDirectory = Join-Path $PSScriptRoot 'render-extract-baseline-report'
$testFiles = @(Get-ChildItem -LiteralPath $testDirectory -File -Filter '*.Tests.ps1' | Sort-Object Name)
if ($testFiles.Count -eq 0) {
    throw "Render-extract baseline report tests are missing under $testDirectory."
}
foreach ($testFile in $testFiles) {
    . $testFile.FullName
}
