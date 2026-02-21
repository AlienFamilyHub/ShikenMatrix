using System;
using ShikenMatrix.Native;

namespace ShikenMatrix.Models
{
    /// <summary>
    /// Log entry model (immutable)
    /// </summary>
    public class LogEntry
    {
        public DateTime Timestamp { get; init; }
        public string Message { get; init; } = string.Empty;
        public SmLogLevel Level { get; init; }
    }
}
