Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($null -eq ('Zircon.Tools.MvpProcessOutputCapture' -as [type])) {
    Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Threading;
using System.Threading.Tasks;

namespace Zircon.Tools
{
    public sealed class MvpProcessOutputCaptureBudget
    {
        private long remainingBytes;

        public long MaximumBytes { get; private set; }
        public long RemainingBytes { get { return Interlocked.Read(ref remainingBytes); } }

        public MvpProcessOutputCaptureBudget(long maximumBytes)
        {
            if (maximumBytes < 1) { throw new ArgumentOutOfRangeException("maximumBytes"); }
            MaximumBytes = maximumBytes;
            remainingBytes = maximumBytes;
        }

        public int Reserve(int requestedBytes)
        {
            if (requestedBytes <= 0) { return 0; }
            while (true)
            {
                long available = Interlocked.Read(ref remainingBytes);
                if (available <= 0) { return 0; }
                int granted = (int)Math.Min((long)requestedBytes, available);
                if (Interlocked.CompareExchange(ref remainingBytes, available - granted, available) == available)
                {
                    return granted;
                }
            }
        }

        public int ReserveExact(int requestedBytes)
        {
            if (requestedBytes <= 0) { return 0; }
            while (true)
            {
                long available = Interlocked.Read(ref remainingBytes);
                if (available < requestedBytes) { return 0; }
                if (Interlocked.CompareExchange(ref remainingBytes, available - requestedBytes, available) == available)
                {
                    return requestedBytes;
                }
            }
        }
    }

    public sealed class MvpProcessOutputCapture
    {
        private readonly byte[] tailBuffer;
        private int tailStart;
        private int tailCount;

        public long TotalBytes { get; private set; }
        public long RetainedBytes { get; private set; }
        public long DroppedBytes { get; private set; }
        public long MaximumTailBytes { get { return tailBuffer.LongLength; } }
        public long TailRetainedBytes { get { return tailCount; } }

        internal MvpProcessOutputCapture(int maximumTailBytes)
        {
            tailBuffer = new byte[maximumTailBytes];
        }

        internal void Record(int bytes, int retainedBytes)
        {
            TotalBytes += bytes;
            RetainedBytes += retainedBytes;
            DroppedBytes += bytes - retainedBytes;
        }

        internal void RecordTail(byte[] source, int offset, int count)
        {
            if (count == 0) { return; }
            if (count >= tailBuffer.Length)
            {
                Buffer.BlockCopy(source, offset + count - tailBuffer.Length, tailBuffer, 0, tailBuffer.Length);
                tailStart = 0;
                tailCount = tailBuffer.Length;
                return;
            }
            int writeIndex = (tailStart + tailCount) % tailBuffer.Length;
            CopyToTail(source, offset, count, writeIndex);
            int overwritten = Math.Max(0, tailCount + count - tailBuffer.Length);
            tailStart = (tailStart + overwritten) % tailBuffer.Length;
            tailCount = Math.Min(tailBuffer.Length, tailCount + count);
        }

        internal void WriteTailTo(Stream output)
        {
            if (tailCount == 0) { return; }
            int firstLength = Math.Min(tailCount, tailBuffer.Length - tailStart);
            output.Write(tailBuffer, tailStart, firstLength);
            int secondLength = tailCount - firstLength;
            if (secondLength > 0) { output.Write(tailBuffer, 0, secondLength); }
        }

        private void CopyToTail(byte[] source, int offset, int count, int destinationOffset)
        {
            int firstLength = Math.Min(count, tailBuffer.Length - destinationOffset);
            Buffer.BlockCopy(source, offset, tailBuffer, destinationOffset, firstLength);
            int secondLength = count - firstLength;
            if (secondLength > 0)
            {
                Buffer.BlockCopy(source, offset + firstLength, tailBuffer, 0, secondLength);
            }
        }
    }

    public static class MvpProcessOutputCaptureRunner
    {
        private const int MaximumTailCapacityBytes = 65536;

        public static Task<MvpProcessOutputCapture> CaptureToFilesAsync(
            StreamReader reader,
            string path,
            long maximumRetainedBytes,
            string tailPath,
            long maximumTailBytes,
            MvpProcessOutputCaptureBudget retainedBudget,
            MvpProcessOutputCaptureBudget tailBudget)
        {
            if (reader == null) { throw new ArgumentNullException("reader"); }
            if (String.IsNullOrWhiteSpace(path)) { throw new ArgumentException("Output path is required.", "path"); }
            if (String.IsNullOrWhiteSpace(tailPath)) { throw new ArgumentException("Tail output path is required.", "tailPath"); }
            if (maximumRetainedBytes < 1) { throw new ArgumentOutOfRangeException("maximumRetainedBytes"); }
            if (maximumTailBytes < 1 || maximumTailBytes > MaximumTailCapacityBytes) { throw new ArgumentOutOfRangeException("maximumTailBytes"); }
            if (retainedBudget == null) { throw new ArgumentNullException("retainedBudget"); }
            if (tailBudget == null) { throw new ArgumentNullException("tailBudget"); }

            var ready = new TaskCompletionSource<object>();
            var captureTask = Task.Run(() =>
            {
                try
                {
                    var tailCapacity = tailBudget.ReserveExact((int)maximumTailBytes);
                    if (tailCapacity != maximumTailBytes)
                    {
                        throw new InvalidOperationException("The shared tail output budget cannot reserve the requested capacity.");
                    }
                    var capture = new MvpProcessOutputCapture(tailCapacity);
                    using (var output = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read, 8192, FileOptions.SequentialScan))
                    using (var tailOutput = new FileStream(tailPath, FileMode.CreateNew, FileAccess.Write, FileShare.Read, 8192, FileOptions.SequentialScan))
                    {
                        ready.TrySetResult(null);
                        var buffer = new byte[8192];
                        var stream = reader.BaseStream;
                        int read;
                        while ((read = stream.Read(buffer, 0, buffer.Length)) > 0)
                        {
                            long streamRemaining = maximumRetainedBytes - capture.RetainedBytes;
                            int requested = streamRemaining > 0 ? (int)Math.Min((long)read, streamRemaining) : 0;
                            int retained = retainedBudget.Reserve(requested);
                            if (retained > 0) { output.Write(buffer, 0, retained); }
                            capture.Record(read, retained);
                            capture.RecordTail(buffer, 0, read);
                        }
                        output.Flush(true);
                        capture.WriteTailTo(tailOutput);
                        tailOutput.Flush(true);
                    }
                    return capture;
                }
                catch (Exception exception)
                {
                    ready.TrySetException(exception);
                    throw;
                }
            });
            ready.Task.GetAwaiter().GetResult();
            return captureTask;
        }
    }
}
'@ -Language CSharp -ErrorAction Stop
}

