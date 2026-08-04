Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Initialize-ZirconWindowsPathResolverNative {
    if ($null -ne ("ZirconEngine.WindowsPathResolver.NativeMethods" -as [type])) {
        return
    }

    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace ZirconEngine.WindowsPathResolver
{
    public static class NativeMethods
    {
        private const uint FileReadAttributes = 0x00000080;
        private const uint FileShareRead = 0x00000001;
        private const uint FileShareWrite = 0x00000002;
        private const uint FileShareDelete = 0x00000004;
        private const uint OpenExisting = 3;
        private const uint FileFlagBackupSemantics = 0x02000000;
        private const uint MoveFileWriteThrough = 0x00000008;

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern SafeFileHandle CreateFileW(
            string fileName,
            uint desiredAccess,
            uint shareMode,
            IntPtr securityAttributes,
            uint creationDisposition,
            uint flagsAndAttributes,
            IntPtr templateFile);

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern uint GetFinalPathNameByHandleW(
            SafeFileHandle file,
            StringBuilder path,
            uint capacity,
            uint flags);

        [StructLayout(LayoutKind.Sequential)]
        private struct ByHandleFileInformation
        {
            public uint FileAttributes;
            public uint CreationTimeLow;
            public uint CreationTimeHigh;
            public uint LastAccessTimeLow;
            public uint LastAccessTimeHigh;
            public uint LastWriteTimeLow;
            public uint LastWriteTimeHigh;
            public uint VolumeSerialNumber;
            public uint FileSizeHigh;
            public uint FileSizeLow;
            public uint NumberOfLinks;
            public uint FileIndexHigh;
            public uint FileIndexLow;
        }

        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetFileInformationByHandle(
            SafeFileHandle file,
            out ByHandleFileInformation information);

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern bool MoveFileExW(
            string existingFileName,
            string newFileName,
            uint flags);

        public static string GetFinalPath(string path)
        {
            SafeFileHandle handle = CreateFileW(
                path,
                FileReadAttributes,
                FileShareRead | FileShareWrite | FileShareDelete,
                IntPtr.Zero,
                OpenExisting,
                FileFlagBackupSemantics,
                IntPtr.Zero);
            if (handle.IsInvalid)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not open path for final-path resolution: " + path);
            }

            using (handle)
            {
                StringBuilder buffer = new StringBuilder(512);
                uint length = GetFinalPathNameByHandleW(handle, buffer, (uint)buffer.Capacity, 0);
                if (length == 0)
                {
                    throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve final path: " + path);
                }
                if (length >= buffer.Capacity)
                {
                    buffer = new StringBuilder((int)length + 1);
                    length = GetFinalPathNameByHandleW(handle, buffer, (uint)buffer.Capacity, 0);
                    if (length == 0 || length >= buffer.Capacity)
                    {
                        throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve final path: " + path);
                    }
                }
                return buffer.ToString();
            }
        }

        public static string GetFileIdentity(string path)
        {
            SafeFileHandle handle = CreateFileW(
                path,
                FileReadAttributes,
                FileShareRead | FileShareWrite | FileShareDelete,
                IntPtr.Zero,
                OpenExisting,
                FileFlagBackupSemantics,
                IntPtr.Zero);
            if (handle.IsInvalid)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not open path for file-identity resolution: " + path);
            }

            using (handle)
            {
                ByHandleFileInformation information;
                if (!GetFileInformationByHandle(handle, out information))
                {
                    throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve file identity: " + path);
                }
                return information.VolumeSerialNumber.ToString("X8") + ":" +
                    information.FileIndexHigh.ToString("X8") + information.FileIndexLow.ToString("X8");
            }
        }

        public static void MovePath(string sourcePath, string destinationPath)
        {
            if (!MoveFileExW(sourcePath, destinationPath, MoveFileWriteThrough))
            {
                throw new Win32Exception(
                    Marshal.GetLastWin32Error(),
                    "Could not move resolved Windows path from '" + sourcePath + "' to '" + destinationPath + "'.");
            }
        }
    }
}
'@
}

function ConvertTo-ZirconWindowsDisplayPath {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    if ($Path.StartsWith('\\?\UNC\', [StringComparison]::OrdinalIgnoreCase)) {
        return '\\' + $Path.Substring(8)
    }
    if ($Path -match '^\\\\\?\\[A-Za-z]:\\') {
        return $Path.Substring(4)
    }
    return $Path
}

function Join-ZirconWindowsPath {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        [string]$ChildPath
    )

    # PowerShell's filesystem provider does not consistently accept `\\?\` paths. Keep path
    # composition in System.IO so resolver callers can retain their physical operation paths.
    return [System.IO.Path]::Combine($Path, $ChildPath)
}

