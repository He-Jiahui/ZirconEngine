Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($null -eq ('ZirconMvpAcceptanceNativeFileSystem' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

public static class ZirconMvpAcceptanceNativeFileSystem
{
    private const uint GenericRead = 0x80000000;
    private const uint Delete = 0x00010000;
    private const uint FileTraverse = 0x00000020;
    private const uint FileReadAttributes = 0x00000080;
    private const uint FileShareRead = 0x00000001;
    private const uint FileShareWrite = 0x00000002;
    private const uint FileShareDelete = 0x00000004;
    private const uint OpenExisting = 3;
    private const uint FileFlagBackupSemantics = 0x02000000;
    private const uint FileFlagOpenReparsePoint = 0x00200000;

    [StructLayout(LayoutKind.Sequential)]
    private struct ByHandleFileInformation
    {
        public uint FileAttributes;
        public System.Runtime.InteropServices.ComTypes.FILETIME CreationTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME LastAccessTime;
        public System.Runtime.InteropServices.ComTypes.FILETIME LastWriteTime;
        public uint VolumeSerialNumber;
        public uint FileSizeHigh;
        public uint FileSizeLow;
        public uint NumberOfLinks;
        public uint FileIndexHigh;
        public uint FileIndexLow;
    }

    [StructLayout(LayoutKind.Sequential)]
    private struct FileDispositionInformation
    {
        [MarshalAs(UnmanagedType.Bool)]
        public bool DeleteFile;
    }

    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    private static extern SafeFileHandle CreateFile(
        string fileName,
        uint desiredAccess,
        uint shareMode,
        IntPtr securityAttributes,
        uint creationDisposition,
        uint flagsAndAttributes,
        IntPtr templateFile);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool GetFileInformationByHandle(
        SafeFileHandle file,
        out ByHandleFileInformation information);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetFileInformationByHandle(
        SafeFileHandle file,
        int fileInformationClass,
        ref FileDispositionInformation information,
        uint bufferSize);

    [DllImport("kernel32.dll", SetLastError = true)]
    private static extern bool SetFileInformationByHandle(
        SafeFileHandle file,
        int fileInformationClass,
        IntPtr information,
        uint bufferSize);

    public static SafeFileHandle OpenNoFollow(string path, bool readContents)
    {
        var desiredAccess = FileReadAttributes | (readContents ? GenericRead : 0u);
        var handle = CreateFile(
            path,
            desiredAccess,
            FileShareRead,
            IntPtr.Zero,
            OpenExisting,
            FileFlagBackupSemantics | FileFlagOpenReparsePoint,
            IntPtr.Zero);
        if (handle.IsInvalid)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to open " + path + " without following reparse points.");
        }
        return handle;
    }

    public static SafeFileHandle OpenNoFollowForDelete(string path)
    {
        var handle = CreateFile(
            path,
            FileReadAttributes | Delete,
            FileShareRead,
            IntPtr.Zero,
            OpenExisting,
            FileFlagBackupSemantics | FileFlagOpenReparsePoint,
            IntPtr.Zero);
        if (handle.IsInvalid)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to open " + path + " for safe deletion.");
        }
        return handle;
    }

    public static SafeFileHandle OpenNoFollowForStagingWriteLease(string path)
    {
        var handle = CreateFile(
            path,
            FileReadAttributes | Delete,
            FileShareRead | FileShareWrite,
            IntPtr.Zero,
            OpenExisting,
            FileFlagBackupSemantics | FileFlagOpenReparsePoint,
            IntPtr.Zero);
        if (handle.IsInvalid)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to open " + path + " for an acceptance staging write lease.");
        }
        return handle;
    }

    public static SafeFileHandle OpenNoFollowForHeldStagingRoot(string path)
    {
        var handle = CreateFile(
            path,
            FileReadAttributes,
            FileShareRead | FileShareWrite | FileShareDelete,
            IntPtr.Zero,
            OpenExisting,
            FileFlagBackupSemantics | FileFlagOpenReparsePoint,
            IntPtr.Zero);
        if (handle.IsInvalid)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to reopen held acceptance staging root " + path + " without following reparse points.");
        }
        return handle;
    }

    public static SafeFileHandle OpenNoFollowForPublicationRoot(string path)
    {
        var handle = CreateFile(
            path,
            FileReadAttributes | Delete,
            FileShareRead | FileShareDelete,
            IntPtr.Zero,
            OpenExisting,
            FileFlagBackupSemantics | FileFlagOpenReparsePoint,
            IntPtr.Zero);
        if (handle.IsInvalid)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to open " + path + " for acceptance publication.");
        }
        return handle;
    }

    public static SafeFileHandle OpenNoFollowForPublicationParent(string path)
    {
        var handle = CreateFile(
            path,
            FileTraverse | FileReadAttributes,
            FileShareRead | FileShareWrite,
            IntPtr.Zero,
            OpenExisting,
            FileFlagBackupSemantics | FileFlagOpenReparsePoint,
            IntPtr.Zero);
        if (handle.IsInvalid)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to open " + path + " as an acceptance publication parent.");
        }
        return handle;
    }

    public static SafeFileHandle OpenNoFollowForPublishedTree(string path)
    {
        var handle = CreateFile(
            path,
            FileReadAttributes,
            FileShareRead | FileShareDelete,
            IntPtr.Zero,
            OpenExisting,
            FileFlagBackupSemantics | FileFlagOpenReparsePoint,
            IntPtr.Zero);
        if (handle.IsInvalid)
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to reopen published acceptance tree " + path + " without following reparse points.");
        }
        return handle;
    }

    public static uint GetAttributes(SafeFileHandle handle)
    {
        ByHandleFileInformation information;
        if (!GetFileInformationByHandle(handle, out information))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to read handle file attributes.");
        }
        return information.FileAttributes;
    }

    public static string GetIdentity(SafeFileHandle handle)
    {
        ByHandleFileInformation information;
        if (!GetFileInformationByHandle(handle, out information))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to read handle file identity.");
        }
        var fileIndex = ((ulong)information.FileIndexHigh << 32) | information.FileIndexLow;
        // File indexes can be recycled after deletion. Bind the handle creation epoch so a
        // same-name replacement cannot be accepted merely because its index was reused.
        var creationTime = ((ulong)(uint)information.CreationTime.dwHighDateTime << 32) |
            (uint)information.CreationTime.dwLowDateTime;
        return information.VolumeSerialNumber.ToString("X8") + ":" + fileIndex.ToString("X16") + ":" + creationTime.ToString("X16");
    }

    public static long GetLength(SafeFileHandle handle)
    {
        ByHandleFileInformation information;
        if (!GetFileInformationByHandle(handle, out information))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to read handle file length.");
        }
        return ((long)information.FileSizeHigh << 32) | information.FileSizeLow;
    }

    public static void MarkForDelete(SafeFileHandle handle)
    {
        var information = new FileDispositionInformation { DeleteFile = true };
        if (!SetFileInformationByHandle(
            handle,
            4,
            ref information,
            (uint)Marshal.SizeOf(typeof(FileDispositionInformation))))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to mark handle for deletion.");
        }
    }

    public static void RenameTo(SafeFileHandle source, string destinationPath)
    {
        if (String.IsNullOrWhiteSpace(destinationPath) ||
            !System.IO.Path.IsPathRooted(destinationPath))
        {
            throw new ArgumentException("The destination path must be absolute.", "destinationPath");
        }

        var nameBytes = System.Text.Encoding.Unicode.GetBytes(destinationPath + "\0");
        var rootDirectoryOffset = IntPtr.Size == 8 ? 8 : 4;
        var fileNameLengthOffset = rootDirectoryOffset + IntPtr.Size;
        var fileRenameInfoFileNameOffset = fileNameLengthOffset + sizeof(uint);
        var bufferSize = checked(fileRenameInfoFileNameOffset + nameBytes.Length);
        var buffer = Marshal.AllocHGlobal(bufferSize);
        try
        {
            Marshal.Copy(new byte[bufferSize], 0, buffer, bufferSize);
            Marshal.WriteInt32(buffer, 0, 0);
            Marshal.WriteIntPtr(buffer, rootDirectoryOffset, IntPtr.Zero);
            Marshal.WriteInt32(buffer, fileNameLengthOffset, nameBytes.Length - sizeof(char));
            Marshal.Copy(nameBytes, 0, IntPtr.Add(buffer, fileRenameInfoFileNameOffset), nameBytes.Length);
            if (!SetFileInformationByHandle(source, 3, buffer, (uint)bufferSize))
            {
                var error = Marshal.GetLastWin32Error();
                throw new Win32Exception(error, "Unable to rename source handle (Win32 " + error + ").");
            }
        }
        finally
        {
            Marshal.FreeHGlobal(buffer);
        }
    }
}
'@ -ErrorAction Stop
}

