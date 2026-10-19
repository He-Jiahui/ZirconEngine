Set-StrictMode -Version Latest

# The wrapper owns this object in the AppDomain, outside every reimportable helper.
$script:MvpFixtureContextKey = 'ZirconEngine.MvpFixtureContext.V1'

function Initialize-MvpTestFixtureDependencies {
    # A cold helper import must not compile anything before finding a living owner.
    # The producer verifies physical TEMP/TMP first; this is the local bootstrap gate.
    foreach ($temporaryPath in @($env:TEMP, $env:TMP)) {
        if ([string]::IsNullOrWhiteSpace($temporaryPath) -or $temporaryPath -notmatch '^E:\\cargo-targets\\' -or
            $temporaryPath -match '~[0-9]' -or -not [IO.Path]::GetFullPath($temporaryPath).TrimEnd('\').Equals($temporaryPath.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase) -or
            -not [IO.Directory]::Exists($temporaryPath)) { throw 'Fixture compiler TEMP/TMP must be registered existing directories under the physical E cargo root.' }
        $ancestor = [IO.DirectoryInfo]::new($temporaryPath)
        while ($null -ne $ancestor) {
            if ([bool]($ancestor.Attributes -band [IO.FileAttributes]::ReparsePoint)) { throw 'Fixture compiler TEMP/TMP has a reparse-point ancestor.' }
            $ancestor = $ancestor.Parent
        }
    }
    Import-Module (Join-Path $PSScriptRoot '..\maintenance\WindowsPathResolver.psm1') -Force -ErrorAction Stop
    Import-Module (Join-Path $PSScriptRoot 'MvpArtifactStoragePolicy.psm1') -Force -ErrorAction Stop
    foreach ($temporaryPath in @($env:TEMP, $env:TMP)) {
        $resolution = Resolve-ZirconWindowsPath -Path $temporaryPath
        if (-not $resolution.DisplayPath.Equals($temporaryPath.TrimEnd('\'), [StringComparison]::OrdinalIgnoreCase)) { throw 'Fixture compiler TEMP/TMP differs from its physical final path.' }
    }
}

function Initialize-MvpTestFixtureNative {
    if ($null -ne ('ZirconEngine.MvpFixtures.V1.Owner' -as [type])) { return }
    Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using Microsoft.Win32.SafeHandles;

namespace ZirconEngine.MvpFixtures.V1
{
    public sealed class Instance
    {
        public string Path { get; internal set; }
        public string Leaf { get; internal set; }
        public string InstanceId { get; internal set; }
        public string State { get; internal set; }
        public string Identity { get; internal set; }
        public bool ReservedInJournal { get; internal set; }
        public SafeFileHandle Handle { get; internal set; }
    }

    public sealed class Journal : IDisposable
    {
        private readonly object gate = new object();
        private readonly FileStream stream;
        private bool disposed;
        public Journal(string path) { stream = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read); }
        public void Append(string json)
        {
            lock (gate)
            {
                if (disposed) throw new ObjectDisposedException("fixture journal");
                byte[] bytes = new UTF8Encoding(false).GetBytes(json + "\n");
                stream.Write(bytes, 0, bytes.Length);
                stream.Flush(true);
            }
        }
        public void Dispose() { lock (gate) { if (!disposed) { stream.Dispose(); disposed = true; } } }
    }

    public sealed class Owner : IDisposable
    {
        public readonly object Gate = new object();
        private readonly Dictionary<string, Instance> instances = new Dictionary<string, Instance>(StringComparer.OrdinalIgnoreCase);
        private readonly int pid;
        private readonly long birth;
        private bool disposed;
        public readonly string OwnerId = Guid.NewGuid().ToString("N");
        public readonly string RootPath;
        public readonly string ParentName;
        public readonly string ParentPath;
        public readonly string ParentDisplayPath;
        public readonly object Metadata;
        public readonly Journal Journal;
        public readonly bool OwnsJournal;
        public readonly bool Keep;
        public SafeFileHandle RootHandle { get; private set; }
        public SafeFileHandle ParentHandle { get; private set; }
        public string ParentIdentity { get; private set; }
        public string ParentState { get; private set; }
        public Owner(string root, string rootDisplayPath, string parentName, object metadata, Journal journal, bool ownsJournal, bool keep)
        {
            Native.CheckLeaf(parentName);
            Process process = Process.GetCurrentProcess();
            pid = process.Id;
            birth = process.StartTime.ToUniversalTime().Ticks;
            RootPath = root.TrimEnd('\\');
            ParentName = parentName;
            ParentPath = RootPath + "\\" + parentName;
            ParentDisplayPath = rootDisplayPath.TrimEnd('\\') + "\\" + parentName;
            Metadata = metadata;
            Journal = journal;
            OwnsJournal = ownsJournal;
            Keep = keep;
            ParentState = "Absent";
            RootHandle = Native.OpenRoot(RootPath);
        }
        public void AssertLive()
        {
            if (disposed || Process.GetCurrentProcess().Id != pid || Process.GetCurrentProcess().StartTime.ToUniversalTime().Ticks != birth)
                throw new InvalidOperationException("Fixture operation requires the original living process owner.");
        }
        public Instance Reserve(string path)
        {
            lock (Gate)
            {
                AssertLive();
                string prefix = ParentPath + "\\";
                if (!path.StartsWith(prefix, StringComparison.OrdinalIgnoreCase)) throw new InvalidOperationException("Fixture is outside its living owner's parent.");
                string leaf = path.Substring(prefix.Length);
                Native.CheckLeaf(leaf);
                if (instances.ContainsKey(path)) throw new InvalidOperationException("Fixture instance path is already recorded by this owner.");
                Instance instance = new Instance { Path = path, Leaf = leaf, InstanceId = Guid.NewGuid().ToString("N"), State = "Reserved" };
                instances.Add(path, instance);
                return instance;
            }
        }
        public void ConfirmReservation(Instance instance) { lock (Gate) { AssertOwned(instance); instance.ReservedInJournal = true; } }
        private void AssertOwned(Instance instance)
        {
            AssertLive();
            Instance actual;
            if (instance == null || !instances.TryGetValue(instance.Path, out actual) || !Object.ReferenceEquals(instance, actual))
                throw new InvalidOperationException("Fixture instance is not owned by this living context.");
        }
        public void Create(Instance instance)
        {
            lock (Gate)
            {
                AssertOwned(instance);
                if (!instance.ReservedInJournal || instance.State != "Reserved") throw new InvalidOperationException("Fixture must be durably Reserved before its first directory write.");
                if (ParentHandle == null)
                {
                    ParentHandle = Native.CreateDirectory(RootHandle, ParentName);
                    ParentState = "Active";
                    ParentIdentity = Native.Identity(ParentHandle);
                }
                Native.AssertIdentity(RootHandle, ParentName, ParentHandle, ParentIdentity);
                instance.Handle = Native.CreateDirectory(ParentHandle, instance.Leaf);
                // Keep the returned handle in the table even if an identity query fails.
                instance.Identity = Native.Identity(instance.Handle);
                Native.AssertIdentity(ParentHandle, instance.Leaf, instance.Handle, instance.Identity);
                instance.State = "Active";
            }
        }
        public Instance Lookup(string path)
        {
            lock (Gate)
            {
                AssertLive();
                Instance instance;
                if (!instances.TryGetValue(path, out instance)) throw new InvalidOperationException("Refusing to remove fixture without its living owner and recorded instance handle: " + path);
                return instance;
            }
        }
        public Instance[] Instances() { lock (Gate) { AssertLive(); Instance[] result = new Instance[instances.Count]; instances.Values.CopyTo(result, 0); return result; } }
        public void AssertDeletable(Instance instance)
        {
            lock (Gate)
            {
                AssertOwned(instance);
                if (instance.Handle == null || instance.Handle.IsClosed || instance.Handle.IsInvalid) throw new InvalidOperationException("Fixture no longer has its original live instance handle.");
                if (ParentIdentity == null) ParentIdentity = Native.Identity(ParentHandle);
                if (instance.Identity == null) instance.Identity = Native.Identity(instance.Handle);
                Native.AssertIdentity(RootHandle, ParentName, ParentHandle, ParentIdentity);
                Native.AssertIdentity(ParentHandle, instance.Leaf, instance.Handle, instance.Identity);
            }
        }
        public void SetState(Instance instance, string state) { lock (Gate) { AssertOwned(instance); instance.State = state; } }
        public void FinishRelease(Instance instance)
        {
            lock (Gate)
            {
                AssertOwned(instance);
                if (instance.Handle != null && !instance.Handle.IsClosed) instance.Handle.Dispose();
                if (Native.Exists(ParentHandle, instance.Leaf)) throw new IOException("Fixture still physically exists after its instance handle was disposed: " + instance.Path);
                instance.State = "Released";
            }
        }
        public void AssertReleasedAbsent(Instance instance)
        {
            lock (Gate)
            {
                AssertOwned(instance);
                using (SafeFileHandle parent = Native.TryOpen(RootHandle, ParentName))
                {
                    if (parent != null && Native.Exists(parent, instance.Leaf)) throw new InvalidOperationException("Refusing to remove a recreated fixture instance at a Released path: " + instance.Path);
                }
            }
        }
        public void FinishUncreatedOrClosed(Instance instance)
        {
            lock (Gate)
            {
                AssertOwned(instance);
                if (instance.Handle != null && !instance.Handle.IsClosed) throw new InvalidOperationException("An open instance must be released through its original handle.");
                using (SafeFileHandle parent = Native.TryOpen(RootHandle, ParentName))
                {
                    if (parent != null && Native.Exists(parent, instance.Leaf)) throw new IOException("An entry exists without an owned live handle; retain it without deletion: " + instance.Path);
                }
                instance.State = "Released";
            }
        }
        public bool HasPendingInstances()
        {
            foreach (Instance instance in Instances()) if (instance.State != "Released") return true;
            return false;
        }
        public void FinishParentRelease()
        {
            lock (Gate)
            {
                AssertLive();
                ParentHandle.Dispose();
                ParentHandle = null;
                if (Native.Exists(RootHandle, ParentName)) throw new IOException("Fixture parent still physically exists after its handle was disposed.");
                ParentIdentity = null;
                ParentState = "Released";
            }
        }
        public void FailParentCleanup() { lock (Gate) { AssertLive(); ParentState = "CleanupFailed"; } }
        public void ConfirmParentAbsent()
        {
            lock (Gate)
            {
                AssertLive();
                if (ParentHandle != null) throw new InvalidOperationException("An open parent must be released through its original handle.");
                if (Native.Exists(RootHandle, ParentName)) throw new IOException("Fixture parent remains present without its original live handle; retain without deleting or adopting it.");
                ParentState = "Released";
            }
        }
        public void Dispose()
        {
            lock (Gate)
            {
                if (disposed) return;
                List<Exception> failures = new List<Exception>();
                foreach (Instance instance in instances.Values)
                {
                    try { if (instance.Handle != null) instance.Handle.Dispose(); }
                    catch (Exception error) { failures.Add(error); }
                }
                try { if (ParentHandle != null) ParentHandle.Dispose(); }
                catch (Exception error) { failures.Add(error); }
                try { RootHandle.Dispose(); }
                catch (Exception error) { failures.Add(error); }
                disposed = true;
                try { if (OwnsJournal) Journal.Dispose(); }
                catch (Exception error) { failures.Add(error); }
                if (failures.Count != 0) throw new AggregateException("Could not dispose every fixture owner resource.", failures);
            }
        }
    }

    public static class Native
    {
        private const uint List = 1, AddDirectory = 4, ReadAttributes = 0x80, WriteAttributes = 0x100, Delete = 0x10000, Synchronize = 0x100000;
        private const uint Directory = 0x10, ReadOnly = 1, Reparse = 0x400;
        private const uint ShareRead = 1, ShareWrite = 2, ShareDelete = 4;
        private const uint Open = 1, Create = 2, DirectoryFile = 1, Synchronous = 0x20, OpenReparse = 0x200000;
        [StructLayout(LayoutKind.Sequential)] private struct UnicodeString { public ushort Length, MaximumLength; public IntPtr Buffer; }
        [StructLayout(LayoutKind.Sequential)] private struct ObjectAttributes { public uint Length; public IntPtr RootDirectory, ObjectName; public uint Attributes; public IntPtr SecurityDescriptor, SecurityQualityOfService; }
        [StructLayout(LayoutKind.Sequential)] private struct IoStatus { public IntPtr Status; public UIntPtr Information; }
        [StructLayout(LayoutKind.Sequential)] private struct FileInformation { public uint Attributes, CreationLow, CreationHigh, AccessLow, AccessHigh, WriteLow, WriteHigh, Volume, SizeHigh, SizeLow, Links, IndexHigh, IndexLow; }
        [StructLayout(LayoutKind.Sequential)] private struct FileBasic { public long Creation, Access, Write, Change; public uint Attributes; }
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern SafeFileHandle CreateFileW(string path, uint access, uint share, IntPtr security, uint disposition, uint flags, IntPtr template);
        [DllImport("kernel32.dll", SetLastError = true)] private static extern bool GetFileInformationByHandle(SafeFileHandle handle, out FileInformation information);
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)] private static extern uint GetFinalPathNameByHandleW(SafeFileHandle handle, StringBuilder path, uint size, uint flags);
        [DllImport("kernel32.dll", SetLastError = true)] private static extern bool SetFileInformationByHandle(SafeFileHandle handle, int infoClass, ref FileBasic information, uint size);
        [DllImport("ntdll.dll", ExactSpelling = true)] private static extern int NtCreateFile(out SafeFileHandle handle, uint access, ref ObjectAttributes attributes, out IoStatus status, IntPtr allocation, uint fileAttributes, uint share, uint disposition, uint options, IntPtr ea, uint eaLength);
        [DllImport("ntdll.dll", ExactSpelling = true)] private static extern int NtQueryDirectoryFile(SafeFileHandle handle, IntPtr eventHandle, IntPtr apc, IntPtr context, out IoStatus status, IntPtr buffer, uint length, int infoClass, [MarshalAs(UnmanagedType.U1)] bool single, IntPtr name, [MarshalAs(UnmanagedType.U1)] bool restart);
        public static void CheckLeaf(string leaf)
        {
            if (String.IsNullOrWhiteSpace(leaf) || leaf == "." || leaf == ".." || leaf.Length > 240 || leaf.EndsWith(".", StringComparison.Ordinal) || leaf.IndexOfAny(new char[] { '\\', '/', ':', '\0', '*', '?', '"', '<', '>', '|' }) >= 0 || leaf.Trim() != leaf)
                throw new ArgumentException("Fixture directory name must be a single ordinary leaf.", "leaf");
        }
        private static void CheckEntryName(string name)
        {
            // Existing children retain their actual names, including long components
            // and leading spaces. Only a plain relative entry can reach NtCreateFile.
            if (String.IsNullOrEmpty(name) || name == "." || name == ".." || name.IndexOfAny(new char[] { '\\', '/', ':', '\0' }) >= 0)
                throw new ArgumentException("Fixture entry name must be a plain child name.", "name");
        }
        private static FileInformation Info(SafeFileHandle handle)
        {
            FileInformation info;
            if (handle == null || handle.IsClosed || handle.IsInvalid) throw new InvalidOperationException("Fixture handle is not live.");
            if (!GetFileInformationByHandle(handle, out info)) throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not inspect fixture handle.");
            return info;
        }
        public static string Identity(SafeFileHandle handle)
        {
            FileInformation info = Info(handle);
            return info.Volume.ToString("X8") + ":" + info.IndexHigh.ToString("X8") + info.IndexLow.ToString("X8");
        }
        public static SafeFileHandle CreateResultLease(string path)
        {
            SafeFileHandle handle = CreateFileW(path, ReadAttributes, ShareRead | ShareWrite, IntPtr.Zero, 1, OpenReparse | 0x80, IntPtr.Zero);
            if (handle.IsInvalid)
            {
                int error = Marshal.GetLastWin32Error();
                handle.Dispose();
                throw new Win32Exception(error, "Could not exclusively create the registered Pester result file.");
            }
            return handle;
        }
        public static void AssertResultIdentity(string path, SafeFileHandle held)
        {
            using (SafeFileHandle actual = CreateFileW(path, ReadAttributes, ShareRead | ShareWrite | ShareDelete, IntPtr.Zero, 3, OpenReparse, IntPtr.Zero))
            {
                if ((Info(actual).Attributes & (Directory | Reparse)) != 0 || !String.Equals(Identity(actual), Identity(held), StringComparison.Ordinal)) throw new IOException("Pester result no longer names its exclusively created file instance.");
            }
        }
        public static SafeFileHandle OpenRoot(string path)
        {
            SafeFileHandle handle = CreateFileW(path, List | AddDirectory | ReadAttributes | WriteAttributes | Synchronize, ShareRead | ShareWrite, IntPtr.Zero, 3, 0x02000000 | OpenReparse, IntPtr.Zero);
            try
            {
                FileInformation info = Info(handle);
                if ((info.Attributes & (Directory | Reparse)) != Directory) throw new IOException("Fixture storage root must be a physical non-reparse directory.");
                StringBuilder final = new StringBuilder(32768);
                uint length = GetFinalPathNameByHandleW(handle, final, (uint)final.Capacity, 0);
                if (length == 0 || length >= final.Capacity || !String.Equals(final.ToString().TrimEnd('\\'), path.TrimEnd('\\'), StringComparison.OrdinalIgnoreCase)) throw new IOException("Fixture storage root changed physical path.");
                return handle;
            }
            catch { handle.Dispose(); throw; }
        }
        private static SafeFileHandle Relative(SafeFileHandle parent, string leaf, bool create, bool mutation, bool absenceAllowed)
        {
            if (create) CheckLeaf(leaf); else CheckEntryName(leaf);
            bool pinned = false;
            IntPtr name = IntPtr.Zero, descriptor = IntPtr.Zero;
            SafeFileHandle handle = null;
            try
            {
                lock (parent)
                {
                    parent.DangerousAddRef(ref pinned);
                    name = Marshal.StringToHGlobalUni(leaf);
                    UnicodeString unicode = new UnicodeString { Length = checked((ushort)(leaf.Length * 2)), MaximumLength = checked((ushort)(leaf.Length * 2 + 2)), Buffer = name };
                    descriptor = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(UnicodeString)));
                    Marshal.StructureToPtr(unicode, descriptor, false);
                    ObjectAttributes attributes = new ObjectAttributes { Length = (uint)Marshal.SizeOf(typeof(ObjectAttributes)), RootDirectory = parent.DangerousGetHandle(), ObjectName = descriptor, Attributes = 0x40 };
                    IoStatus io;
                    uint access = mutation ? List | ReadAttributes | WriteAttributes | Synchronize | (create ? AddDirectory | Delete : 0) : ReadAttributes | Synchronize;
                    int status = NtCreateFile(out handle, access, ref attributes, out io, IntPtr.Zero, create ? Directory : 0, mutation ? ShareRead : ShareRead | ShareWrite | ShareDelete, create ? Create : Open, Synchronous | OpenReparse | (create ? DirectoryFile : 0), IntPtr.Zero, 0);
                    if (status != 0 || handle == null || handle.IsInvalid)
                    {
                        if (handle != null) { handle.Dispose(); handle = null; }
                        uint code = unchecked((uint)status);
                        if (absenceAllowed && (code == 0xC0000034 || code == 0xC000003A)) return null;
                        throw new IOException("Fixture handle-relative " + (create ? "exclusive create" : "open") + " failed (NTSTATUS 0x" + code.ToString("X8") + ").");
                    }
                    SafeFileHandle result = handle;
                    handle = null;
                    return result;
                }
            }
            finally
            {
                if (handle != null) handle.Dispose();
                if (descriptor != IntPtr.Zero) Marshal.FreeHGlobal(descriptor);
                if (name != IntPtr.Zero) Marshal.FreeHGlobal(name);
                if (pinned) parent.DangerousRelease();
            }
        }
        public static SafeFileHandle CreateDirectory(SafeFileHandle parent, string leaf) { return Relative(parent, leaf, true, true, false); }
        public static SafeFileHandle TryOpen(SafeFileHandle parent, string leaf) { return Relative(parent, leaf, false, false, true); }
        public static bool Exists(SafeFileHandle parent, string leaf) { using (SafeFileHandle handle = TryOpen(parent, leaf)) { return handle != null; } }
        public static void AssertIdentity(SafeFileHandle parent, string leaf, SafeFileHandle held, string identity)
        {
            if (!String.Equals(Identity(held), identity, StringComparison.Ordinal)) throw new IOException("Fixture instance handle identity changed.");
            using (SafeFileHandle actual = TryOpen(parent, leaf))
            {
                if (actual == null || !String.Equals(Identity(actual), identity, StringComparison.Ordinal) || (Info(actual).Attributes & (Directory | Reparse)) != Directory)
                    throw new IOException("Fixture pathname no longer names its original pinned directory instance.");
            }
        }
        private static List<string> Names(SafeFileHandle directory)
        {
            List<string> names = new List<string>();
            const int capacity = 65536;
            IntPtr buffer = Marshal.AllocHGlobal(capacity);
            try
            {
                bool restart = true;
                while (true)
                {
                    IoStatus io;
                    int status = NtQueryDirectoryFile(directory, IntPtr.Zero, IntPtr.Zero, IntPtr.Zero, out io, buffer, capacity, 12, false, IntPtr.Zero, restart);
                    restart = false;
                    if (unchecked((uint)status) == 0x80000006) return names;
                    if (status != 0) throw new IOException("Fixture no-follow enumeration failed (NTSTATUS 0x" + unchecked((uint)status).ToString("X8") + ").");
                    ulong bytes = io.Information.ToUInt64();
                    if (bytes < 12 || bytes > capacity) throw new IOException("Invalid fixture directory enumeration length.");
                    int offset = 0;
                    while (true)
                    {
                        if ((ulong)offset + 12 > bytes) throw new IOException("Truncated fixture directory entry.");
                        IntPtr entry = IntPtr.Add(buffer, offset);
                        uint next = unchecked((uint)Marshal.ReadInt32(entry, 0));
                        uint length = unchecked((uint)Marshal.ReadInt32(entry, 8));
                        if ((length & 1) != 0 || (ulong)offset + 12 + length > bytes) throw new IOException("Invalid fixture directory entry name.");
                        string name = Marshal.PtrToStringUni(IntPtr.Add(entry, 12), checked((int)(length / 2)));
                        if (name != "." && name != "..") { CheckEntryName(name); names.Add(name); }
                        if (next == 0) break;
                        if (next < 12 || (ulong)offset + next + 12 > bytes) throw new IOException("Invalid fixture directory entry offset.");
                        offset = checked(offset + (int)next);
                    }
                }
            }
            finally { Marshal.FreeHGlobal(buffer); }
        }
        public static void ClearReadOnlyTree(SafeFileHandle directory)
        {
            FileInformation root = Info(directory);
            if ((root.Attributes & (Directory | Reparse)) != Directory) throw new IOException("Fixture traversal requires a pinned non-reparse directory.");
            foreach (string name in Names(directory))
            {
                using (SafeFileHandle child = Relative(directory, name, false, true, false))
                {
                    uint attributes = Info(child).Attributes;
                    if ((attributes & (Directory | Reparse)) == Directory) ClearReadOnlyTree(child);
                    ClearReadOnly(child, attributes);
                }
            }
            ClearReadOnly(directory, root.Attributes);
        }
        private static void ClearReadOnly(SafeFileHandle handle, uint attributes)
        {
            if ((attributes & ReadOnly) == 0) return;
            uint writable = attributes & ~(ReadOnly | Directory | Reparse);
            FileBasic basic = new FileBasic { Attributes = writable == 0 ? 0x80 : writable };
            if (!SetFileInformationByHandle(handle, 0, ref basic, (uint)Marshal.SizeOf(typeof(FileBasic)))) throw new Win32Exception(Marshal.GetLastWin32Error(), "Could not clear read-only fixture attributes through its no-follow handle.");
        }
    }
}
'@ -ErrorAction Stop
}

