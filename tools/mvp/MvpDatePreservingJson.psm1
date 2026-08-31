Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:MvpDatePreservingJsonHasDateKind =
    (Get-Command ConvertFrom-Json -ErrorAction Stop).Parameters.ContainsKey('DateKind')
$script:MvpDatePreservingJsonHasNoEnumerate =
    (Get-Command ConvertFrom-Json -ErrorAction Stop).Parameters.ContainsKey('NoEnumerate')

function Initialize-MvpDatePreservingJsonConverter {
    if ($null -ne ('Zircon.Tools.MvpDatePreservingJson' -as [type])) {
        return
    }

    Add-Type -TypeDefinition @'
using System;
using System.Collections;
using System.IO;
using System.Management.Automation;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace Zircon.Tools
{
    public static class MvpDatePreservingJson
    {
        public static void RestoreDateStrings(object value, string json)
        {
            using (var text = new StringReader(json))
            using (var reader = new JsonTextReader(text))
            {
                reader.DateParseHandling = DateParseHandling.None;
                RestoreToken(value, JToken.ReadFrom(reader));
            }
        }

        private static void RestoreToken(object value, JToken token)
        {
            var jsonObject = token as JObject;
            if (jsonObject != null)
            {
                var target = PSObject.AsPSObject(value);
                foreach (var property in jsonObject.Properties())
                {
                    var targetProperty = target.Properties[property.Name];
                    if (targetProperty == null)
                    {
                        throw new InvalidOperationException(
                            "Date-preserving JSON property '" + property.Name + "' is missing from the native object.");
                    }

                    var propertyValue = property.Value as JValue;
                    if (propertyValue != null)
                    {
                        RestoreValue(targetProperty, propertyValue);
                    }
                    else
                    {
                        RestoreToken(targetProperty.Value, property.Value);
                    }
                }
                return;
            }

            var jsonArray = token as JArray;
            if (jsonArray != null)
            {
                var target = value as IList;
                if (target == null || target.Count != jsonArray.Count)
                {
                    throw new InvalidOperationException(
                        "Date-preserving JSON array shape does not match the native object.");
                }

                for (var index = 0; index < jsonArray.Count; index++)
                {
                    var itemValue = jsonArray[index] as JValue;
                    if (itemValue != null)
                    {
                        if (itemValue.Type == JTokenType.String &&
                            (target[index] is DateTime || target[index] is DateTimeOffset))
                        {
                            target[index] = itemValue.Value;
                        }
                    }
                    else
                    {
                        RestoreToken(target[index], jsonArray[index]);
                    }
                }
                return;
            }

            if (token is JValue)
            {
                return;
            }

            throw new InvalidOperationException("Unsupported JSON token type.");
        }

        private static void RestoreValue(PSPropertyInfo targetProperty, JValue token)
        {
            if (token.Type != JTokenType.String || targetProperty.Value == null)
            {
                return;
            }

            if (targetProperty.Value is DateTime || targetProperty.Value is DateTimeOffset)
            {
                targetProperty.Value = token.Value;
            }
        }
    }
}
'@ `
        -ReferencedAssemblies @(
            [Newtonsoft.Json.JsonConvert].Assembly.Location,
            [psobject].Assembly.Location
        ) `
        -IgnoreWarnings `
        -WarningAction SilentlyContinue `
        -ErrorAction Stop
}

function Test-MvpDatePreservingJsonObject {
    param([AllowNull()][object]$Value)

    return $null -ne $Value -and
        $Value.GetType() -eq [System.Management.Automation.PSCustomObject]
}

function ConvertFrom-MvpDatePreservingJson {
    param(
        [Parameter(Mandatory)][string]$Json,
        [switch]$AllowObjectArray
    )

    $value = $null
    if ($script:MvpDatePreservingJsonHasDateKind) {
        $value = $Json | ConvertFrom-Json -DateKind String
    }
    elseif ($PSVersionTable.PSEdition -eq 'Desktop') {
        $value = $Json | ConvertFrom-Json
    }
    else {
        Initialize-MvpDatePreservingJsonConverter
        if ($script:MvpDatePreservingJsonHasNoEnumerate) {
            $value = $Json | ConvertFrom-Json -NoEnumerate
        }
        else {
            $value = $Json | ConvertFrom-Json
        }
        [Zircon.Tools.MvpDatePreservingJson]::RestoreDateStrings($value, $Json)
    }

    if ($value -is [Array]) {
        if (-not $AllowObjectArray) {
            throw 'Date-preserving JSON root must be one object.'
        }
        foreach ($item in $value) {
            if (-not (Test-MvpDatePreservingJsonObject -Value $item)) {
                throw 'Date-preserving JSON array root must contain only objects.'
            }
        }
    }
    elseif (-not (Test-MvpDatePreservingJsonObject -Value $value)) {
        throw 'Date-preserving JSON root must be one object.'
    }

    return ,$value
}

Export-ModuleMember -Function 'ConvertFrom-MvpDatePreservingJson'