function Move-ZirconWindowsPath {
    param(
        [Parameter(Mandatory)]
        [string]$Source,
        [Parameter(Mandatory)]
        [string]$Destination
    )

    Initialize-ZirconWindowsPathResolverNative
    [ZirconEngine.WindowsPathResolver.NativeMethods]::MovePath($Source, $Destination)
}

function Get-ZirconWindowsAbsolutePath {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [string]$BasePath
    )

    # Do not use GetFullPath for an absolute input: it would collapse `junction/..` before
    # GetFinalPathNameByHandleW can resolve the junction's physical identity.
    if ($Path -match '^\\(?!\\)') {
        return [System.IO.Path]::GetFullPath($Path)
    }
    if ([System.IO.Path]::IsPathRooted($Path)) {
        return $Path
    }
    if (-not [string]::IsNullOrWhiteSpace($BasePath)) {
        return [System.IO.Path]::Combine($BasePath, $Path)
    }
    return [System.IO.Path]::Combine([Environment]::CurrentDirectory, $Path)
}

function Add-ZirconWindowsUnresolvedPathSegment {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        [string]$Segment
    )

    if ($Segment -eq '.') {
        return $Path
    }
    if ($Segment -eq '..') {
        $parent = [System.IO.Path]::GetDirectoryName($Path)
        if ([string]::IsNullOrWhiteSpace($parent)) {
            return $Path
        }
        return $parent
    }
    return Join-ZirconWindowsPath -Path $Path -ChildPath $Segment
}

function Resolve-ZirconWindowsPath {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [string]$BasePath
    )

    if ($Path -match '^[A-Za-z]:(?:$|[^\\/])' -or $Path -match '^\\\\\?\\[A-Za-z]:(?:$|[^\\/])') {
        throw "Windows paths must be drive-rooted, not drive-relative: '$Path'."
    }

    Initialize-ZirconWindowsPathResolverNative
    $requestedPath = Get-ZirconWindowsAbsolutePath -Path $Path -BasePath $BasePath
    $existingPath = $requestedPath
    $unresolvedSegments = [System.Collections.Generic.List[string]]::new()
    while (-not [System.IO.File]::Exists($existingPath) -and -not [System.IO.Directory]::Exists($existingPath)) {
        $parentPath = [System.IO.Path]::GetDirectoryName($existingPath)
        if ([string]::IsNullOrWhiteSpace($parentPath) -or $parentPath -eq $existingPath) {
            throw "Could not resolve an existing Windows path ancestor for '$requestedPath'."
        }
        $segment = [System.IO.Path]::GetFileName($existingPath)
        if ([string]::IsNullOrEmpty($segment)) {
            throw "Could not determine an unresolved Windows path segment for '$requestedPath'."
        }
        $unresolvedSegments.Insert(0, $segment)
        $existingPath = $parentPath
    }

    # Keep GetFinalPathNameByHandleW output for all filesystem operations. Removing the
    # verbatim prefix can make long paths and names with Win32-normalized suffixes unreachable.
    $operationalExistingPath = [ZirconEngine.WindowsPathResolver.NativeMethods]::GetFinalPath($existingPath)
    $operationalPath = $operationalExistingPath
    foreach ($segment in $unresolvedSegments) {
        $operationalPath = Add-ZirconWindowsUnresolvedPathSegment -Path $operationalPath -Segment $segment
    }
    $displayExistingPath = ConvertTo-ZirconWindowsDisplayPath -Path $operationalExistingPath
    $displayPath = $displayExistingPath
    foreach ($segment in $unresolvedSegments) {
        $displayPath = Add-ZirconWindowsUnresolvedPathSegment -Path $displayPath -Segment $segment
    }

    return [pscustomobject]@{
        RequestedPath           = $requestedPath
        ExistingAncestorPath    = $existingPath
        OperationalExistingPath = $operationalExistingPath
        OperationalPath         = $operationalPath
        DisplayExistingPath     = $displayExistingPath
        DisplayPath             = $displayPath
    }
}

function Get-ZirconWindowsFileIdentity {
    param(
        [Parameter(Mandatory)]
        [string]$Path
    )

    $resolution = Resolve-ZirconWindowsPath -Path $Path
    if (-not [System.IO.File]::Exists($resolution.OperationalPath) -and
        -not [System.IO.Directory]::Exists($resolution.OperationalPath)) {
        throw "Windows file identity requires an existing path: '$Path'."
    }
    return [ZirconEngine.WindowsPathResolver.NativeMethods]::GetFileIdentity($resolution.OperationalPath)
}

Export-ModuleMember -Function Resolve-ZirconWindowsPath, Get-ZirconWindowsFileIdentity, Join-ZirconWindowsPath, Move-ZirconWindowsPath