function New-MvpProcessOutputCaptureBudget {
    param([Parameter(Mandatory)][ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumBytes)

    return [Zircon.Tools.MvpProcessOutputCaptureBudget]::new($MaximumBytes)
}

function Start-MvpProcessOutputCapture {
    param(
        [Parameter(Mandatory)][IO.StreamReader]$Reader,
        [Parameter(Mandatory)][string]$OutputPath,
        [Parameter(Mandatory)][ValidateRange(1, [Int64]::MaxValue)][Int64]$MaximumRetainedBytes,
        [Parameter(Mandatory)][string]$TailOutputPath,
        [Parameter(Mandatory)][ValidateRange(1, 65536)][Int64]$MaximumTailBytes,
        [Parameter(Mandatory)][Zircon.Tools.MvpProcessOutputCaptureBudget]$RetainedBudget,
        [Parameter(Mandatory)][Zircon.Tools.MvpProcessOutputCaptureBudget]$TailBudget
    )

    return [Zircon.Tools.MvpProcessOutputCaptureRunner]::CaptureToFilesAsync(
        $Reader,
        $OutputPath,
        $MaximumRetainedBytes,
        $TailOutputPath,
        $MaximumTailBytes,
        $RetainedBudget,
        $TailBudget)
}

Export-ModuleMember -Function @(
    'New-MvpProcessOutputCaptureBudget',
    'Start-MvpProcessOutputCapture'
)
