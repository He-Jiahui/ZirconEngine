$modulePath = Join-Path $PSScriptRoot '..\mvp\MvpDatePreservingJson.psm1'

Describe 'MVP date-preserving JSON authority' {
    BeforeEach {
        Import-Module $modulePath -Force -ErrorAction Stop
    }

    It 'preserves timestamp lexemes in nested PowerShell objects' {
        $value = ConvertFrom-MvpDatePreservingJson -Json @'
{
  "recorded_at_utc": "2026-08-01T00:00:00+00:00",
  "nested": { "value": 3 }
}
'@

        $value | Should BeOfType ([pscustomobject])
        $value.recorded_at_utc | Should Be '2026-08-01T00:00:00+00:00'
        $value.recorded_at_utc | Should BeOfType ([string])
        ($null -ne $value.PSObject.Properties['nested']) | Should Be $true
        ($null -ne $value.nested.PSObject.Properties['value']) | Should Be $true
    }

    It 'preserves one object array without enumerating it through the function boundary' {
        $value = ConvertFrom-MvpDatePreservingJson `
            -Json '[{"name":"first"},{"name":"second"}]' `
            -AllowObjectArray

        ($value -is [object[]]) | Should Be $true
        $value.Count | Should Be 2
        $value[0].name | Should Be 'first'
        ($null -ne $value[1].PSObject.Properties['name']) | Should Be $true
    }

    It 'rejects scalar roots and arrays outside the explicit fixture contract' {
        $scalarRejected = $false
        try { ConvertFrom-MvpDatePreservingJson -Json '42' | Out-Null } catch {
            $scalarRejected = $true
        }
        $scalarRejected | Should Be $true

        $arrayRejected = $false
        try { ConvertFrom-MvpDatePreservingJson -Json '[{"name":"first"}]' | Out-Null } catch {
            $arrayRejected = $true
        }
        $arrayRejected | Should Be $true

        $itemRejected = $false
        try { ConvertFrom-MvpDatePreservingJson -Json '[1]' -AllowObjectArray | Out-Null } catch {
            $itemRejected = $true
        }
        $itemRejected | Should Be $true
    }

    It 'uses native PSObject construction without a recursive PowerShell token walk' {
        $source = Get-Content -LiteralPath $modulePath -Raw -Encoding UTF8

        $source | Should Match 'PSObject\.AsPSObject'
        $source | Should Match 'DateParseHandling\.None'
        $source | Should Not Match 'function ConvertFrom-MvpJsonToken'
    }
}
