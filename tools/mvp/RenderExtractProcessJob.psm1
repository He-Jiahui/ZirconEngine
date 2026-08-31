Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$outputCaptureModule = Join-Path $PSScriptRoot 'MvpProcessOutputCapture.psm1'
Import-Module $outputCaptureModule -Force -ErrorAction Stop

if ($null -eq ('Zircon.Tools.RenderExtractProcessJob' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.Collections;
using System.Collections.Generic;
using System.Collections.Specialized;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using Microsoft.Win32.SafeHandles;

namespace Zircon.Tools
{
    public sealed class RenderExtractSuspendedProcess : IDisposable
    {
        public Process Process { get; private set; }
        public StreamReader StandardOutput { get; private set; }
        public StreamReader StandardError { get; private set; }
        private IntPtr nativeProcessHandle;
        private IntPtr nativeThreadHandle;

        internal RenderExtractSuspendedProcess(
            Process process,
            StreamReader stdout,
            StreamReader stderr,
            IntPtr nativeProcessHandle,
            IntPtr nativeThreadHandle)
        {
            Process = process;
            StandardOutput = stdout;
            StandardError = stderr;
            this.nativeProcessHandle = nativeProcessHandle;
            this.nativeThreadHandle = nativeThreadHandle;
        }

        public int? TryGetExitCode()
        {
            if (nativeProcessHandle == IntPtr.Zero)
            {
                throw new ObjectDisposedException("RenderExtractSuspendedProcess");
            }
            uint exitCode;
            if (!GetExitCodeProcess(nativeProcessHandle, out exitCode))
            {
                return null;
            }
            return exitCode == 259 ? (int?)null : unchecked((int)exitCode);
        }

        public void Resume()
        {
            if (nativeThreadHandle == IntPtr.Zero)
            {
                throw new ObjectDisposedException("RenderExtractSuspendedProcess");
            }
            if (ResumeThread(nativeThreadHandle) == UInt32.MaxValue)
            {
                throw Win32("Could not resume the job-bound render-extract product process.");
            }
            CloseHandle(nativeThreadHandle);
            nativeThreadHandle = IntPtr.Zero;
        }

        public void Dispose()
        {
            if (nativeThreadHandle != IntPtr.Zero) {
                CloseHandle(nativeThreadHandle);
                nativeThreadHandle = IntPtr.Zero;
            }
            if (StandardOutput != null) { StandardOutput.Dispose(); StandardOutput = null; }
            if (StandardError != null) { StandardError.Dispose(); StandardError = null; }
            if (Process != null) { Process.Dispose(); Process = null; }
            if (nativeProcessHandle != IntPtr.Zero) {
                CloseHandle(nativeProcessHandle);
                nativeProcessHandle = IntPtr.Zero;
            }
        }

        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool GetExitCodeProcess(IntPtr process, out uint exitCode);

        [DllImport("kernel32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool CloseHandle(IntPtr handle);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern uint ResumeThread(IntPtr thread);

        private static Exception Win32(string message)
        {
            return new System.ComponentModel.Win32Exception(Marshal.GetLastWin32Error(), message);
        }
    }

    public sealed class RenderExtractProcessJobLimitSnapshot
    {
        public uint ActiveProcessLimit { get; private set; }
        public ulong JobMemoryLimitBytes { get; private set; }
        public uint CpuRateControlFlags { get; private set; }
        public uint CpuRatePerTenThousand { get; private set; }

        internal RenderExtractProcessJobLimitSnapshot(
            uint activeProcessLimit,
            ulong jobMemoryLimitBytes,
            uint cpuRateControlFlags,
            uint cpuRatePerTenThousand)
        {
            ActiveProcessLimit = activeProcessLimit;
            JobMemoryLimitBytes = jobMemoryLimitBytes;
            CpuRateControlFlags = cpuRateControlFlags;
            CpuRatePerTenThousand = cpuRatePerTenThousand;
        }
    }

    public sealed class RenderExtractProcessJob : IDisposable
    {
        private const uint JobObjectExtendedLimitInformation = 9;
        private const uint JobObjectBasicAccountingInformation = 1;
        private const uint JobObjectCpuRateControlInformation = 15;
        private const uint JobObjectLimitActiveProcess = 0x00000008;
        private const uint JobObjectLimitJobMemory = 0x00000200;
        private const uint JobObjectLimitKillOnJobClose = 0x00002000;
        private const uint JobObjectCpuRateControlEnable = 0x00000001;
        private const uint JobObjectCpuRateControlHardCap = 0x00000004;
        private const uint MaximumCpuRatePerTenThousand = 10000;
        private const uint CreateSuspended = 0x00000004;
        private const uint CreateNoWindow = 0x08000000;
        private const uint CreateUnicodeEnvironment = 0x00000400;
        private const uint StartfUseStdHandles = 0x00000100;
        private const uint HandleFlagInherit = 0x00000001;
        private const uint WaitObject0 = 0;
        private const uint WaitFailed = 0xFFFFFFFF;

        private IntPtr handle;

        public RenderExtractProcessJob()
            : this(0, 0, 0)
        {
        }

        public RenderExtractProcessJob(
            uint maximumActiveProcessCount,
            ulong maximumJobMemoryBytes,
            uint maximumCpuRatePerTenThousand)
        {
            if (maximumCpuRatePerTenThousand > MaximumCpuRatePerTenThousand)
            {
                throw new ArgumentOutOfRangeException(
                    "maximumCpuRatePerTenThousand",
                    "The process job CPU rate must be between 1 and 10,000 when configured.");
            }
            if (IntPtr.Size == 4 && maximumJobMemoryBytes > UInt32.MaxValue)
            {
                throw new ArgumentOutOfRangeException(
                    "maximumJobMemoryBytes",
                    "The process job memory limit exceeds the current platform address width.");
            }
            handle = CreateJobObjectW(IntPtr.Zero, null);
            if (handle == IntPtr.Zero)
            {
                throw Win32("Could not create the render-extract process job.");
            }
            var limits = new JobObjectExtendedLimitInformationData();
            limits.BasicLimitInformation.LimitFlags = JobObjectLimitKillOnJobClose;
            if (maximumActiveProcessCount != 0)
            {
                limits.BasicLimitInformation.LimitFlags |= JobObjectLimitActiveProcess;
                limits.BasicLimitInformation.ActiveProcessLimit = maximumActiveProcessCount;
            }
            if (maximumJobMemoryBytes != 0)
            {
                limits.BasicLimitInformation.LimitFlags |= JobObjectLimitJobMemory;
                limits.JobMemoryLimit = new UIntPtr(maximumJobMemoryBytes);
            }
            int size = Marshal.SizeOf(typeof(JobObjectExtendedLimitInformationData));
            IntPtr buffer = Marshal.AllocHGlobal(size);
            try
            {
                Marshal.StructureToPtr(limits, buffer, false);
                if (!SetInformationJobObject(handle, JobObjectExtendedLimitInformation, buffer, (uint)size))
                {
                    throw Win32("Could not configure the render-extract process job.");
                }
                if (maximumCpuRatePerTenThousand != 0)
                {
                    var cpuRate = new JobObjectCpuRateControlInformationData();
                    cpuRate.ControlFlags = JobObjectCpuRateControlEnable | JobObjectCpuRateControlHardCap;
                    cpuRate.CpuRate = maximumCpuRatePerTenThousand;
                    int cpuRateSize = Marshal.SizeOf(typeof(JobObjectCpuRateControlInformationData));
                    IntPtr cpuRateBuffer = Marshal.AllocHGlobal(cpuRateSize);
                    try
                    {
                        Marshal.StructureToPtr(cpuRate, cpuRateBuffer, false);
                        if (!SetInformationJobObject(handle, JobObjectCpuRateControlInformation, cpuRateBuffer, (uint)cpuRateSize))
                        {
                            throw Win32("Could not configure the render-extract process job CPU rate.");
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(cpuRateBuffer);
                    }
                }
            }
            catch
            {
                CloseHandle(handle);
                handle = IntPtr.Zero;
                throw;
            }
            finally
            {
                Marshal.FreeHGlobal(buffer);
            }
        }

        public RenderExtractSuspendedProcess StartAssigned(ProcessStartInfo startInfo)
        {
            var assignedProcess = StartSuspendedAssigned(startInfo);
            try
            {
                assignedProcess.Resume();
                return assignedProcess;
            }
            catch
            {
                assignedProcess.Dispose();
                throw;
            }
        }

        public RenderExtractSuspendedProcess StartSuspendedAssigned(ProcessStartInfo startInfo)
        {
            if (startInfo == null) { throw new ArgumentNullException("startInfo"); }
            EnsureOpen();
            IntPtr stdoutRead = IntPtr.Zero;
            IntPtr stdoutWrite = IntPtr.Zero;
            IntPtr stderrRead = IntPtr.Zero;
            IntPtr stderrWrite = IntPtr.Zero;
            IntPtr environment = IntPtr.Zero;
            ProcessInformation processInformation = new ProcessInformation();
            Process process = null;
            StreamReader stdout = null;
            StreamReader stderr = null;
            try
            {
                CreateOutputPipe(out stdoutRead, out stdoutWrite);
                CreateOutputPipe(out stderrRead, out stderrWrite);
                environment = BuildEnvironmentBlock(startInfo.EnvironmentVariables);
                var startup = new StartupInfo();
                startup.Cb = (uint)Marshal.SizeOf(typeof(StartupInfo));
                startup.Flags = StartfUseStdHandles;
                startup.StdInput = GetStdHandle(-10);
                startup.StdOutput = stdoutWrite;
                startup.StdError = stderrWrite;
                string commandLine = Quote(startInfo.FileName);
                if (!String.IsNullOrWhiteSpace(startInfo.Arguments))
                {
                    commandLine += " " + startInfo.Arguments;
                }
                var mutableCommandLine = new StringBuilder(commandLine);
                if (!CreateProcessW(
                        startInfo.FileName,
                        mutableCommandLine,
                        IntPtr.Zero,
                        IntPtr.Zero,
                        true,
                        CreateSuspended | CreateNoWindow | CreateUnicodeEnvironment,
                        environment,
                        startInfo.WorkingDirectory,
                        ref startup,
                        out processInformation))
                {
                    throw Win32("Could not create the suspended render-extract product process.");
                }
                if (!AssignProcessToJobObject(handle, processInformation.Process))
                {
                    throw Win32("Could not atomically bind the suspended render-extract product to its process job.");
                }
                CloseHandle(stdoutWrite);
                stdoutWrite = IntPtr.Zero;
                CloseHandle(stderrWrite);
                stderrWrite = IntPtr.Zero;
                process = Process.GetProcessById(unchecked((int)processInformation.ProcessId));
                stdout = ReaderFromHandle(stdoutRead);
                stdoutRead = IntPtr.Zero;
                stderr = ReaderFromHandle(stderrRead);
                stderrRead = IntPtr.Zero;
                var assignedProcess = new RenderExtractSuspendedProcess(
                    process,
                    stdout,
                    stderr,
                    processInformation.Process,
                    processInformation.Thread);
                process = null;
                stdout = null;
                stderr = null;
                processInformation.Process = IntPtr.Zero;
                processInformation.Thread = IntPtr.Zero;
                return assignedProcess;
            }
            catch
            {
                if (stdout != null) { stdout.Dispose(); }
                if (stderr != null) { stderr.Dispose(); }
                if (process != null) { process.Dispose(); }
                if (processInformation.Process != IntPtr.Zero)
                {
                    TerminateProcess(processInformation.Process, 1);
                }
                throw;
            }
            finally
            {
                CloseIfOpen(processInformation.Thread);
                CloseIfOpen(processInformation.Process);
                CloseIfOpen(stdoutRead);
                CloseIfOpen(stdoutWrite);
                CloseIfOpen(stderrRead);
                CloseIfOpen(stderrWrite);
                if (environment != IntPtr.Zero) { Marshal.FreeHGlobal(environment); }
            }
        }

        public void TerminateAndWait(int timeoutMilliseconds)
        {
            EnsureOpen();
            if (!TerminateJobObject(handle, 1))
            {
                throw Win32("Could not terminate the render-extract process job.");
            }
            uint wait = WaitForSingleObject(handle, checked((uint)timeoutMilliseconds));
            if (wait == WaitObject0) { return; }
            if (wait == WaitFailed) { throw Win32("Could not wait for the render-extract process job to terminate."); }
            throw new TimeoutException("The render-extract process job remained active after forced termination.");
        }

        public bool IsEmpty()
        {
            EnsureOpen();
            int size = Marshal.SizeOf(typeof(JobObjectBasicAccountingInformationData));
            IntPtr buffer = Marshal.AllocHGlobal(size);
            try
            {
                uint returnedLength;
                if (!QueryInformationJobObject(
                    handle,
                    JobObjectBasicAccountingInformation,
                    buffer,
                    (uint)size,
                    out returnedLength))
                {
                    throw Win32("Could not query active processes in the render-extract process job.");
                }
                var information = (JobObjectBasicAccountingInformationData)Marshal.PtrToStructure(
                    buffer,
                    typeof(JobObjectBasicAccountingInformationData));
                return information.ActiveProcesses == 0;
            }
            finally
            {
                Marshal.FreeHGlobal(buffer);
            }
        }

        public bool WaitUntilEmpty(int timeoutMilliseconds)
        {
            EnsureOpen();
            var elapsed = Stopwatch.StartNew();
            while (true)
            {
                if (IsEmpty()) { return true; }
                var remaining = timeoutMilliseconds - elapsed.ElapsedMilliseconds;
                if (remaining <= 0) { return false; }
                Thread.Sleep((int)Math.Min(20L, Math.Max(1L, remaining)));
            }
        }

        public RenderExtractProcessJobLimitSnapshot GetLimitSnapshot()
        {
            EnsureOpen();
            int extendedSize = Marshal.SizeOf(typeof(JobObjectExtendedLimitInformationData));
            IntPtr extendedBuffer = Marshal.AllocHGlobal(extendedSize);
            int cpuRateSize = Marshal.SizeOf(typeof(JobObjectCpuRateControlInformationData));
            IntPtr cpuRateBuffer = Marshal.AllocHGlobal(cpuRateSize);
            try
            {
                uint returnedLength;
                if (!QueryInformationJobObject(
                    handle,
                    JobObjectExtendedLimitInformation,
                    extendedBuffer,
                    (uint)extendedSize,
                    out returnedLength))
                {
                    throw Win32("Could not query the render-extract process job limits.");
                }
                if (!QueryInformationJobObject(
                    handle,
                    JobObjectCpuRateControlInformation,
                    cpuRateBuffer,
                    (uint)cpuRateSize,
                    out returnedLength))
                {
                    throw Win32("Could not query the render-extract process job CPU rate.");
                }
                var extended = (JobObjectExtendedLimitInformationData)Marshal.PtrToStructure(
                    extendedBuffer,
                    typeof(JobObjectExtendedLimitInformationData));
                var cpuRate = (JobObjectCpuRateControlInformationData)Marshal.PtrToStructure(
                    cpuRateBuffer,
                    typeof(JobObjectCpuRateControlInformationData));
                return new RenderExtractProcessJobLimitSnapshot(
                    extended.BasicLimitInformation.ActiveProcessLimit,
                    extended.JobMemoryLimit.ToUInt64(),
                    cpuRate.ControlFlags,
                    cpuRate.CpuRate);
            }
            finally
            {
                Marshal.FreeHGlobal(cpuRateBuffer);
                Marshal.FreeHGlobal(extendedBuffer);
            }
        }

        public void Dispose()
        {
            if (handle != IntPtr.Zero)
            {
                CloseHandle(handle);
                handle = IntPtr.Zero;
            }
            GC.SuppressFinalize(this);
        }

        ~RenderExtractProcessJob() { Dispose(); }

        private static void CreateOutputPipe(out IntPtr read, out IntPtr write)
        {
            var security = new SecurityAttributes();
            security.Length = Marshal.SizeOf(typeof(SecurityAttributes));
            security.InheritHandle = true;
            if (!CreatePipe(out read, out write, ref security, 0))
            {
                throw Win32("Could not create a render-extract output pipe.");
            }
            if (!SetHandleInformation(read, HandleFlagInherit, 0))
            {
                CloseHandle(read);
                CloseHandle(write);
                read = IntPtr.Zero;
                write = IntPtr.Zero;
                throw Win32("Could not protect a render-extract output pipe from inheritance.");
            }
        }

        private static StreamReader ReaderFromHandle(IntPtr handle)
        {
            var safeHandle = new SafeFileHandle(handle, true);
            var stream = new FileStream(safeHandle, FileAccess.Read, 4096, false);
            return new StreamReader(stream, Encoding.UTF8, true, 4096, false);
        }

        private static IntPtr BuildEnvironmentBlock(StringDictionary environmentVariables)
        {
            var variables = new SortedDictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            foreach (DictionaryEntry entry in environmentVariables)
            {
                variables[(string)entry.Key] = entry.Value == null ? String.Empty : (string)entry.Value;
            }
            var block = new StringBuilder();
            foreach (var pair in variables)
            {
                block.Append(pair.Key).Append('=').Append(pair.Value).Append('\0');
            }
            block.Append('\0');
            byte[] bytes = Encoding.Unicode.GetBytes(block.ToString());
            IntPtr pointer = Marshal.AllocHGlobal(bytes.Length);
            Marshal.Copy(bytes, 0, pointer, bytes.Length);
            return pointer;
        }

        private static string Quote(string value)
        {
            if (!value.Contains("\"") && value.IndexOfAny(new[] { ' ', '\t' }) < 0) { return value; }
            return "\"" + value.Replace("\"", "\\\"") + "\"";
        }

        private static void CloseIfOpen(IntPtr value)
        {
            if (value != IntPtr.Zero && value != new IntPtr(-1)) { CloseHandle(value); }
        }

        private static Win32Exception Win32(string message)
        {
            return new Win32Exception(Marshal.GetLastWin32Error(), message);
        }

        private void EnsureOpen()
        {
            if (handle == IntPtr.Zero) { throw new ObjectDisposedException("RenderExtractProcessJob"); }
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct SecurityAttributes
        {
            public int Length;
            public IntPtr SecurityDescriptor;
            [MarshalAs(UnmanagedType.Bool)] public bool InheritHandle;
        }

        [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
        private struct StartupInfo
        {
            public uint Cb;
            public string Reserved;
            public string Desktop;
            public string Title;
            public uint X;
            public uint Y;
            public uint XSize;
            public uint YSize;
            public uint XCountChars;
            public uint YCountChars;
            public uint FillAttribute;
            public uint Flags;
            public ushort ShowWindow;
            public ushort Reserved2;
            public IntPtr ReservedData;
            public IntPtr StdInput;
            public IntPtr StdOutput;
            public IntPtr StdError;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct ProcessInformation
        {
            public IntPtr Process;
            public IntPtr Thread;
            public uint ProcessId;
            public uint ThreadId;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct JobObjectBasicLimitInformation
        {
            public long PerProcessUserTimeLimit;
            public long PerJobUserTimeLimit;
            public uint LimitFlags;
            public UIntPtr MinimumWorkingSetSize;
            public UIntPtr MaximumWorkingSetSize;
            public uint ActiveProcessLimit;
            public IntPtr Affinity;
            public uint PriorityClass;
            public uint SchedulingClass;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct JobObjectBasicAccountingInformationData
        {
            public long TotalUserTime;
            public long TotalKernelTime;
            public long ThisPeriodTotalUserTime;
            public long ThisPeriodTotalKernelTime;
            public uint TotalPageFaultCount;
            public uint TotalProcesses;
            public uint ActiveProcesses;
            public uint TotalTerminatedProcesses;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct IoCounters
        {
            public ulong ReadOperationCount;
            public ulong WriteOperationCount;
            public ulong OtherOperationCount;
            public ulong ReadTransferCount;
            public ulong WriteTransferCount;
            public ulong OtherTransferCount;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct JobObjectExtendedLimitInformationData
        {
            public JobObjectBasicLimitInformation BasicLimitInformation;
            public IoCounters IoInfo;
            public UIntPtr ProcessMemoryLimit;
            public UIntPtr JobMemoryLimit;
            public UIntPtr PeakProcessMemoryUsed;
            public UIntPtr PeakJobMemoryUsed;
        }

        [StructLayout(LayoutKind.Sequential)]
        private struct JobObjectCpuRateControlInformationData
        {
            public uint ControlFlags;
            public uint CpuRate;
        }

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern IntPtr CreateJobObjectW(IntPtr securityAttributes, string name);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool SetInformationJobObject(IntPtr job, uint informationClass, IntPtr information, uint informationLength);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool QueryInformationJobObject(IntPtr job, uint informationClass, IntPtr information, uint informationLength, out uint returnLength);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool TerminateJobObject(IntPtr job, uint exitCode);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool TerminateProcess(IntPtr process, uint exitCode);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern uint ResumeThread(IntPtr thread);
        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool CreateProcessW(string applicationName, StringBuilder commandLine, IntPtr processAttributes, IntPtr threadAttributes, bool inheritHandles, uint creationFlags, IntPtr environment, string currentDirectory, ref StartupInfo startupInfo, out ProcessInformation processInformation);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool CreatePipe(out IntPtr readPipe, out IntPtr writePipe, ref SecurityAttributes pipeAttributes, uint size);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool SetHandleInformation(IntPtr handle, uint mask, uint flags);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern IntPtr GetStdHandle(int standardHandle);
        [DllImport("kernel32.dll", SetLastError = true)]
        private static extern uint WaitForSingleObject(IntPtr handle, uint milliseconds);
        [DllImport("kernel32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool CloseHandle(IntPtr handle);
    }
}
'@ -Language CSharp -ErrorAction Stop
}

function New-RenderExtractBaselineProcessJob {
    param(
        [ValidateRange(0, [Int32]::MaxValue)][int]$MaximumActiveProcessCount = 0,
        [ValidateRange(0, [Int64]::MaxValue)][Int64]$MaximumJobMemoryBytes = 0,
        [ValidateRange(0, 10000)][int]$MaximumCpuRatePerTenThousand = 0
    )

    return [Zircon.Tools.RenderExtractProcessJob]::new(
        [uint32]$MaximumActiveProcessCount,
        [uint64]$MaximumJobMemoryBytes,
        [uint32]$MaximumCpuRatePerTenThousand)
}

function Get-RenderExtractBaselineProcessJobLimits {
    param([Parameter(Mandatory)][Zircon.Tools.RenderExtractProcessJob]$Job)

    return $Job.GetLimitSnapshot()
}

function Start-RenderExtractBaselineAssignedProcess {
    param(
        [Parameter(Mandatory)][Zircon.Tools.RenderExtractProcessJob]$Job,
        [Parameter(Mandatory)][Diagnostics.ProcessStartInfo]$StartInfo
    )

    try {
        return $Job.StartAssigned($StartInfo)
    }
    catch {
        throw "Windows did not start '$($StartInfo.FileName)' in the render-extract process job: $($_.Exception.Message)"
    }
}

function Start-RenderExtractBaselineSuspendedProcess {
    param(
        [Parameter(Mandatory)][Zircon.Tools.RenderExtractProcessJob]$Job,
        [Parameter(Mandatory)][Diagnostics.ProcessStartInfo]$StartInfo
    )

    try {
        return $Job.StartSuspendedAssigned($StartInfo)
    }
    catch {
        throw "Windows did not start '$($StartInfo.FileName)' suspended in the render-extract process job: $($_.Exception.Message)"
    }
}

function Resume-RenderExtractBaselineProcess {
    param([Parameter(Mandatory)][Zircon.Tools.RenderExtractSuspendedProcess]$Process)

    try {
        $Process.Resume()
    }
    catch {
        throw "Windows did not resume the render-extract process job member: $($_.Exception.Message)"
    }
}

function Stop-RenderExtractBaselineProcessJob {
    param(
        [Parameter(Mandatory)][Zircon.Tools.RenderExtractProcessJob]$Job,
        [Parameter(Mandatory)][string]$SessionId
    )

    try {
        $Job.TerminateAndWait(5000)
    }
    catch {
        throw "Render-extract run '$SessionId' process job could not be terminated: $($_.Exception.Message)"
    }
}

function Test-RenderExtractBaselineProcessJobEmpty {
    param(
        [Parameter(Mandatory)][Zircon.Tools.RenderExtractProcessJob]$Job,
        [Parameter(Mandatory)][string]$SessionId
    )

    try {
        return $Job.IsEmpty()
    }
    catch {
        throw "Render-extract run '$SessionId' process job could not report whether it is empty: $($_.Exception.Message)"
    }
}

function Wait-RenderExtractBaselineProcessJobEmpty {
    param(
        [Parameter(Mandatory)][Zircon.Tools.RenderExtractProcessJob]$Job,
        [Parameter(Mandatory)][string]$SessionId,
        [Parameter(Mandatory)][ValidateRange(1, 60000)][int]$TimeoutMilliseconds
    )

    try {
        return $Job.WaitUntilEmpty($TimeoutMilliseconds)
    }
    catch {
        throw "Render-extract run '$SessionId' process job could not wait for child cleanup: $($_.Exception.Message)"
    }
}

function Start-RenderExtractBaselineBoundedOutputCapture {
    param(
        [Parameter(Mandatory)][IO.StreamReader]$Reader,
        [Parameter(Mandatory)][string]$OutputPath,
        [Parameter(Mandatory)][ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumRetainedBytes,
        [Parameter(Mandatory)][string]$TailOutputPath,
        [Parameter(Mandatory)][ValidateRange(1, 65536)][Int64]$MaximumTailBytes,
        [AllowNull()]$RetainedBudget,
        [AllowNull()]$TailBudget
    )

    try {
        if ($null -eq $RetainedBudget) {
            $RetainedBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes $MaximumRetainedBytes
        }
        if ($null -eq $TailBudget) {
            $TailBudget = New-MvpProcessOutputCaptureBudget -MaximumBytes $MaximumTailBytes
        }
        return Start-MvpProcessOutputCapture `
            -Reader $Reader `
            -OutputPath $OutputPath `
            -MaximumRetainedBytes $MaximumRetainedBytes `
            -TailOutputPath $TailOutputPath `
            -MaximumTailBytes $MaximumTailBytes `
            -RetainedBudget $RetainedBudget `
            -TailBudget $TailBudget
    }
    catch {
        throw "Could not start bounded product output capture '$OutputPath': $($_.Exception.Message)"
    }
}

Export-ModuleMember -Function @(
    'New-RenderExtractBaselineProcessJob',
    'Get-RenderExtractBaselineProcessJobLimits',
    'Start-RenderExtractBaselineAssignedProcess',
    'Start-RenderExtractBaselineSuspendedProcess',
    'Resume-RenderExtractBaselineProcess',
    'Stop-RenderExtractBaselineProcessJob',
    'Test-RenderExtractBaselineProcessJobEmpty',
    'Wait-RenderExtractBaselineProcessJobEmpty',
    'Start-RenderExtractBaselineBoundedOutputCapture'
)