function Write-MvpFixtureJournal {
    param([Parameter(Mandatory)]$Context, [Parameter(Mandatory)][Collections.IDictionary]$Event)
    $Context.AssertLive()
    $record = [ordered]@{ schemaVersion = 1; utc = [DateTime]::UtcNow.ToString('o'); ownerId = $Context.OwnerId }
    foreach ($key in $Event.Keys) { $record[$key] = $Event[$key] }
    $Context.Journal.Append(($record | ConvertTo-Json -Depth 12 -Compress))
}

function Complete-MvpFixtureCleanupErrors {
    [CmdletBinding()]
    param(
        [AllowNull()][System.Management.Automation.ErrorRecord]$PrimaryError,
        [Parameter(Mandatory)][AllowEmptyCollection()][object[]]$Failures
    )
    if ($Failures.Count -eq 0) { return }
    $errorToReport = if ($null -ne $PrimaryError) { $PrimaryError } else { $Failures[0] }
    $allFailures = [Collections.Generic.List[object]]::new()
    foreach ($failure in $errorToReport.Exception.Data['MvpFixtureCleanupFailures']) { $allFailures.Add($failure) }
    foreach ($failure in $Failures) { $allFailures.Add($failure) }
    $errorToReport.Exception.Data['MvpFixtureCleanupFailures'] = $allFailures.ToArray()
    if ($null -eq $PrimaryError) { $PSCmdlet.ThrowTerminatingError($errorToReport) }
}

