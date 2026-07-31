param(
    [ValidateSet('docs', 'guards', 'layering', 'structure', 'fmt', 'clippy')]
    [string[]]$Only,
    [switch]$DryRun,
    [switch]$Json
)

$arguments = @('tools/check_conventions.py')
foreach ($gate in $Only) {
    $arguments += @('--only', $gate)
}
if ($DryRun) {
    $arguments += '--dry-run'
}
if ($Json) {
    $arguments += '--json'
}

& python @arguments
exit $LASTEXITCODE
