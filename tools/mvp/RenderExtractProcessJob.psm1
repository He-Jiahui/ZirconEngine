Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

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
using Microsoft.Win32.SafeHandles;

namespace Zircon.Tools
{
    public sealed class RenderExtractSuspendedProcess : IDisposable
    {
        public Process Process { get; private set; }
        public StreamReader StandardOutput { get; private set; }
        public StreamReader StandardError { get; private set; }
        private IntPtr nativeProcessHandle;

        internal RenderExtractSuspendedProcess(
            Process process,
            StreamReader stdout,
            StreamReader stderr,
            IntPtr nativeProcessHandle)
        {
            Process = process;
            StandardOutput = stdout;
            StandardError = stderr;
            this.nativeProcessHandle = nativeProcessHandle;
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

        public void Dispose()
        {
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
    }

    public sealed class RenderExtractProcessJob : IDisposable
    {
        private const uint JobObjectExtendedLimitInformation = 9;
        private const uint JobObjectLimitKillOnJobClose = 0x00002000;
        private const uint CreateSuspended = 0x00000004;
        private const uint CreateNoWindow = 0x08000000;
        private const uint CreateUnicodeEnvironment = 0x00000400;
        private const uint StartfUseStdHandles = 0x00000100;
        private const uint HandleFlagInherit = 0x00000001;
        private const uint WaitObject0 = 0;
        private const uint WaitFailed = 0xFFFFFFFF;

        private IntPtr handle;

        public RenderExtractProcessJob()
        {
            handle = CreateJobObjectW(IntPtr.Zero, null);
            if (handle == IntPtr.Zero)
            {
                throw Win32("Could not create the render-extract process job.");
            }
            var limits = new JobObjectExtendedLimitInformationData();
            limits.BasicLimitInformation.LimitFlags = JobObjectLimitKillOnJobClose;
            int size = Marshal.SizeOf(typeof(JobObjectExtendedLimitInformationData));
            IntPtr buffer = Marshal.AllocHGlobal(size);
            try
            {
                Marshal.StructureToPtr(limits, buffer, false);
                if (!SetInformationJobObject(handle, JobObjectExtendedLimitInformation, buffer, (uint)size))
                {
                    throw Win32("Could not configure the render-extract process job.");
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
                if (ResumeThread(processInformation.Thread) == UInt32.MaxValue)
                {
                    throw Win32("Could not resume the job-bound render-extract product process.");
                }
                var assignedProcess = new RenderExtractSuspendedProcess(
                    process,
                    stdout,
                    stderr,
                    processInformation.Process);
                process = null;
                stdout = null;
                stderr = null;
                processInformation.Process = IntPtr.Zero;
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

        [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
        private static extern IntPtr CreateJobObjectW(IntPtr securityAttributes, string name);
        [DllImport("kernel32.dll", SetLastError = true)]
        [return: MarshalAs(UnmanagedType.Bool)]
        private static extern bool SetInformationJobObject(IntPtr job, uint informationClass, IntPtr information, uint informationLength);
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
    return [Zircon.Tools.RenderExtractProcessJob]::new()
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

Export-ModuleMember -Function @(
    'New-RenderExtractBaselineProcessJob',
    'Start-RenderExtractBaselineAssignedProcess',
    'Stop-RenderExtractBaselineProcessJob'
)