function New-MvpTestFixtureContext {
    [CmdletBinding(DefaultParameterSetName = 'Wrapper')]
    param(
        [Parameter(Mandatory, ParameterSetName = 'Wrapper')][Collections.IDictionary]$Metadata,
        [Parameter(Mandatory, ParameterSetName = 'Wrapper')][string]$JournalPath,
        [Parameter(Mandatory, ParameterSetName = 'TestOwner')]$ParentContext,
        [Parameter(ParameterSetName = 'TestOwner')][string]$ParentName
    )
    Initialize-MvpTestFixtureDependencies
    Initialize-MvpTestFixtureNative
    $journal = $null
    $context = $null
    $ownsJournal = $PSCmdlet.ParameterSetName -eq 'Wrapper'
    try {
        $artifactRoot = Get-MvpArtifactStorageDefaultRootPath -CapabilityClass 'windows-local-artifact'
        $fixtureBase = Join-Path $artifactRoot 'zircon-local'
        $resolution = Resolve-ZirconWindowsPath -Path $fixtureBase
        if (-not $resolution.DisplayPath.Equals($fixtureBase, [StringComparison]::OrdinalIgnoreCase)) { throw 'Fixture storage root is an alias.' }
        if ($ownsJournal) {
            foreach ($field in @('taskId', 'requestId', 'attemptId', 'driverId', 'component', 'commandId', 'invocationId', 'suite')) {
                if (-not $Metadata.Contains($field) -or [string]$Metadata[$field] -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$') { throw "Fixture owner metadata is missing its bounded $field." }
            }
            $expectedJournal = Join-Path $artifactRoot ("zircon-local\reports\{0}\s78-fixture-owner\{1}\fixture-owner-{1}.jsonl" -f $Metadata['taskId'], $Metadata['invocationId'])
            $journalResolution = Resolve-ZirconWindowsPath -Path $JournalPath
            if (-not $journalResolution.DisplayPath.Equals($expectedJournal, [StringComparison]::OrdinalIgnoreCase)) { throw 'Fixture owner journal is outside its registered task/invocation report path.' }
            $journal = [ZirconEngine.MvpFixtures.V1.Journal]::new($journalResolution.OperationalPath)
            $ParentName = "mvp-test-fixtures-$PID"
        }
        else {
            $ParentContext.AssertLive()
            $journal = $ParentContext.Journal
            $Metadata = $ParentContext.Metadata
            if ([string]::IsNullOrWhiteSpace($ParentName)) { $ParentName = "mvp-test-fixtures-$PID-$([guid]::NewGuid().ToString('N'))" }
        }
        $context = [ZirconEngine.MvpFixtures.V1.Owner]::new($resolution.OperationalPath, $resolution.DisplayPath, $ParentName, $Metadata, $journal, $ownsJournal, ($env:ZIRCON_KEEP_MVP_ACCEPTANCE_FIXTURE -eq '1'))
        $process = [Diagnostics.Process]::GetCurrentProcess()
        Write-MvpFixtureJournal $context ([ordered]@{
            kind = 'owner'; state = 'Active'; metadata = $Metadata; testOwner = (-not $ownsJournal)
            processId = $PID; processStartUtcTicks = $process.StartTime.ToUniversalTime().Ticks.ToString([Globalization.CultureInfo]::InvariantCulture)
            processStartTicksEpoch = '0001-01-01T00:00:00Z'; processStartFileTimeUtc = $process.StartTime.ToUniversalTime().ToFileTimeUtc().ToString([Globalization.CultureInfo]::InvariantCulture)
            keep = $context.Keep; parentPath = $context.ParentDisplayPath
        })
        return $context
    }
    catch {
        $primaryError = $_
        $failures = [Collections.Generic.List[object]]::new()
        try {
            if ($null -ne $context) { $context.Dispose() }
            elseif ($ownsJournal -and $null -ne $journal) { $journal.Dispose() }
        }
        catch { $failures.Add($_) }
        Complete-MvpFixtureCleanupErrors -PrimaryError $primaryError -Failures $failures.ToArray()
        throw
    }
}

function Get-MvpTestFixtureContext {
    $context = [AppDomain]::CurrentDomain.GetData($script:MvpFixtureContextKey)
    $ownerType = 'ZirconEngine.MvpFixtures.V1.Owner' -as [type]
    if ($null -eq $context -or $null -eq $ownerType -or -not $ownerType.IsInstanceOfType($context)) { throw 'MVP fixtures require Invoke-MvpTestSuite.ps1 and its living process owner.' }
    $context.AssertLive()
    Initialize-MvpTestFixtureDependencies
    return $context
}

function Assert-MvpOwnedFixturePath {
    param([Parameter(Mandatory)]$Context, [Parameter(Mandatory)][string]$Path)
    $Context.AssertLive()
    $resolution = Resolve-ZirconWindowsPath -Path $Path
    Resolve-MvpArtifactStorageRootPath -Path $resolution.DisplayPath -CapabilityClass 'windows-local-artifact' | Out-Null
    Resolve-MvpArtifactStoragePath -Path $resolution.DisplayPath -NamespaceId 'mvp-test-fixtures' | Out-Null
    $parent = $Context.ParentDisplayPath.TrimEnd('\') + '\'
    if (-not $resolution.DisplayPath.StartsWith($parent, [StringComparison]::OrdinalIgnoreCase) -or
        $resolution.DisplayPath.Substring($parent.Length) -notmatch '^[A-Za-z0-9][A-Za-z0-9._-]*-[0-9a-f]{32}$') {
        throw "Refusing to remove fixture outside the approved MVP fixture root of its living owner: $($resolution.DisplayPath)"
    }
    return $resolution
}

function New-MvpOwnedFixture {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)]$Context,
        [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._-]{0,119}$')][string]$Prefix,
        [string]$ExactLeaf
    )
    $leaf = if ([string]::IsNullOrEmpty($ExactLeaf)) { "$Prefix-$([guid]::NewGuid().ToString('N'))" } else { $ExactLeaf }
    $path = Join-ZirconWindowsPath -Path $Context.ParentPath -ChildPath $leaf
    $resolution = Assert-MvpOwnedFixturePath -Context $Context -Path $path
    [Threading.Monitor]::Enter($Context.Gate)
    $instance = $null
    $primaryError = $null
    try {
        $instance = $Context.Reserve($resolution.OperationalPath)
        Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'fixture'; state = 'Reserved'; path = $resolution.DisplayPath; instanceId = $instance.InstanceId })
        $Context.ConfirmReservation($instance)
        $Context.Create($instance)
        Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'fixture'; state = 'Active'; path = $resolution.DisplayPath; instanceId = $instance.InstanceId; fileIdentity = $instance.Identity })
        return $resolution.DisplayPath
    }
    catch {
        $primaryError = $_
        if ($null -ne $instance) {
            try { Remove-MvpOwnedFixture -Context $Context -Path $resolution.DisplayPath }
            catch { $primaryError.Exception.Data['MvpFixtureCleanupError'] = $_; Write-Warning -Message $_.Exception.Message -WarningAction Continue }
        }
        $PSCmdlet.ThrowTerminatingError($primaryError)
    }
    finally { [Threading.Monitor]::Exit($Context.Gate) }
}

