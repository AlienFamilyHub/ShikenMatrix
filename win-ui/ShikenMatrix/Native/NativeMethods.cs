using System;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.WindowsRuntime;

namespace ShikenMatrix.Native
{
    /// <summary>
    /// Log level enumeration matching Rust's LogLevel
    /// </summary>
    public enum SmLogLevel : byte
    {
        Info = 0,
        Warning = 1,
        Error = 2
    }

    /// <summary>
    /// Configuration structure for the reporter
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct SmConfig
    {
        [MarshalAs(UnmanagedType.U1)]
        public bool Enabled;
        // 7 bytes padding for alignment before pointer
        private byte _pad1, _pad2, _pad3, _pad4, _pad5, _pad6, _pad7;
        public IntPtr WsUrl;      // char* (owned by Rust)
        public IntPtr Token;      // char* (owned by Rust)
        [MarshalAs(UnmanagedType.U1)]
        public bool EnableMediaReporting;
        // 7 bytes padding at end
        private byte _pad8, _pad9, _pad10, _pad11, _pad12, _pad13, _pad14;
    }

    /// <summary>
    /// Opaque handle for Reporter instance
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct SmReporter
    {
        private IntPtr _handle; // Opaque pointer
    }

    /// <summary>
    /// Status structure for the reporter
    /// </summary>
    [StructLayout(LayoutKind.Sequential)]
    public struct SmStatus
    {
        [MarshalAs(UnmanagedType.U1)]
        public bool IsRunning;
        [MarshalAs(UnmanagedType.U1)]
        public bool IsConnected;
        // 6 bytes padding for alignment before pointer
        private byte _pad1, _pad2, _pad3, _pad4, _pad5, _pad6;
        public IntPtr LastError; // char* (owned by Rust, null if no error)
    }

    /// <summary>
    /// Callback delegate for log messages
    /// </summary>
    /// <param name="level">Log level</param>
    /// <param name="message">Log message (null-terminated UTF-8 string)</param>
    /// <param name="userData">User-provided data pointer</param>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void SmLogCallbackDelegate(SmLogLevel level, IntPtr message, UIntPtr userData);

