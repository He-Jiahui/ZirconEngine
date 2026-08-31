Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($null -eq ('ZirconMvpAcceptanceNativeFileSystem' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
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

    private static string FormatIdentity(ByHandleFileInformation information)
    {
        var fileIndex = ((ulong)information.FileIndexHigh << 32) | information.FileIndexLow;
        // File indexes can be recycled after deletion. Bind the handle creation epoch so a
        // same-name replacement cannot be accepted merely because its index was reused.
        var creationTime = ((ulong)(uint)information.CreationTime.dwHighDateTime << 32) |
            (uint)information.CreationTime.dwLowDateTime;
        return information.VolumeSerialNumber.ToString("X8") + ":" + fileIndex.ToString("X16") + ":" + creationTime.ToString("X16");
    }

    public static string GetIdentity(SafeFileHandle handle)
    {
        ByHandleFileInformation information;
        if (!GetFileInformationByHandle(handle, out information))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to read handle file identity.");
        }
        return FormatIdentity(information);
    }

    private static ByHandleFileInformation GetVerifiedIdentityInformation(
        SafeFileHandle handle,
        string path)
    {
        ByHandleFileInformation information;
        if (!GetFileInformationByHandle(handle, out information))
        {
            throw new Win32Exception(Marshal.GetLastWin32Error(), "Unable to read handle file attributes.");
        }
        if ((information.FileAttributes & (uint)System.IO.FileAttributes.ReparsePoint) != 0)
        {
            throw new InvalidOperationException("Acceptance staging tree contains reparse point '" + path + "'.");
        }
        return information;
    }

    public static string GetVerifiedNonDirectoryIdentity(
        SafeFileHandle handle,
        string path,
        string kind)
    {
        var information = GetVerifiedIdentityInformation(handle, path);
        if ((information.FileAttributes & (uint)System.IO.FileAttributes.Directory) != 0)
        {
            throw new InvalidOperationException("Acceptance " + kind + " '" + path + "' is a directory.");
        }
        return FormatIdentity(information);
    }

    public static string GetVerifiedDirectoryIdentity(
        SafeFileHandle handle,
        string path,
        string kind)
    {
        var information = GetVerifiedIdentityInformation(handle, path);
        if ((information.FileAttributes & (uint)System.IO.FileAttributes.Directory) == 0)
        {
            throw new InvalidOperationException("Acceptance " + kind + " '" + path + "' is not a directory.");
        }
        return FormatIdentity(information);
    }

    public static string GetCleanupDirectoryIdentity(SafeFileHandle handle, string path)
    {
        var information = GetVerifiedIdentityInformation(handle, path);
        if ((information.FileAttributes & (uint)System.IO.FileAttributes.Directory) == 0)
        {
            throw new InvalidOperationException(
                "Acceptance publication root '" + path + "' changed before its failure cleanup could restore access.");
        }
        return FormatIdentity(information);
    }

    public static void VerifyDirectory(
        SafeFileHandle handle,
        string path,
        string kind)
    {
        var information = GetVerifiedIdentityInformation(handle, path);
        if ((information.FileAttributes & (uint)System.IO.FileAttributes.Directory) == 0)
        {
            throw new InvalidOperationException("Acceptance " + kind + " '" + path + "' is not a directory.");
        }
    }

    private static SafeFileHandle[] OpenNoFollowDirectoryLeaseCore(
        string directoryPath,
        string compatibleWriteLeaseRoot,
        bool captureTargetIdentity,
        out string targetIdentity)
    {
        targetIdentity = null;
        var absoluteDirectoryPath = System.IO.Path.GetFullPath(directoryPath);
        var compatibleRoot = String.IsNullOrWhiteSpace(compatibleWriteLeaseRoot)
            ? null
            : System.IO.Path.GetFullPath(compatibleWriteLeaseRoot);
        var paths = new List<string>();
        var currentPath = absoluteDirectoryPath;
        while (!String.IsNullOrWhiteSpace(currentPath))
        {
            paths.Add(currentPath);
            var parent = System.IO.Directory.GetParent(currentPath);
            if (parent == null)
            {
                break;
            }
            currentPath = parent.FullName;
        }

        var handles = new List<SafeFileHandle>();
        try
        {
            for (var index = paths.Count - 1; index >= 0; index--)
            {
                var path = paths[index];
                SafeFileHandle handle = null;
                try
                {
                    handle = compatibleRoot != null &&
                        path.Equals(compatibleRoot, StringComparison.OrdinalIgnoreCase)
                        ? OpenNoFollowForHeldStagingRoot(path)
                        : OpenNoFollowForPublicationParent(path);
                    if (captureTargetIdentity && index == 0)
                    {
                        targetIdentity = GetVerifiedDirectoryIdentity(handle, path, "directory lease path");
                    }
                    else
                    {
                        VerifyDirectory(handle, path, "directory lease path");
                    }
                    handles.Add(handle);
                    handle = null;
                }
                finally
                {
                    if (handle != null)
                    {
                        handle.Dispose();
                    }
                }
            }
            return handles.ToArray();
        }
        catch
        {
            DisposeHandles(handles);
            throw;
        }
    }

    public static SafeFileHandle[] OpenNoFollowDirectoryLease(
        string directoryPath,
        string compatibleWriteLeaseRoot)
    {
        string ignoredIdentity;
        return OpenNoFollowDirectoryLeaseCore(
            directoryPath,
            compatibleWriteLeaseRoot,
            false,
            out ignoredIdentity);
    }

    public static string GetVerifiedDirectoryIdentityWithLease(
        string path,
        string compatibleWriteLeaseRoot)
    {
        SafeFileHandle[] directoryLease = null;
        try
        {
            string targetIdentity;
            directoryLease = OpenNoFollowDirectoryLeaseCore(
                path,
                compatibleWriteLeaseRoot,
                true,
                out targetIdentity);
            return targetIdentity;
        }
        finally
        {
            DisposeHandles(directoryLease);
        }
    }

    public static void RemoveFileNoFollow(string path, string expectedIdentity)
    {
        var handle = OpenNoFollowForDelete(path);
        try
        {
            var actualIdentity = GetVerifiedNonDirectoryIdentity(handle, path, "file");
            if (!String.IsNullOrWhiteSpace(expectedIdentity) &&
                !actualIdentity.Equals(expectedIdentity, StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException(
                    "Acceptance file '" + path + "' identity changed before cleanup.");
            }
            MarkForDelete(handle);
        }
        finally
        {
            handle.Dispose();
        }
    }

    public static void MoveFileNoFollow(
        string sourcePath,
        string destinationPath,
        string expectedSourceIdentity,
        string compatibleWriteLeaseRoot)
    {
        var absoluteSourcePath = System.IO.Path.GetFullPath(sourcePath);
        var absoluteDestinationPath = System.IO.Path.GetFullPath(destinationPath);
        var destinationParentPath = System.IO.Path.GetDirectoryName(absoluteDestinationPath);
        if (String.IsNullOrWhiteSpace(destinationParentPath))
        {
            throw new InvalidOperationException(
                "Acceptance destination '" + destinationPath + "' has no parent directory.");
        }

        SafeFileHandle sourceHandle = null;
        SafeFileHandle[] destinationParentLease = null;
        SafeFileHandle destinationHandle = null;
        try
        {
            sourceHandle = OpenNoFollowForDelete(absoluteSourcePath);
            var sourceIdentity = GetVerifiedNonDirectoryIdentity(
                sourceHandle,
                absoluteSourcePath,
                "source");
            if (!String.IsNullOrWhiteSpace(expectedSourceIdentity) &&
                !sourceIdentity.Equals(expectedSourceIdentity, StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException(
                    "Acceptance source '" + absoluteSourcePath + "' identity changed before publication.");
            }
            destinationParentLease = OpenNoFollowDirectoryLease(
                destinationParentPath,
                compatibleWriteLeaseRoot);
            RenameTo(sourceHandle, absoluteDestinationPath);
            destinationHandle = OpenNoFollowForPublishedTree(absoluteDestinationPath);
            var destinationIdentity = GetVerifiedNonDirectoryIdentity(
                destinationHandle,
                absoluteDestinationPath,
                "destination");
            if (!destinationIdentity.Equals(sourceIdentity, StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException(
                    "Acceptance destination '" + absoluteDestinationPath +
                    "' does not identify the renamed source file.");
            }
        }
        finally
        {
            if (destinationHandle != null)
            {
                destinationHandle.Dispose();
            }
            DisposeHandles(destinationParentLease);
            if (sourceHandle != null)
            {
                sourceHandle.Dispose();
            }
        }
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

    public static bool ByteSequencesEqual(byte[] expected, byte[] actual)
    {
        if (Object.ReferenceEquals(expected, actual))
        {
            return true;
        }
        if (expected == null || actual == null || expected.Length != actual.Length)
        {
            return false;
        }
        for (var index = 0; index < expected.Length; index++)
        {
            if (expected[index] != actual[index])
            {
                return false;
            }
        }
        return true;
    }

    public static void DisposeHandles(SafeFileHandle[] handles)
    {
        if (handles == null)
        {
            return;
        }
        for (var index = handles.Length - 1; index >= 0; index--)
        {
            if (handles[index] != null)
            {
                handles[index].Dispose();
            }
        }
    }

    public static void DisposeHandles(List<SafeFileHandle> handles)
    {
        if (handles == null)
        {
            return;
        }
        for (var index = handles.Count - 1; index >= 0; index--)
        {
            if (handles[index] != null)
            {
                handles[index].Dispose();
            }
        }
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

    return [ZirconMvpAcceptanceNativeFileSystem]::ByteSequencesEqual($Expected, $Actual)
}

function Assert-MvpAcceptanceNativeSourceAttributes {
    param(
        [Parameter(Mandatory)][uint32]$Attributes,
        [Parameter(Mandatory)][string]$Path
    )

    if ([bool]($Attributes -band [uint32][System.IO.FileAttributes]::ReparsePoint)) {
        throw "Acceptance staging tree contains reparse point '$Path'."
    }
}

function Open-MvpAcceptanceNoFollowDirectoryLease {
    param(
        [Parameter(Mandatory)][string]$DirectoryPath,
        [string]$CompatibleWriteLeaseRoot
    )

    Write-Output -NoEnumerate ([ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowDirectoryLease(
        $DirectoryPath,
        $CompatibleWriteLeaseRoot))
}

function Close-MvpAcceptanceNoFollowDirectoryLease {
    param([Microsoft.Win32.SafeHandles.SafeFileHandle[]]$Handles)

    [ZirconMvpAcceptanceNativeFileSystem]::DisposeHandles($Handles)
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
    try {
        # Retain every ancestor while CreateNew resolves the leaf. A pre-existing leaf is an
        # error rather than a path that can be followed or overwritten.
        $parentLease = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowDirectoryLease(
            $parentPath,
            $CompatibleWriteLeaseRoot)
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
        $sourceIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetVerifiedNonDirectoryIdentity(
            $sourceHandle,
            $absolutePath,
            'file')
        $sourceLength = [ZirconMvpAcceptanceNativeFileSystem]::GetLength($sourceHandle)
        if ($sourceLength -ne $ContentBytes.LongLength) {
            throw "Acceptance file '$absolutePath' content changed before verification."
        }
        $inputStream = [IO.FileStream]::new($sourceHandle, [IO.FileAccess]::Read)
        $sourceHandle = $null
        [byte[]]$writtenBytes = [byte[]]::new($ContentBytes.Length)
        $offset = 0
        while ($offset -lt $writtenBytes.Length) {
            $read = $inputStream.Read($writtenBytes, $offset, $writtenBytes.Length - $offset)
            if ($read -eq 0) {
                throw "Acceptance file '$absolutePath' content changed before verification."
            }
            $offset += $read
        }
        if ($inputStream.ReadByte() -ne -1) {
            throw "Acceptance file '$absolutePath' content changed before verification."
        }
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
            [ZirconMvpAcceptanceNativeFileSystem]::DisposeHandles($parentLease)
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

    [ZirconMvpAcceptanceNativeFileSystem]::MoveFileNoFollow(
        $SourcePath,
        $DestinationPath,
        $ExpectedSourceIdentity,
        $CompatibleWriteLeaseRoot)
}

function Remove-MvpAcceptanceFileNoFollow {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$ExpectedIdentity
    )

    [ZirconMvpAcceptanceNativeFileSystem]::RemoveFileNoFollow($Path, $ExpectedIdentity)
}

function Get-MvpAcceptanceNativeDirectoryIdentity {
    param(
        [Parameter(Mandatory)][string]$Path,
        [string]$CompatibleWriteLeaseRoot
    )

    return [ZirconMvpAcceptanceNativeFileSystem]::GetVerifiedDirectoryIdentityWithLease(
        $Path,
        $CompatibleWriteLeaseRoot)
}

function Ensure-MvpAcceptanceDirectoryPathNoFollow {
    param(
        [Parameter(Mandatory)][string]$RootPath,
        [Parameter(Mandatory)][string]$RelativePath,
        [string]$CompatibleWriteLeaseRoot
    )

    $segments = $RelativePath.Split(
        [char[]]@('\', '/'),
        [StringSplitOptions]::RemoveEmptyEntries)
    if ([IO.Path]::IsPathRooted($RelativePath) -or $segments -contains '..') {
        throw "Acceptance directory path '$RelativePath' is unsafe."
    }

    $root = [IO.Path]::GetFullPath($RootPath)
    $rootLease = $null
    $nestedHandles = [System.Collections.Generic.List[Microsoft.Win32.SafeHandles.SafeFileHandle]]::new()
    try {
        $rootLease = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowDirectoryLease(
            $root,
            $CompatibleWriteLeaseRoot)
        $rootHandle = $rootLease[$rootLease.Count - 1]
        [ZirconMvpAcceptanceNativeFileSystem]::VerifyDirectory(
            $rootHandle,
            $root,
            'directory root')

        $currentPath = $root
        foreach ($segment in $segments) {
            if ([string]::IsNullOrWhiteSpace($segment) -or $segment -eq '.') {
                continue
            }
            $nextHandle = $null
            try {
                $nextPath = [IO.Path]::Combine($currentPath, $segment)
                if (-not [IO.Directory]::Exists($nextPath)) {
                    if ([IO.File]::Exists($nextPath)) {
                        throw "Acceptance directory path '$nextPath' is not a directory."
                    }
                    $null = [IO.Directory]::CreateDirectory($nextPath)
                }
                $nextHandle = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowForPublicationParent($nextPath)
                [ZirconMvpAcceptanceNativeFileSystem]::VerifyDirectory(
                    $nextHandle,
                    $nextPath,
                    'directory path')
                $null = $nestedHandles.Add($nextHandle)
                $nextHandle = $null
                $currentPath = $nextPath
            }
            finally {
                if ($null -ne $nextHandle) {
                    $nextHandle.Dispose()
                }
            }
        }
        return $currentPath
    }
    finally {
        [ZirconMvpAcceptanceNativeFileSystem]::DisposeHandles($nestedHandles)
        if ($null -ne $rootLease) {
            [ZirconMvpAcceptanceNativeFileSystem]::DisposeHandles($rootLease)
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
    try {
        $directoryLease = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowDirectoryLease(
            $absolutePath,
            $CompatibleWriteLeaseRoot)
        $directoryHandle = $directoryLease[$directoryLease.Count - 1]
        $identity = [ZirconMvpAcceptanceNativeFileSystem]::GetVerifiedDirectoryIdentity(
            $directoryHandle,
            $absolutePath,
            'publication root')
        if ($identity -ne $ExpectedIdentity) {
            throw "Acceptance publication root '$absolutePath' no longer identifies the frozen partial tree."
        }

        $directory = [IO.DirectoryInfo]::new($absolutePath)
        $accessSections = [Security.AccessControl.AccessControlSections]::Access
        $security = Get-Acl -LiteralPath $absolutePath -ErrorAction Stop
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
        Set-Acl -LiteralPath $absolutePath -AclObject $security -ErrorAction Stop
        return [pscustomobject]@{
            path = $absolutePath
            identity = $identity
            original_sddl = $originalSddl
            compatible_write_lease_root = $CompatibleWriteLeaseRoot
        }
    }
    finally {
        if ($null -ne $directoryLease) {
            [ZirconMvpAcceptanceNativeFileSystem]::DisposeHandles($directoryLease)
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
    try {
        $directoryLease = [ZirconMvpAcceptanceNativeFileSystem]::OpenNoFollowDirectoryLease(
            $absolutePath,
            $compatibleWriteLeaseRoot)
        $directoryHandle = $directoryLease[$directoryLease.Count - 1]
        $actualIdentity = [ZirconMvpAcceptanceNativeFileSystem]::GetCleanupDirectoryIdentity(
            $directoryHandle,
            $absolutePath)
        if ($actualIdentity -ne $expectedIdentity) {
            throw "Acceptance publication root '$absolutePath' changed before its failure cleanup could restore access."
        }
        $accessSections = [Security.AccessControl.AccessControlSections]::Access
        $security = [Security.AccessControl.DirectorySecurity]::new()
        $security.SetSecurityDescriptorSddlForm($originalSddl, $accessSections)
        Set-Acl -LiteralPath $absolutePath -AclObject $security -ErrorAction Stop
    }
    finally {
        if ($null -ne $directoryLease) {
            [ZirconMvpAcceptanceNativeFileSystem]::DisposeHandles($directoryLease)
        }
    }
}

Export-ModuleMember -Function Test-MvpAcceptanceNativeFileAttribute, Assert-MvpAcceptanceNativeSourceAttributes, Open-MvpAcceptanceNoFollowDirectoryLease, Close-MvpAcceptanceNoFollowDirectoryLease, Write-MvpAcceptanceNewFileNoFollow, Move-MvpAcceptanceNewFileNoFollow, Remove-MvpAcceptanceFileNoFollow, Get-MvpAcceptanceNativeDirectoryIdentity, Ensure-MvpAcceptanceDirectoryPathNoFollow, Protect-MvpAcceptanceStagingDirectoryForPublication, Unprotect-MvpAcceptanceStagingDirectoryForPublication