function Remove-MvpOwnedFixture {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Context, [Parameter(Mandatory)][string]$Path)
    $resolution = Assert-MvpOwnedFixturePath -Context $Context -Path $Path
    [Threading.Monitor]::Enter($Context.Gate)
    try {
        $instance = $Context.Lookup($resolution.OperationalPath)
        if ($instance.State -eq 'Released') { $Context.AssertReleasedAbsent($instance); Remove-MvpOwnedFixtureParent -Context $Context; return }
        if ($Context.Keep) {
            $Context.SetState($instance, 'Retained')
            Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'fixture'; state = 'Retained'; reason = 'KEEP'; path = $resolution.DisplayPath; instanceId = $instance.InstanceId; fileIdentity = $instance.Identity })
            return
        }
        try {
            if ($null -ne $instance.Handle -and -not $instance.Handle.IsClosed) {
                $Context.AssertDeletable($instance)
                [ZirconEngine.MvpFixtures.V1.Native]::ClearReadOnlyTree($instance.Handle)
                Remove-ZirconWindowsLeasedDirectoryTree -Lease $instance.Handle
                $Context.FinishRelease($instance)
            }
            else {
                # No handle permits only an absence check, never pathname cleanup or adoption.
                $Context.FinishUncreatedOrClosed($instance)
            }
            Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'fixture'; state = 'Released'; path = $resolution.DisplayPath; instanceId = $instance.InstanceId; fileIdentity = $instance.Identity; physicalAbsent = $true })
        }
        catch {
            $primaryError = $_
            $Context.SetState($instance, 'CleanupFailed')
            try { Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'fixture'; state = 'CleanupFailed'; path = $resolution.DisplayPath; instanceId = $instance.InstanceId; fileIdentity = $instance.Identity; error = $primaryError.Exception.Message }) }
            catch { $primaryError.Exception.Data['MvpFixtureJournalError'] = $_ }
            $PSCmdlet.ThrowTerminatingError($primaryError)
        }
        Remove-MvpOwnedFixtureParent -Context $Context
    }
    finally { [Threading.Monitor]::Exit($Context.Gate) }
}