function Test-MvpAcceptanceNativeFileAttribute {
    param(
        [Parameter(Mandatory)][uint32]$Attributes,
        [Parameter(Mandatory)][System.IO.FileAttributes]$Expected
    )

    return [bool]($Attributes -band [uint32]$Expected)
}

function Test-MvpAcceptanceNativeByteSequence {
    param(
        [Parameter(Mandatory)][byte[]]$Expected,
        [Parameter(Mandatory)][byte[]]$Actual
    )

    if ($Expected.Length -ne $Actual.Length) {
        return $false
    }
    for ($index = 0; $index -lt $Expected.Length; $index++) {
        if ($Expected[$index] -ne $Actual[$index]) {
            return $false
        }
    }
    return $true
}

function Assert-MvpAcceptanceNativeSourceAttributes {
    param(
        [Parameter(Mandatory)][uint32]$Attributes,
        [Parameter(Mandatory)][string]$Path
    )

    if (Test-MvpAcceptanceNativeFileAttribute -Attributes $Attributes -Expected ([System.IO.FileAttributes]::ReparsePoint)) {
        throw "Acceptance staging tree contains reparse point '$Path'."
    }
}

function Open-MvpAcceptanceNoFollowDirectoryLease {
    param(
        [Parameter(Mandatory)][string]$DirectoryPath,
        [string]$CompatibleWriteLeaseRoot
    )

    $absoluteDirectoryPath = [IO.Path]::GetFullPath($DirectoryPath)
    $compatibleRoot = if ([string]::IsNullOrWhiteSpace($CompatibleWriteLeaseRoot)) {
        $null
    }
    else {
        [IO.Path]::GetFullPath($CompatibleWriteLeaseRoot)
    }
    $paths = [System.Collections.Generic.List[string]]::new()
    $currentPath = $absoluteDirectoryPath
    while (-not [string]::IsNullOrWhiteSpace($currentPath)) {
        $null = $paths.Add($currentPath)
        $parent = [IO.Directory]::GetParent($currentPath)
        if ($null -eq $parent) {
            break
        }
        $currentPath = $parent.FullName
    }

    $handles = [System.Collections.Generic.List[Microsoft.Win32.SafeHandles.SafeFileHandle]]::new()
    try {
        for ($index = $paths.Count - 1; $index -ge 0; $index--) {
            $path = $paths[$index]
            $handle = if ($null -ne $compatibleRoot -and
                $path.Equals($compatibleRoot, [StringComparison]::OrdinalIgnoreCase)) {
                [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForHeldStagingRoot($path)
            }
            else {
                [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($path)
            }
            try {
                $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($handle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $path
                if (-not (Test-MvpAcceptanceNativeFileAttribute `
                    -Attributes $attributes `
                    -Expected ([System.IO.FileAttributes]::Directory))) {
                    throw "Acceptance directory lease path '$path' is not a directory."
                }
                $null = $handles.Add($handle)
                $handle = $null
            }
            finally {
                if ($null -ne $handle) {
                    $handle.Dispose()
                }
            }
        }
        Write-Output -NoEnumerate $handles.ToArray()
    }
    catch {
        for ($index = $handles.Count - 1; $index -ge 0; $index--) {
            $handles[$index].Dispose()
        }
        throw
    }
}

function Close-MvpAcceptanceNoFollowDirectoryLease {
    param([Microsoft.Win32.SafeHandles.SafeFileHandle[]]$Handles)

    if ($null -eq $Handles) {
        return
    }
    for ($index = $Handles.Count - 1; $index -ge 0; $index--) {
        $Handles[$index].Dispose()
    }
}

function Write-MvpAcceptanceNewFileNoFollow {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][byte[]]$ContentBytes,
        [switch]$PassThruDetails,
        [scriptblock]$BeforeReopenHook,
        [string]$CompatibleWriteLeaseRoot
    )

    $absolutePath = [IO.Path]::GetFullPath($Path)
    $parentPath = [IO.Path]::GetDirectoryName($absolutePath)
    if ([string]::IsNullOrWhiteSpace($parentPath)) {
        throw "Acceptance file '$Path' has no parent directory."
    }

    $parentLease = $null
    $outputStream = $null
    $sourceHandle = $null
    $inputStream = $null
    $memoryStream = $null
    try {
        # Retain every ancestor while CreateNew resolves the leaf. A pre-existing leaf is an
        # error rather than a path that can be followed or overwritten.
        $parentLease = Open-MvpAcceptanceNoFollowDirectoryLease `
            -DirectoryPath $parentPath `
            -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        $outputStream = [IO.File]::Open(
            $absolutePath,
            [IO.FileMode]::CreateNew,
            [IO.FileAccess]::Write,
            [IO.FileShare]::None)
        $outputStream.Write($ContentBytes, 0, $ContentBytes.Length)
        $outputStream.Flush()
        $outputStream.Dispose()
        $outputStream = $null

        if ($null -ne $BeforeReopenHook) {
            & $BeforeReopenHook $absolutePath
        }
        $sourceHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollow($absolutePath, $true)
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($sourceHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $absolutePath
        if (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory)) {
            throw "Acceptance file '$absolutePath' is a directory."
        }
        $sourceIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($sourceHandle)
        $inputStream = [IO.FileStream]::new($sourceHandle, [IO.FileAccess]::Read)
        $sourceHandle = $null
        $memoryStream = [IO.MemoryStream]::new()
        $inputStream.CopyTo($memoryStream)
        $writtenBytes = $memoryStream.ToArray()
        if (-not (Test-MvpAcceptanceNativeByteSequence -Expected $ContentBytes -Actual $writtenBytes)) {
            throw "Acceptance file '$absolutePath' content changed before verification."
        }
        if ($PassThruDetails) {
            Write-Output -NoEnumerate ([pscustomobject]@{
                content_bytes = $writtenBytes
                identity = $sourceIdentity
            })
        }
        else {
            Write-Output -NoEnumerate $writtenBytes
        }
    }
    finally {
        if ($null -ne $memoryStream) {
            $memoryStream.Dispose()
        }
        if ($null -ne $inputStream) {
            $inputStream.Dispose()
        }
        if ($null -ne $sourceHandle) {
            $sourceHandle.Dispose()
        }
        if ($null -ne $outputStream) {
            $outputStream.Dispose()
        }
        if ($null -ne $parentLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $parentLease
        }
    }
}

function Move-MvpAcceptanceNewFileNoFollow {
    param(
        [Parameter(Mandatory)][string]$SourcePath,
        [Parameter(Mandatory)][string]$DestinationPath,
        [string]$ExpectedSourceIdentity,
        [string]$CompatibleWriteLeaseRoot
    )

    $absoluteSourcePath = [IO.Path]::GetFullPath($SourcePath)
    $absoluteDestinationPath = [IO.Path]::GetFullPath($DestinationPath)
    $destinationParentPath = [IO.Path]::GetDirectoryName($absoluteDestinationPath)
    if ([string]::IsNullOrWhiteSpace($destinationParentPath)) {
        throw "Acceptance destination '$DestinationPath' has no parent directory."
    }

    $sourceHandle = $null
    $destinationParentLease = $null
    $destinationHandle = $null
    try {
        $sourceHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForDelete(
            $absoluteSourcePath)
        $sourceAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($sourceHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $sourceAttributes -Path $absoluteSourcePath
        if (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $sourceAttributes `
            -Expected ([System.IO.FileAttributes]::Directory)) {
            throw "Acceptance source '$absoluteSourcePath' is a directory."
        }
        $sourceIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($sourceHandle)
        if (-not [string]::IsNullOrWhiteSpace($ExpectedSourceIdentity) -and
            $sourceIdentity -ne $ExpectedSourceIdentity) {
            throw "Acceptance source '$absoluteSourcePath' identity changed before publication."
        }
        $destinationParentLease = Open-MvpAcceptanceNoFollowDirectoryLease `
            -DirectoryPath $destinationParentPath `
            -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot

        # RenameTo does not replace an existing destination. The held source handle and the
        # destination ancestor lease keep both names from being redirected during the commit.
        [ZirconMvpAcceptanceNativeFileSystem]::RenameTo($sourceHandle, $absoluteDestinationPath)
        # The source rename handle still owns DELETE access. Reopen with compatible sharing
        # while it remains held so the destination identity is verified on Windows.
        $destinationHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublishedTree(
            $absoluteDestinationPath)
        $destinationAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($destinationHandle)
        Assert-MvpAcceptanceNativeSourceAttributes `
            -Attributes $destinationAttributes `
            -Path $absoluteDestinationPath
        if (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $destinationAttributes `
            -Expected ([System.IO.FileAttributes]::Directory)) {
            throw "Acceptance destination '$absoluteDestinationPath' is a directory."
        }
        if ([ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($destinationHandle) -ne $sourceIdentity) {
            throw "Acceptance destination '$absoluteDestinationPath' does not identify the renamed source file."
        }
    }
    finally {
        if ($null -ne $destinationHandle) {
            $destinationHandle.Dispose()
        }
        if ($null -ne $destinationParentLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $destinationParentLease
        }
        if ($null -ne $sourceHandle) {
            $sourceHandle.Dispose()
        }
    }
}

function Remove-MvpAcceptanceFileNoFollow {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedIdentity
    )

    $handle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForDelete($Path)
    try {
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($handle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $Path
        if (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory)) {
            throw "Acceptance file '$Path' is a directory."
        }
        $actualIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($handle)
        if (-not [string]::IsNullOrWhiteSpace($ExpectedIdentity) -and
            $actualIdentity -ne $ExpectedIdentity) {
            throw "Acceptance file '$Path' identity changed before cleanup."
        }
        [ZirconMvpAcceptanceNativeFileSystem]::MarkForDelete($handle)
    }
    finally {
        $handle.Dispose()
    }
}

function Get-MvpAcceptanceNativeDirectoryIdentity {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$CompatibleWriteLeaseRoot
    )

    $directoryLease = $null
    $directoryHandle = $null
    try {
        $absolutePath = [IO.Path]::GetFullPath($Path)
        $directoryLease = Open-MvpAcceptanceNoFollowDirectoryLease `
            -DirectoryPath $absolutePath `
            -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        $directoryHandle = if (-not [string]::IsNullOrWhiteSpace($CompatibleWriteLeaseRoot) -and
            $absolutePath.Equals([IO.Path]::GetFullPath($CompatibleWriteLeaseRoot), [StringComparison]::OrdinalIgnoreCase)) {
            [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForHeldStagingRoot($absolutePath)
        }
        else {
            [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($absolutePath)
        }
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($directoryHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $absolutePath
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance directory '$absolutePath' is not a directory."
        }
        return [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($directoryHandle)
    }
    finally {
        if ($null -ne $directoryHandle) {
            $directoryHandle.Dispose()
        }
        if ($null -ne $directoryLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $directoryLease
        }
    }
}

function Ensure-MvpAcceptanceDirectoryPathNoFollow {
    param(
        [Parameter(Mandatory)][string]$RootPath,
        [Parameter(Mandatory)][string]$RelativePath,
        [string]$CompatibleWriteLeaseRoot
    )

    if ([IO.Path]::IsPathRooted($RelativePath) -or $RelativePath -match '(^|[\\/])\.\.([\\/]|$)') {
        throw "Acceptance directory path '$RelativePath' is unsafe."
    }

    $root = [IO.Path]::GetFullPath($RootPath)
    $rootHandle = $null
    $rootLease = $null
    try {
        $rootLease = Open-MvpAcceptanceNoFollowDirectoryLease `
            -DirectoryPath $root `
            -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        $rootHandle = if (-not [string]::IsNullOrWhiteSpace($CompatibleWriteLeaseRoot) -and
            $root.Equals([IO.Path]::GetFullPath($CompatibleWriteLeaseRoot), [StringComparison]::OrdinalIgnoreCase)) {
            [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForHeldStagingRoot($root)
        }
        else {
            [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($root)
        }
        $rootAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($rootHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $rootAttributes -Path $root
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $rootAttributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance directory root '$root' is not a directory."
        }

        $currentPath = $root
        foreach ($segment in @($RelativePath -split '[\\/]+')) {
            if ([string]::IsNullOrWhiteSpace($segment) -or $segment -eq '.') {
                continue
            }
            $currentLease = $null
            $currentHandle = $null
            $nextHandle = $null
            try {
                $currentLease = Open-MvpAcceptanceNoFollowDirectoryLease `
                    -DirectoryPath $currentPath `
                    -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
                $currentHandle = if (-not [string]::IsNullOrWhiteSpace($CompatibleWriteLeaseRoot) -and
                    $currentPath.Equals([IO.Path]::GetFullPath($CompatibleWriteLeaseRoot), [StringComparison]::OrdinalIgnoreCase)) {
                    [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForHeldStagingRoot($currentPath)
                }
                else {
                    [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($currentPath)
                }
                $currentAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($currentHandle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $currentAttributes -Path $currentPath
                if (-not (Test-MvpAcceptanceNativeFileAttribute `
                    -Attributes $currentAttributes `
                    -Expected ([System.IO.FileAttributes]::Directory))) {
                    throw "Acceptance directory path '$currentPath' is not a directory."
                }

                $nextPath = Join-Path $currentPath $segment
                if (-not (Test-Path -LiteralPath $nextPath)) {
                    [IO.Directory]::CreateDirectory($nextPath) | Out-Null
                }
                $nextHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($nextPath)
                $nextAttributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($nextHandle)
                Assert-MvpAcceptanceNativeSourceAttributes -Attributes $nextAttributes -Path $nextPath
                if (-not (Test-MvpAcceptanceNativeFileAttribute `
                    -Attributes $nextAttributes `
                    -Expected ([System.IO.FileAttributes]::Directory))) {
                    throw "Acceptance directory path '$nextPath' is not a directory."
                }
                $currentPath = $nextPath
            }
            finally {
                if ($null -ne $nextHandle) {
                    $nextHandle.Dispose()
                }
                if ($null -ne $currentHandle) {
                    $currentHandle.Dispose()
                }
                if ($null -ne $currentLease) {
                    Close-MvpAcceptanceNoFollowDirectoryLease -Handles $currentLease
                }
            }
        }
        return $currentPath
    }
    finally {
        if ($null -ne $rootHandle) {
            $rootHandle.Dispose()
        }
        if ($null -ne $rootLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $rootLease
        }
    }
}

function Protect-MvpAcceptanceStagingDirectoryForPublication {
    param(
        [Parameter(Mandatory)][string]$Path,
        [Parameter(Mandatory)][string]$ExpectedIdentity,
        [string]$CompatibleWriteLeaseRoot
    )

    $absolutePath = [IO.Path]::GetFullPath($Path)
    $directoryLease = $null
    $directoryHandle = $null
    try {
        $directoryLease = Open-MvpAcceptanceNoFollowDirectoryLease `
            -DirectoryPath $absolutePath `
            -CompatibleWriteLeaseRoot $CompatibleWriteLeaseRoot
        $directoryHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForHeldStagingRoot($absolutePath)
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($directoryHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $absolutePath
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory))) {
            throw "Acceptance publication root '$absolutePath' is not a directory."
        }
        $identity = [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($directoryHandle)
        if ($identity -ne $ExpectedIdentity) {
            throw "Acceptance publication root '$absolutePath' no longer identifies the frozen partial tree."
        }

        $directory = [IO.DirectoryInfo]::new($absolutePath)
        $accessSections = [Security.AccessControl.AccessControlSections]::Access
        $security = [IO.FileSystemAclExtensions]::GetAccessControl($directory, $accessSections)
        $originalSddl = $security.GetSecurityDescriptorSddlForm($accessSections)
        $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent().User
        if ($null -eq $currentUser) {
            throw "Acceptance publication root '$absolutePath' has no current Windows security identity."
        }
        $blockedRights = [Security.AccessControl.FileSystemRights]::CreateFiles -bor
            [Security.AccessControl.FileSystemRights]::CreateDirectories -bor
            [Security.AccessControl.FileSystemRights]::Delete -bor
            [Security.AccessControl.FileSystemRights]::DeleteSubdirectoriesAndFiles -bor
            [Security.AccessControl.FileSystemRights]::WriteAttributes -bor
            [Security.AccessControl.FileSystemRights]::WriteExtendedAttributes
        $freezeRule = [Security.AccessControl.FileSystemAccessRule]::new(
            $currentUser,
            $blockedRights,
            [Security.AccessControl.InheritanceFlags]::None,
            [Security.AccessControl.PropagationFlags]::None,
            [Security.AccessControl.AccessControlType]::Deny)
        $security.AddAccessRule($freezeRule) | Out-Null
        [IO.FileSystemAclExtensions]::SetAccessControl($directory, $security)
        return [pscustomobject]@{
            path = $absolutePath
            identity = $identity
            original_sddl = $originalSddl
            compatible_write_lease_root = $CompatibleWriteLeaseRoot
        }
    }
    finally {
        if ($null -ne $directoryHandle) {
            $directoryHandle.Dispose()
        }
        if ($null -ne $directoryLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $directoryLease
        }
    }
}

function Unprotect-MvpAcceptanceStagingDirectoryForPublication {
    param([Parameter(Mandatory)]$Protection)

    $absolutePath = [IO.Path]::GetFullPath([string]$Protection.path)
    $expectedIdentity = [string]$Protection.identity
    $originalSddl = [string]$Protection.original_sddl
    $compatibleWriteLeaseRoot = [string]$Protection.compatible_write_lease_root
    if ([string]::IsNullOrWhiteSpace($expectedIdentity) -or [string]::IsNullOrWhiteSpace($originalSddl)) {
        throw 'Acceptance publication protection is missing its root identity or original access control descriptor.'
    }

    $directoryLease = $null
    $directoryHandle = $null
    try {
        $directoryLease = Open-MvpAcceptanceNoFollowDirectoryLease `
            -DirectoryPath $absolutePath `
            -CompatibleWriteLeaseRoot $compatibleWriteLeaseRoot
        $directoryHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForHeldStagingRoot($absolutePath)
        $attributes = [ZirconMvpAcceptanceNativeFileSystem]::GetAttributes($directoryHandle)
        Assert-MvpAcceptanceNativeSourceAttributes -Attributes $attributes -Path $absolutePath
        if (-not (Test-MvpAcceptanceNativeFileAttribute `
            -Attributes $attributes `
            -Expected ([System.IO.FileAttributes]::Directory)) -or
            [ZirconMvpAcceptanceNativeFileSystem]::GetIdentity($directoryHandle) -ne $expectedIdentity) {
            throw "Acceptance publication root '$absolutePath' changed before its failure cleanup could restore access."
        }
        $accessSections = [Security.AccessControl.AccessControlSections]::Access
        $security = [Security.AccessControl.DirectorySecurity]::new()
        $security.SetSecurityDescriptorSddlForm($originalSddl, $accessSections)
        [IO.FileSystemAclExtensions]::SetAccessControl([IO.DirectoryInfo]::new($absolutePath), $security)
    }
    finally {
        if ($null -ne $directoryHandle) {
            $directoryHandle.Dispose()
        }
        if ($null -ne $directoryLease) {
            Close-MvpAcceptanceNoFollowDirectoryLease -Handles $directoryLease
        }
    }
}

Export-ModuleMember -Function Test-MvpAcceptanceNativeFileAttribute, Assert-MvpAcceptanceNativeSourceAttributes, Open-MvpAcceptanceNoFollowDirectoryLease, Close-MvpAcceptanceNoFollowDirectoryLease, Write-MvpAcceptanceNewFileNoFollow, Move-MvpAcceptanceNewFileNoFollow, Remove-MvpAcceptanceFileNoFollow, Get-MvpAcceptanceNativeDirectoryIdentity, Ensure-MvpAcceptanceDirectoryPathNoFollow, Protect-MvpAcceptanceStagingDirectoryForPublication, Unprotect-MvpAcceptanceStagingDirectoryForPublication
