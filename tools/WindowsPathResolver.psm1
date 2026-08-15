Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Initialize-ZirconWindowsPathResolverNative {
    # Keep the loaded interop contract distinct from earlier module revisions.
    if ($null -ne ("ZirconEngine.WindowsPathResolver.NativeMethodsV4" -as [type])) {
        return
    }

    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace ZirconEngine.WindowsPathResolver
{
    public static class NativeMethodsV4
    {
        private const uint FileListDirectory = 0x00000001;
        private const uint FileReadAttributes = 0x00000080;
        private const uint FileTraverse = 0x00000020;
        private const uint FileAddFile = 0x00000002;
        private const uint FileAddSubdirectory = 0x00000004;
        private const uint FileDelete = 0x00010000;
        private const uint FileAttributeDirectory = 0x00000010;
        private const uint FileAttributeReparsePoint = 0x00000400;
        private const uint FileShareRead = 0x00000001;
        private const uint FileShareWrite = 0x00000002;
        private const uint FileShareDelete = 0x00000004;
        private const uint FileSynchronize = 0x00100000;
        private const uint OpenExisting = 3;
        private const uint FileFlagBackupSemantics = 0x02000000;
        private const uint FileFlagOpenReparsePoint = 0x00200000;
        private const uint MoveFileWriteThrough = 0x00000008;
        private const int FileRenameInfo = 3;
        private const int FileDispositionInfo = 4;
        private const int FileNamesInformation = 12;
        private const uint FileOpen = 1;
        private const uint FileSynchronousIoNonAlert = 0x00000020;
        private const uint ObjectCaseInsensitive = 0x00000040;
        private const uint StatusNoMoreFiles = 0x80000006;

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

        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool SetFileInformationByHandle(
            SafeFileHandle file,
            int fileInformationClass,
            IntPtr fileInformation,
            uint bufferSize);

        [StructLayout(LayoutKind.Sequential)]
        private struct FileRenameInformation
        {
            [MarshalAs(UnmanagedType.U1)]
            public bool ReplaceIfExists;
            public IntPtr RootDirectory;
            public uint FileNameLength;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct FileDispositionInformation
        {
            [MarshalAs(UnmanagedType.U1)]
            public bool DeleteFile;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct IoStatusBlock
        {
            public IntPtr StatusPointer;
            public UIntPtr Information;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct UnicodeString
        {
            public ushort Length;
            public ushort MaximumLength;
            public IntPtr Buffer;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct ObjectAttributes
        {
            public uint Length;
            public IntPtr RootDirectory;
            public IntPtr ObjectName;
            public uint Attributes;
            public IntPtr SecurityDescriptor;
            public IntPtr SecurityQualityOfService;
        }

        [DllImport("ntdll.dll", ExactSpelling = true)]
        private static extern int NtCreateFile(
            out SafeFileHandle fileHandle,
            uint desiredAccess,
            ref ObjectAttributes objectAttributes,
            out IoStatusBlock ioStatusBlock,
            IntPtr allocationSize,
            uint fileAttributes,
            uint shareAccess,
            uint createDisposition,
            uint createOptions,
            IntPtr eaBuffer,
            uint eaLength);

        [DllImport("ntdll.dll", ExactSpelling = true)]
        private static extern int NtQueryDirectoryFile(
            SafeFileHandle fileHandle,
            IntPtr eventHandle,
            IntPtr apcRoutine,
            IntPtr apcContext,
            out IoStatusBlock ioStatusBlock,
            IntPtr fileInformation,
            uint length,
            int fileInformationClass,
            [MarshalAs(UnmanagedType.U1)] bool returnSingleEntry,
            IntPtr fileName,
            [MarshalAs(UnmanagedType.U1)] bool restartScan);

        private static SafeFileHandle OpenDirectory(string path, uint desiredAccess)
        {
            return OpenDirectory(
                path,
                desiredAccess,
                FileShareRead | FileShareWrite | FileShareDelete);
        }

        private static SafeFileHandle OpenDirectory(
            string path,
            uint desiredAccess,
            uint shareMode)
        {
            return OpenDirectory(path, desiredAccess, shareMode, false);
        }

        private static SafeFileHandle OpenDirectory(
            string path,
            uint desiredAccess,
            uint shareMode,
            bool openReparsePoint)
        {
            SafeFileHandle handle = CreateFileW(
                path,
                desiredAccess,
                shareMode,
                IntPtr.Zero,
                OpenExisting,
                FileFlagBackupSemantics | (openReparsePoint ? FileFlagOpenReparsePoint : 0),
                IntPtr.Zero);
            if (handle.IsInvalid)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not open directory for resolved Windows path operation: " + path);
            }
            return handle;
        }

        private static SafeFileHandle OpenDirectoryEntryRelative(
            SafeFileHandle parentDirectoryHandle,
            string entryName)
        {
            if (parentDirectoryHandle == null || parentDirectoryHandle.IsInvalid || parentDirectoryHandle.IsClosed)
            {
                throw new ArgumentException("Resolved Windows parent directory lease is not open.", "parentDirectoryHandle");
            }
            if (String.IsNullOrWhiteSpace(entryName) ||
                entryName == "." ||
                entryName == ".." ||
                entryName.IndexOfAny(new[] { '\\', '/' }) >= 0)
            {
                throw new ArgumentException("Resolved Windows directory entry name is not a plain child name.", "entryName");
            }

            IntPtr nameBuffer = Marshal.StringToHGlobalUni(entryName);
            IntPtr unicodeStringPointer = IntPtr.Zero;
            try
            {
                UnicodeString unicodeString = new UnicodeString();
                unicodeString.Length = checked((ushort)(entryName.Length * sizeof(char)));
                unicodeString.MaximumLength = checked((ushort)(unicodeString.Length + sizeof(char)));
                unicodeString.Buffer = nameBuffer;
                unicodeStringPointer = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(UnicodeString)));
                Marshal.StructureToPtr(unicodeString, unicodeStringPointer, false);

                ObjectAttributes objectAttributes = new ObjectAttributes();
                objectAttributes.Length = (uint)Marshal.SizeOf(typeof(ObjectAttributes));
                objectAttributes.RootDirectory = parentDirectoryHandle.DangerousGetHandle();
                objectAttributes.ObjectName = unicodeStringPointer;
                objectAttributes.Attributes = ObjectCaseInsensitive;

                IoStatusBlock ioStatusBlock;
                SafeFileHandle entryHandle;
                int status = NtCreateFile(
                    out entryHandle,
                    FileDelete | FileReadAttributes | FileListDirectory | FileSynchronize,
                    ref objectAttributes,
                    out ioStatusBlock,
                    IntPtr.Zero,
                    0,
                    FileShareRead,
                    FileOpen,
                    FileFlagOpenReparsePoint | FileSynchronousIoNonAlert,
                    IntPtr.Zero,
                    0);
                if (status != 0 || entryHandle == null || entryHandle.IsInvalid)
                {
                    if (entryHandle != null)
                    {
                        entryHandle.Dispose();
                    }
                    throw new IOException(
                        "Could not open resolved Windows directory entry relative to its held parent lease (NTSTATUS 0x" +
                        unchecked((uint)status).ToString("X8") + ").");
                }
                return entryHandle;
            }
            finally
            {
                if (unicodeStringPointer != IntPtr.Zero)
                {
                    Marshal.FreeHGlobal(unicodeStringPointer);
                }
                Marshal.FreeHGlobal(nameBuffer);
            }
        }

        private static IEnumerable<string> EnumerateDirectoryEntryNames(SafeFileHandle directoryHandle)
        {
            const int BufferLength = 64 * 1024;
            const int FileNamesInformationHeaderLength = 12;
            IntPtr buffer = Marshal.AllocHGlobal(BufferLength);
            bool restartScan = true;
            try
            {
                while (true)
                {
                    IoStatusBlock ioStatusBlock;
                    int status = NtQueryDirectoryFile(
                        directoryHandle,
                        IntPtr.Zero,
                        IntPtr.Zero,
                        IntPtr.Zero,
                        out ioStatusBlock,
                        buffer,
                        BufferLength,
                        FileNamesInformation,
                        false,
                        IntPtr.Zero,
                        restartScan);
                    restartScan = false;
                    if (unchecked((uint)status) == StatusNoMoreFiles)
                    {
                        yield break;
                    }
                    if (status != 0)
                    {
                        throw new IOException(
                            "Could not enumerate resolved Windows directory through its held lease (NTSTATUS 0x" +
                            unchecked((uint)status).ToString("X8") + ").");
                    }

                    ulong bytesReturned = ioStatusBlock.Information.ToUInt64();
                    if (bytesReturned == 0 || bytesReturned > BufferLength)
                    {
                        throw new IOException(
                            "Resolved Windows directory enumeration returned an invalid entry buffer length.");
                    }

                    ulong entryOffset = 0;
                    while (true)
                    {
                        if (entryOffset + FileNamesInformationHeaderLength > bytesReturned)
                        {
                            throw new IOException("Resolved Windows directory enumeration returned a truncated entry.");
                        }
                        IntPtr entryPointer = IntPtr.Add(buffer, checked((int)entryOffset));
                        uint nextEntryOffset = unchecked((uint)Marshal.ReadInt32(entryPointer, 0));
                        uint fileNameLength = unchecked((uint)Marshal.ReadInt32(entryPointer, 8));
                        if ((fileNameLength & 1) != 0 ||
                            entryOffset + FileNamesInformationHeaderLength + fileNameLength > bytesReturned)
                        {
                            throw new IOException("Resolved Windows directory enumeration returned an invalid entry name.");
                        }
                        string entryName = Marshal.PtrToStringUni(
                            IntPtr.Add(entryPointer, FileNamesInformationHeaderLength),
                            checked((int)(fileNameLength / sizeof(char))));
                        if (!String.IsNullOrEmpty(entryName) && entryName != "." && entryName != "..")
                        {
                            yield return entryName;
                        }

                        if (nextEntryOffset == 0)
                        {
                            break;
                        }
                        if (nextEntryOffset < FileNamesInformationHeaderLength ||
                            entryOffset + nextEntryOffset + FileNamesInformationHeaderLength > bytesReturned)
                        {
                            throw new IOException("Resolved Windows directory enumeration returned an invalid entry offset.");
                        }
                        entryOffset += nextEntryOffset;
                    }
                }
            }
            finally
            {
                Marshal.FreeHGlobal(buffer);
            }
        }

        private static string GetFinalPath(SafeFileHandle handle)
        {
            StringBuilder buffer = new StringBuilder(512);
            uint length = GetFinalPathNameByHandleW(handle, buffer, (uint)buffer.Capacity, 0);
            if (length == 0)
            {
                throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve final path from an open handle.");
            }
            if (length >= buffer.Capacity)
            {
                buffer = new StringBuilder((int)length + 1);
                length = GetFinalPathNameByHandleW(handle, buffer, (uint)buffer.Capacity, 0);
                if (length == 0 || length >= buffer.Capacity)
                {
                    throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not resolve final path from an open handle.");
                }
            }
            return buffer.ToString();
        }

        private static uint GetFileAttributes(SafeFileHandle handle)
        {
            ByHandleFileInformation information;
            if (!GetFileInformationByHandle(handle, out information))
            {
                throw new Win32Exception(
                    Marshal.GetLastWin32Error(),
                    "Could not resolve file attributes from an open handle.");
            }
            return information.FileAttributes;
        }

        private static bool IsPathWithinRoot(string path, string root)
        {
            string normalizedPath = path.TrimEnd('\\', '/');
            string normalizedRoot = root.TrimEnd('\\', '/');
            return String.Equals(normalizedPath, normalizedRoot, StringComparison.OrdinalIgnoreCase) ||
                normalizedPath.StartsWith(normalizedRoot + "\\", StringComparison.OrdinalIgnoreCase);
        }

        // Pin the root-to-parent chain without delete sharing so a later junction replacement
        // cannot redirect a resolved rename after its containment check.
        private static List<SafeFileHandle> OpenPinnedDirectoryChain(
            string approvedRoot,
            string destinationDirectory)
        {
            string normalizedRoot = approvedRoot.TrimEnd('\\', '/');
            string normalizedDestination = destinationDirectory.TrimEnd('\\', '/');
            if (!IsPathWithinRoot(normalizedDestination, normalizedRoot))
            {
                throw new InvalidOperationException(
                    "Resolved Windows move destination is outside the approved root: " + destinationDirectory);
            }

            List<SafeFileHandle> handles = new List<SafeFileHandle>();
            try
            {
                string relativeDestination = normalizedDestination.Substring(normalizedRoot.Length).TrimStart('\\');
                string[] destinationSegments = relativeDestination.Split(
                    new[] { '\\' },
                    StringSplitOptions.RemoveEmptyEntries);
                uint rootAccess = FileTraverse | FileReadAttributes;
                if (destinationSegments.Length == 0)
                {
                    rootAccess |= FileAddFile | FileAddSubdirectory;
                }
                SafeFileHandle rootHandle = OpenDirectory(
                    normalizedRoot,
                    rootAccess,
                    FileShareRead | FileShareWrite);
                handles.Add(rootHandle);
                string resolvedRoot = GetFinalPath(rootHandle).TrimEnd('\\', '/');
                if (!String.Equals(resolvedRoot, normalizedRoot, StringComparison.OrdinalIgnoreCase))
                {
                    throw new InvalidOperationException(
                        "Resolved Windows move approved root changed while opening its pinned handle: " + resolvedRoot);
                }

                string currentPath = normalizedRoot;
                for (int index = 0; index < destinationSegments.Length; index++)
                {
                    string segment = destinationSegments[index];
                    currentPath = Path.Combine(currentPath, segment);
                    uint directoryAccess = FileTraverse | FileReadAttributes;
                    if (index == destinationSegments.Length - 1)
                    {
                        directoryAccess |= FileAddFile | FileAddSubdirectory;
                    }
                    SafeFileHandle directoryHandle = OpenDirectory(
                        currentPath,
                        directoryAccess,
                        FileShareRead | FileShareWrite);
                    handles.Add(directoryHandle);
                    if (!IsPathWithinRoot(GetFinalPath(directoryHandle), normalizedRoot))
                    {
                        throw new InvalidOperationException(
                            "Resolved Windows move destination parent is outside the approved root: " + currentPath);
                    }
                }

                return handles;
            }
            catch
            {
                foreach (SafeFileHandle handle in handles)
                {
                    handle.Dispose();
                }
                throw;
            }
        }

        public static string GetFinalPath(string path)
        {
            SafeFileHandle handle = OpenDirectory(path, FileReadAttributes);
            using (handle)
            {
                return GetFinalPath(handle);
            }
        }

        public static SafeFileHandle OpenDirectoryLease(
            string path,
            bool allowMove,
            bool denyWrite,
            bool openReparsePoint)
        {
            uint desiredAccess = FileTraverse | FileReadAttributes;
            if (allowMove)
            {
                desiredAccess |= FileDelete | FileListDirectory;
            }
            uint shareMode = denyWrite ? FileShareRead : FileShareRead | FileShareWrite;
            return OpenDirectory(path, desiredAccess, shareMode, openReparsePoint);
        }

        public static string GetDirectoryLeaseFinalPath(SafeFileHandle handle)
        {
            if (handle == null || handle.IsInvalid || handle.IsClosed)
            {
                throw new ArgumentException("Resolved Windows directory lease is not open.", "handle");
            }
            return GetFinalPath(handle);
        }

        public static uint GetDirectoryLeaseAttributes(SafeFileHandle handle)
        {
            if (handle == null || handle.IsInvalid || handle.IsClosed)
            {
                throw new ArgumentException("Resolved Windows directory lease is not open.", "handle");
            }
            return GetFileAttributes(handle);
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

        public static string MovePathWithinRoot(
            string sourcePath,
            string destinationDirectory,
            string destinationName,
            string approvedRoot)
        {
            using (SafeFileHandle sourceHandle = OpenDirectory(
                sourcePath,
                FileDelete | FileReadAttributes,
                FileShareRead | FileShareWrite))
            {
                return MoveLeasedPathWithinRoot(
                    sourceHandle,
                    destinationDirectory,
                    destinationName,
                    approvedRoot);
            }
        }

        public static string MoveLeasedPathWithinRoot(
            SafeFileHandle sourceHandle,
            string destinationDirectory,
            string destinationName,
            string approvedRoot)
        {
            if (sourceHandle == null || sourceHandle.IsInvalid || sourceHandle.IsClosed)
            {
                throw new ArgumentException("Resolved Windows source directory lease is not open.", "sourceHandle");
            }
            if (String.IsNullOrWhiteSpace(destinationName) ||
                !String.Equals(destinationName, Path.GetFileName(destinationName), StringComparison.Ordinal))
            {
                throw new ArgumentException("Resolved Windows move destination must be a plain file name.", "destinationName");
            }

            List<SafeFileHandle> destinationDirectoryHandles = OpenPinnedDirectoryChain(
                approvedRoot,
                destinationDirectory);
            try
            {
                SafeFileHandle destinationDirectoryHandle = destinationDirectoryHandles[
                    destinationDirectoryHandles.Count - 1];
                string resolvedDestinationDirectory = GetFinalPath(destinationDirectoryHandle);
                string resolvedSource = GetFinalPath(sourceHandle);
                if (!IsPathWithinRoot(resolvedSource, approvedRoot))
                {
                    throw new InvalidOperationException(
                        "Resolved Windows move source is outside the approved root: " + resolvedSource);
                }

                string resolvedDestinationPath = Path.Combine(
                    resolvedDestinationDirectory,
                    destinationName);
                byte[] destinationPathBytes = Encoding.Unicode.GetBytes(resolvedDestinationPath);
                int nameOffset = (int)Marshal.OffsetOf(typeof(FileRenameInformation), "FileNameLength") + sizeof(uint);
                int bufferAllocation = Math.Max(
                    Marshal.SizeOf(typeof(FileRenameInformation)),
                    nameOffset + destinationPathBytes.Length + sizeof(char));
                IntPtr renameInformation = Marshal.AllocHGlobal(bufferAllocation);
                try
                {
                    FileRenameInformation rename = new FileRenameInformation();
                    rename.ReplaceIfExists = false;
                    rename.RootDirectory = IntPtr.Zero;
                    rename.FileNameLength = (uint)destinationPathBytes.Length;
                    Marshal.StructureToPtr(rename, renameInformation, false);
                    Marshal.Copy(
                        destinationPathBytes,
                        0,
                        IntPtr.Add(renameInformation, nameOffset),
                        destinationPathBytes.Length);
                    Marshal.WriteInt16(
                        renameInformation,
                        nameOffset + destinationPathBytes.Length,
                        0);
                    if (!SetFileInformationByHandle(
                            sourceHandle,
                            FileRenameInfo,
                            renameInformation,
                            (uint)(nameOffset + destinationPathBytes.Length)))
                    {
                        throw new Win32Exception(
                            Marshal.GetLastWin32Error(),
                            "Could not move resolved Windows path into its opened destination directory.");
                    }
                    return resolvedDestinationPath;
                }
                finally
                {
                    Marshal.FreeHGlobal(renameInformation);
                }
            }
            finally
            {
                foreach (SafeFileHandle handle in destinationDirectoryHandles)
                {
                    handle.Dispose();
                }
            }
        }

        private static void MarkLeasedEntryForDeletion(SafeFileHandle handle)
        {
            if (handle == null || handle.IsInvalid || handle.IsClosed)
            {
                throw new ArgumentException("Resolved Windows directory lease is not open.", "handle");
            }
            FileDispositionInformation disposition = new FileDispositionInformation();
            disposition.DeleteFile = true;
            IntPtr dispositionInformation = Marshal.AllocHGlobal(
                Marshal.SizeOf(typeof(FileDispositionInformation)));
            try
            {
                Marshal.StructureToPtr(disposition, dispositionInformation, false);
                if (!SetFileInformationByHandle(
                        handle,
                        FileDispositionInfo,
                        dispositionInformation,
                        (uint)Marshal.SizeOf(typeof(FileDispositionInformation))))
                {
                    throw new Win32Exception(
                        Marshal.GetLastWin32Error(),
                        "Could not mark resolved Windows directory for deletion through its lease.");
                }
            }
            finally
            {
                Marshal.FreeHGlobal(dispositionInformation);
            }
        }

        public static void DeleteLeasedDirectoryContents(SafeFileHandle directoryHandle)
        {
            if (directoryHandle == null || directoryHandle.IsInvalid || directoryHandle.IsClosed)
            {
                throw new ArgumentException("Resolved Windows directory lease is not open.", "directoryHandle");
            }
            uint directoryAttributes = GetFileAttributes(directoryHandle);
            if ((directoryAttributes & FileAttributeDirectory) == 0 ||
                (directoryAttributes & FileAttributeReparsePoint) != 0)
            {
                throw new InvalidOperationException(
                    "Resolved Windows cleanup lease must name a non-reparse directory.");
            }

            // Enumerate and open every child relative to this pinned handle. A pathname-based
            // reopen would let a pre-existing writable child handle turn into a reparse point
            // after inspection and before the next traversal operation.
            foreach (string childName in EnumerateDirectoryEntryNames(directoryHandle))
            {
                using (SafeFileHandle childHandle = OpenDirectoryEntryRelative(
                    directoryHandle,
                    childName))
                {
                    uint childAttributes = GetFileAttributes(childHandle);
                    bool childIsDirectory = (childAttributes & FileAttributeDirectory) != 0;
                    bool childIsReparsePoint = (childAttributes & FileAttributeReparsePoint) != 0;
                    if (childIsDirectory && !childIsReparsePoint)
                    {
                        DeleteLeasedDirectoryContents(childHandle);
                    }
                    MarkLeasedEntryForDeletion(childHandle);
                }
            }
        }

        public static void DeleteLeasedEmptyDirectory(SafeFileHandle handle)
        {
            uint attributes = GetFileAttributes(handle);
            if ((attributes & FileAttributeDirectory) == 0 ||
                (attributes & FileAttributeReparsePoint) != 0)
            {
                throw new InvalidOperationException(
                    "Resolved Windows cleanup lease must name a non-reparse directory.");
            }
            MarkLeasedEntryForDeletion(handle);
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
    if ($Path -match '^\\\\\?\\[A-Za-z]:') {
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
        [string]$Destination,
        [string]$ApprovedRoot
    )

    Initialize-ZirconWindowsPathResolverNative
    if (-not [string]::IsNullOrWhiteSpace($ApprovedRoot)) {
        $destinationDirectory = [System.IO.Path]::GetDirectoryName($Destination)
        $destinationName = [System.IO.Path]::GetFileName($Destination)
        if ([string]::IsNullOrWhiteSpace($destinationDirectory) -or
            [string]::IsNullOrWhiteSpace($destinationName)) {
            throw "Resolved Windows move destination must name a child path: '$Destination'."
        }
        return [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::MovePathWithinRoot(
            $Source,
            $destinationDirectory,
            $destinationName,
            $ApprovedRoot)
    }
    [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::MovePath($Source, $Destination)
}

function Open-ZirconWindowsDirectoryLease {
    param(
        [Parameter(Mandatory)]
        [string]$Path,
        [Parameter(Mandatory)]
        [string]$ExpectedOperationalPath,
        [switch]$ForMove,
        [switch]$DenyWrite,
        [switch]$NoFollow
    )

    Initialize-ZirconWindowsPathResolverNative
    $lease = [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::OpenDirectoryLease(
        $Path,
        $ForMove.IsPresent,
        $DenyWrite.IsPresent,
        $NoFollow.IsPresent)
    try {
        $attributes = [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::GetDirectoryLeaseAttributes($lease)
        if ($NoFollow.IsPresent -and
            [bool]($attributes -band [System.IO.FileAttributes]::ReparsePoint)) {
            throw "Resolved Windows directory lease is a reparse point: '$Path'."
        }
        $resolvedPath = [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::GetDirectoryLeaseFinalPath($lease)
        if (-not [string]::Equals(
                $resolvedPath.TrimEnd('\\'),
                $ExpectedOperationalPath.TrimEnd('\\'),
                [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Resolved Windows directory lease changed physical path: '$resolvedPath'."
        }
        return $lease
    }
    catch {
        $lease.Dispose()
        throw
    }
}

function Move-ZirconWindowsLeasedPathWithinRoot {
    param(
        [Parameter(Mandatory)]
        [Microsoft.Win32.SafeHandles.SafeFileHandle]$SourceLease,
        [Parameter(Mandatory)]
        [string]$Destination,
        [Parameter(Mandatory)]
        [string]$ApprovedRoot
    )

    Initialize-ZirconWindowsPathResolverNative
    $destinationDirectory = [System.IO.Path]::GetDirectoryName($Destination)
    $destinationName = [System.IO.Path]::GetFileName($Destination)
    if ([string]::IsNullOrWhiteSpace($destinationDirectory) -or
        [string]::IsNullOrWhiteSpace($destinationName)) {
        throw "Resolved Windows move destination must name a child path: '$Destination'."
    }
    return [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::MoveLeasedPathWithinRoot(
        $SourceLease,
        $destinationDirectory,
        $destinationName,
        $ApprovedRoot)
}

function Remove-ZirconWindowsLeasedDirectory {
    param(
        [Parameter(Mandatory)]
        [Microsoft.Win32.SafeHandles.SafeFileHandle]$Lease
    )

    Initialize-ZirconWindowsPathResolverNative
    [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::DeleteLeasedEmptyDirectory($Lease)
}

function Remove-ZirconWindowsLeasedDirectoryTree {
    param(
        [Parameter(Mandatory)]
        [Microsoft.Win32.SafeHandles.SafeFileHandle]$Lease
    )

    Initialize-ZirconWindowsPathResolverNative
    [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::DeleteLeasedDirectoryContents($Lease)
    [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::DeleteLeasedEmptyDirectory($Lease)
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
        $displayPath = ConvertTo-ZirconWindowsDisplayPath -Path $Path
        throw "Windows paths must be drive-rooted, not drive-relative: '$displayPath'."
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
    $operationalExistingPath = [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::GetFinalPath($existingPath)
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
    return [ZirconEngine.WindowsPathResolver.NativeMethodsV4]::GetFileIdentity($resolution.OperationalPath)
}

Export-ModuleMember -Function Resolve-ZirconWindowsPath, Get-ZirconWindowsFileIdentity, Join-ZirconWindowsPath, Move-ZirconWindowsPath, Open-ZirconWindowsDirectoryLease, Move-ZirconWindowsLeasedPathWithinRoot, Remove-ZirconWindowsLeasedDirectory, Remove-ZirconWindowsLeasedDirectoryTree