function Remove-MvpOwnedFixtureParent {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Context)
    if ($Context.HasPendingInstances()) { return }
    try {
        if ($null -eq $Context.ParentHandle) {
            if ($Context.ParentState -in @('Absent', 'Released')) { return }
            $Context.ConfirmParentAbsent()
        }
        else {
            [ZirconEngine.MvpFixtures.V1.Native]::AssertIdentity($Context.RootHandle, $Context.ParentName, $Context.ParentHandle, $Context.ParentIdentity)
            Remove-ZirconWindowsLeasedDirectory -Lease $Context.ParentHandle
            $Context.FinishParentRelease()
        }
        Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'parent'; state = 'Released'; path = $Context.ParentDisplayPath; physicalAbsent = $true })
    }
    catch {
        $primaryError = $_
        $Context.FailParentCleanup()
        try { Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'parent'; state = 'CleanupFailed'; path = $Context.ParentDisplayPath; error = $primaryError.Exception.Message }) }
        catch { $primaryError.Exception.Data['MvpFixtureJournalError'] = $_ }
        $PSCmdlet.ThrowTerminatingError($primaryError)
    }
}

function Close-MvpTestFixtureContext {
    [CmdletBinding()]
    param([Parameter(Mandatory)]$Context, [AllowNull()][System.Management.Automation.ErrorRecord]$PrimaryError)
    $failures = [Collections.Generic.List[object]]::new()
    try {
        foreach ($instance in $Context.Instances()) {
            if ($instance.State -eq 'Released') { continue }
            try { Remove-MvpOwnedFixture -Context $Context -Path $instance.Path }
            catch { $failures.Add($_); Write-Warning -Message $_.Exception.Message -WarningAction Continue }
        }
        try { Remove-MvpOwnedFixtureParent -Context $Context }
        catch { $failures.Add($_); Write-Warning -Message $_.Exception.Message -WarningAction Continue }
        Write-MvpFixtureJournal $Context ([ordered]@{ kind = 'owner'; state = $(if ($failures.Count -gt 0) { 'CleanupFailed' } elseif ($Context.HasPendingInstances()) { 'Retained' } else { 'Released' }); parentState = $Context.ParentState; parentRetained = ($Context.ParentState -notin @('Absent', 'Released')); pending = @($Context.Instances() | Where-Object { $_.State -ne 'Released' } | ForEach-Object { [ordered]@{ path = $_.Path; state = $_.State; instanceId = $_.InstanceId; fileIdentity = $_.Identity } }) })
    }
    catch { $failures.Add($_) }
    finally {
        try { $Context.Dispose() }
        catch { $failures.Add($_) }
    }
    Complete-MvpFixtureCleanupErrors -PrimaryError $PrimaryError -Failures $failures.ToArray()
}

Export-ModuleMember -Function @('New-MvpTestFixtureContext', 'Get-MvpTestFixtureContext', 'New-MvpOwnedFixture', 'Remove-MvpOwnedFixture', 'Close-MvpTestFixtureContext', 'Write-MvpFixtureJournal')