    /// <summary>
    /// Callback delegate for window data
    /// </summary>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void SmWindowCallbackDelegate(
        IntPtr title,            // const char*
        IntPtr processName,      // const char*
        uint pid,                // uint32_t
        IntPtr iconData,         // const uint8_t*
        UIntPtr iconSize,        // uintptr_t
        UIntPtr userData);       // uintptr_t

    /// <summary>
    /// Callback delegate for media data
    /// </summary>
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    public delegate void SmMediaCallbackDelegate(
        IntPtr title,            // const char*
        IntPtr artist,           // const char*
        IntPtr album,            // const char*
        double duration,         // double
        double elapsedTime,      // double
        bool playing,            // bool
        IntPtr artworkData,      // const uint8_t*
        UIntPtr artworkSize,     // uintptr_t
        UIntPtr userData);       // uintptr_t

    /// <summary>
    /// P/Invoke declarations for ShikenMatrix native library
    /// </summary>
    internal static class NativeMethods
    {
        private const string DllName = "shikenmatrix_native";

        #region Permission Methods

        /// <summary>
        /// Check if accessibility permission is granted
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_check_accessibility_permission", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern bool SmCheckAccessibilityPermission();

        /// <summary>
        /// Request accessibility permission
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_request_accessibility_permission", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern bool SmRequestAccessibilityPermission();

        /// <summary>
        /// Check if media API is available
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_check_media_permission", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern bool SmCheckMediaPermission();

        /// <summary>
        /// Reset media permission check (removes the blocked marker)
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reset_media_permission_check", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void SmResetMediaPermissionCheck();

        #endregion

        #region Configuration Methods

        /// <summary>
        /// Load configuration from file
        /// Returns a pointer to SmConfig that must be freed with sm_config_free
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_config_load", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern IntPtr SmConfigLoad();

        /// <summary>
        /// Save configuration to file
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_config_save", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: MarshalAs(UnmanagedType.U1)]
        internal static extern bool SmConfigSave(IntPtr config);

        /// <summary>
        /// Free a SmConfig struct created by sm_config_load
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_config_free", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void SmConfigFree(IntPtr config);

        /// <summary>
        /// Free a string allocated by Rust
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_string_free", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void SmStringFree(IntPtr s);

        #endregion

        #region Reporter Lifecycle Methods

        /// <summary>
        /// Start the reporter with the given configuration
        /// Returns handle to the running reporter, or null if failed
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reporter_start", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern IntPtr SmReporterStart(IntPtr config);

        /// <summary>
        /// Stop the running reporter
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reporter_stop", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: MarshalAs(UnmanagedType.U1)]
        internal static extern bool SmReporterStop(IntPtr handle);

        /// <summary>
        /// Get the current status of the reporter
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reporter_get_status", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern SmStatus SmReporterGetStatus(IntPtr handle);

        /// <summary>
        /// Check if the reporter is currently running
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reporter_is_running", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        [return: MarshalAs(UnmanagedType.U1)]
        internal static extern bool SmReporterIsRunning();

        #endregion

        #region Callback Methods

        /// <summary>
        /// Set log callback for receiving formatted logs from backend
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reporter_set_log_callback", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void SmReporterSetLogCallback(SmLogCallbackDelegate callback, UIntPtr userData);

        /// <summary>
        /// Set window data callback for receiving window information
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reporter_set_window_callback", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void SmReporterSetWindowCallback(SmWindowCallbackDelegate callback, UIntPtr userData);

        /// <summary>
        /// Set media data callback for receiving media playback information
        /// </summary>
        [DllImport(DllName, EntryPoint = "sm_reporter_set_media_callback", CallingConvention = CallingConvention.Cdecl, ExactSpelling = true)]
        internal static extern void SmReporterSetMediaCallback(SmMediaCallbackDelegate callback, UIntPtr userData);

        #endregion
    }

    /// <summary>
    /// Helper methods for marshalling data between managed and unmanaged code
    /// </summary>
    internal static class MarshalHelper
    {
        /// <summary>
        /// Convert a UTF-8 string pointer to a C# string
        /// </summary>
        internal static string PtrToStringUTF8(IntPtr ptr)
        {
            if (ptr == IntPtr.Zero)
                return null;

            try
            {
                // Find the length of the null-terminated string
                int len = 0;
                while (Marshal.ReadByte(ptr, len) != 0)
                    len++;

                if (len == 0)
                    return string.Empty;

                byte[] buffer = new byte[len];
                Marshal.Copy(ptr, buffer, 0, len);
                return System.Text.Encoding.UTF8.GetString(buffer);
            }
            catch
            {
                return null;
            }
        }

        /// <summary>
        /// Convert a C# string to a UTF-8 string pointer (allocated with CoTaskMemAlloc)
        /// Remember to free the returned pointer with Marshal.FreeCoTaskMem
        /// </summary>
        internal static IntPtr StringToPtrUTF8(string str)
        {
            if (string.IsNullOrEmpty(str))
                return IntPtr.Zero;

            byte[] buffer = System.Text.Encoding.UTF8.GetBytes(str);
            IntPtr ptr = Marshal.AllocCoTaskMem(buffer.Length + 1);
            Marshal.Copy(buffer, 0, ptr, buffer.Length);
            Marshal.WriteByte(ptr, buffer.Length, 0); // Null terminator
            return ptr;
        }

        /// <summary>
        /// Convert a byte array pointer to a C# byte array
        /// </summary>
        internal static byte[] PtrToByteArray(IntPtr ptr, UIntPtr size)
        {
            if (ptr == IntPtr.Zero || size == UIntPtr.Zero)
                return null;

            try
            {
                byte[] buffer = new byte[(int)size];
                Marshal.Copy(ptr, buffer, 0, (int)size);
                return buffer;
            }
            catch
            {
                return null;
            }
        }
    }
}
