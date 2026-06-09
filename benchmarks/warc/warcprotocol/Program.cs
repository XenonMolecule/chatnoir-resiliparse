using System.Diagnostics;
using System.Reflection;
using Toimik.WarcProtocol;

const int DefaultBufferSize = 4096 << 10;

if (args.Length != 1)
{
    Console.WriteLine($"Usage: {Environment.GetCommandLineArgs()[0]} WARCFILE");
    return;
}

var path = args[0];
var bufferSize = BufferSize();
var start = Stopwatch.StartNew();
var lastTimer = Stopwatch.StartNew();
var lastCount = 0L;
var lastBytes = 0L;
var totalCount = 0L;
var totalBytes = 0L;

Console.WriteLine($"Reading WARC file: {path}");

var parser = new WarcParser();
var parseLog = new IgnoreParseLog();
var isCompressed = path.EndsWith(".gz", StringComparison.OrdinalIgnoreCase);

await using var stream = new FileStream(
    path,
    FileMode.Open,
    FileAccess.Read,
    FileShare.Read,
    bufferSize,
    FileOptions.SequentialScan);

await foreach (var record in parser.Parse(stream, isCompressed, parseLog))
{
    var contentLength = GetContentLength(record);
    lastCount += 1;
    lastBytes += contentLength;
    totalCount += 1;
    totalBytes += contentLength;

    var elapsed = lastTimer.Elapsed;
    if (elapsed >= TimeSpan.FromMilliseconds(500))
    {
        Console.WriteLine(
            "{0:F0} records/s, {1:F1} MiB/s, {2:F1} KiB/rec ({3} total, {4:F1} MiB)",
            lastCount / elapsed.TotalSeconds,
            lastBytes / elapsed.TotalSeconds / 1024.0 / 1024.0,
            lastBytes / (double)Math.Max(lastCount, 1) / 1024.0,
            totalCount,
            totalBytes / 1024.0 / 1024.0);
        lastCount = 0;
        lastBytes = 0;
        lastTimer.Restart();
    }
}

var totalElapsed = start.Elapsed.TotalSeconds;
Console.WriteLine(
    "Summary: {0:F1}s, {1:F0} records/s, {2:F1} MiB/s, {3:F1} KiB/rec ({4} total, {5:F1} MiB)",
    totalElapsed,
    totalCount / totalElapsed,
    totalBytes / totalElapsed / 1024.0 / 1024.0,
    totalBytes / (double)Math.Max(totalCount, 1) / 1024.0,
    totalCount,
    totalBytes / 1024.0 / 1024.0);

static long GetContentLength(object record)
{
    var type = record.GetType();
    foreach (var propertyName in new[] { "ContentLength", "BlockLength", "Length" })
    {
        var property = type.GetProperty(propertyName, BindingFlags.Instance | BindingFlags.Public);
        if (property?.GetValue(record) is { } value && TryConvertToInt64(value, out var length))
        {
            return length;
        }
    }

    foreach (var propertyName in new[] { "Content", "Body", "Block", "Payload" })
    {
        var property = type.GetProperty(propertyName, BindingFlags.Instance | BindingFlags.Public);
        if (property?.GetValue(record) is { } value && TryGetLength(value, out var length))
        {
            return length;
        }
    }

    return 0;
}

static int BufferSize()
{
    return int.TryParse(Environment.GetEnvironmentVariable("BUFFER_SIZE"), out var value) && value > 0
        ? value
        : DefaultBufferSize;
}

static bool TryGetLength(object value, out long length)
{
    switch (value)
    {
        case byte[] bytes:
            length = bytes.LongLength;
            return true;
        case Memory<byte> memory:
            length = memory.Length;
            return true;
        case ReadOnlyMemory<byte> memory:
            length = memory.Length;
            return true;
        case Stream stream when stream.CanSeek:
            length = stream.Length;
            return true;
    }

    var property = value.GetType().GetProperty("Length", BindingFlags.Instance | BindingFlags.Public)
        ?? value.GetType().GetProperty("Count", BindingFlags.Instance | BindingFlags.Public);
    if (property?.GetValue(value) is { } propertyValue && TryConvertToInt64(propertyValue, out length))
    {
        return true;
    }

    length = 0;
    return false;
}

static bool TryConvertToInt64(object value, out long result)
{
    switch (value)
    {
        case byte b:
            result = b;
            return true;
        case short s:
            result = s;
            return true;
        case int i:
            result = i;
            return true;
        case long l:
            result = l;
            return true;
        case ushort us:
            result = us;
            return true;
        case uint ui:
            result = ui;
            return true;
        case ulong ul when ul <= long.MaxValue:
            result = (long)ul;
            return true;
        default:
            result = 0;
            return false;
    }
}

sealed class IgnoreParseLog : IParseLog
{
    public void ChunkSkipped(string chunk)
    {
    }

    public void ErrorEncountered(string error)
    {
    }
}
