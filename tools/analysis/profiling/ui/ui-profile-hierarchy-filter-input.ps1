function Find-ZirconHierarchyFilterProfileTarget {
    param([object]$Geometry)

    if ($null -eq $Geometry) {
        throw 'Hierarchy filter profiling requires current UI profile geometry.'
    }
    $matches = @($Geometry.template_controls | Where-Object {
            $_.id -eq 'template.left.HierarchySearchQuery' -and
            $_.kind -eq 'template_control' -and
            $_.surface -eq 'left' -and
            $null -ne $_.frame -and
            [double]$_.frame.width -gt 0 -and
            [double]$_.frame.height -gt 0
        })
    if ($matches.Count -ne 1) {
        throw 'Hierarchy filter profiling requires exactly one visible template.left.HierarchySearchQuery control.'
    }
    return $matches[0]
}

function Get-ZirconProfileUtf16CodeUnits {
    param([AllowEmptyString()][string]$Text)

    foreach ($character in $Text.ToCharArray()) {
        [uint16][char]$character
    }
}

function Initialize-ZirconProfileUnicodeInputApi {
    if ('ZirconProfileUnicodeInputNative' -as [type]) {
        return
    }

    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;

[StructLayout(LayoutKind.Sequential)]
public struct ZirconProfileUnicodeInput
{
    public uint type;
    public ZirconProfileUnicodeInputUnion data;
}

[StructLayout(LayoutKind.Explicit)]
public struct ZirconProfileUnicodeInputUnion
{
    [FieldOffset(0)]
    public ZirconProfileUnicodeKeyInput key;
    [FieldOffset(0)]
    public ZirconProfileUnicodeMouseInput mouse;
}

[StructLayout(LayoutKind.Sequential)]
public struct ZirconProfileUnicodeKeyInput
{
    public ushort virtualKey;
    public ushort scanCode;
    public uint flags;
    public uint time;
    public IntPtr extraInfo;
}

[StructLayout(LayoutKind.Sequential)]
public struct ZirconProfileUnicodeMouseInput
{
    public int dx;
    public int dy;
    public uint mouseData;
    public uint flags;
    public uint time;
    public IntPtr extraInfo;
}

public static class ZirconProfileUnicodeInputNative
{
    private const uint InputKeyboard = 1;
    private const uint KeyEventUnicode = 0x0004;
    private const uint KeyEventKeyUp = 0x0002;
    private const ushort VirtualKeyControl = 0x0011;
    private const ushort VirtualKeyA = 0x0041;
    private const ushort VirtualKeyBackspace = 0x0008;

    public static int GetInputSize()
    {
        return Marshal.SizeOf(typeof(ZirconProfileUnicodeInput));
    }

    public static int GetTextResetInputCount()
    {
        return 6;
    }

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint SendInput(
        uint inputCount,
        [In, Out] ZirconProfileUnicodeInput[] inputs,
        int inputSize);

    public static uint SendUnicodeText(string text)
    {
        if (String.IsNullOrEmpty(text))
        {
            return 0;
        }
        var inputs = new ZirconProfileUnicodeInput[text.Length * 2];
        for (var index = 0; index < text.Length; index++)
        {
            var down = index * 2;
            inputs[down].type = InputKeyboard;
            inputs[down].data.key.scanCode = text[index];
            inputs[down].data.key.flags = KeyEventUnicode;
            inputs[down + 1].type = InputKeyboard;
            inputs[down + 1].data.key.scanCode = text[index];
            inputs[down + 1].data.key.flags = KeyEventUnicode | KeyEventKeyUp;
        }
        return SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(ZirconProfileUnicodeInput)));
    }

    public static uint ClearTextInput()
    {
        var inputs = new[]
        {
            CreateVirtualKeyInput(VirtualKeyControl, 0),
            CreateVirtualKeyInput(VirtualKeyA, 0),
            CreateVirtualKeyInput(VirtualKeyA, KeyEventKeyUp),
            CreateVirtualKeyInput(VirtualKeyControl, KeyEventKeyUp),
            CreateVirtualKeyInput(VirtualKeyBackspace, 0),
            CreateVirtualKeyInput(VirtualKeyBackspace, KeyEventKeyUp),
        };
        return SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(ZirconProfileUnicodeInput)));
    }

    private static ZirconProfileUnicodeInput CreateVirtualKeyInput(ushort virtualKey, uint flags)
    {
        var input = new ZirconProfileUnicodeInput();
        input.type = InputKeyboard;
        input.data.key.virtualKey = virtualKey;
        input.data.key.flags = flags;
        return input;
    }
}
"@
}

function Get-ZirconProfileTextResetInputCount {
    Initialize-ZirconProfileUnicodeInputApi
    return [ZirconProfileUnicodeInputNative]::GetTextResetInputCount()
}

function Clear-ZirconProfileTextInput {
    $expectedInputCount = [uint32](Get-ZirconProfileTextResetInputCount)
    $sentInputCount = [ZirconProfileUnicodeInputNative]::ClearTextInput()
    if ($sentInputCount -ne $expectedInputCount) {
        throw "Text reset sent $sentInputCount of $expectedInputCount input events."
    }
    return [int]$expectedInputCount
}

function Send-ZirconProfileUnicodeText {
    param([AllowEmptyString()][string]$Text)

    $codeUnitCount = @(Get-ZirconProfileUtf16CodeUnits -Text $Text).Count
    if ($codeUnitCount -eq 0) {
        return 0
    }
    Initialize-ZirconProfileUnicodeInputApi
    $expectedInputCount = [uint32]($codeUnitCount * 2)
    $sentInputCount = [ZirconProfileUnicodeInputNative]::SendUnicodeText($Text)
    if ($sentInputCount -ne $expectedInputCount) {
        throw "Unicode text injection sent $sentInputCount of $expectedInputCount input events."
    }
    return $codeUnitCount
}
