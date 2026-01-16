using System;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Threading.Tasks;
using ShikenMatrix.Models;
using ShikenMatrix.Native;
using Microsoft.UI.Xaml.Media.Imaging;

namespace ShikenMatrix.Services
{
    /// <summary>
    /// Bridge service to communicate with Rust native library
    /// </summary>
    public class RustBridge
    {
        // Callback delegates to prevent garbage collection
        private SmLogCallbackDelegate? _logCallbackDelegate;
        private SmWindowCallbackDelegate? _windowCallbackDelegate;
        private SmMediaCallbackDelegate? _mediaCallbackDelegate;

        // Callback actions
        public Action<SmLogLevel, string>? OnLog { get; set; }
        public Action<WindowData>? OnWindowData { get; set; }
        public Action<MediaData>? OnMediaData { get; set; }
        public Action? OnClearState { get; set; }

        private IntPtr _reporterHandle = IntPtr.Zero;
        private bool _updatesEnabled = true;

        #region Configuration Methods

        /// <summary>
        /// Load configuration from native storage
        /// </summary>
        public ReporterConfig? LoadConfig()
        {
            IntPtr configPtr = NativeMethods.SmConfigLoad();
            if (configPtr == IntPtr.Zero)
                return null;

            try
            {
                SmConfig config = Marshal.PtrToStructure<SmConfig>(configPtr);
                var result = new ReporterConfig
                {
                    Enabled = config.Enabled,
                    WsUrl = MarshalHelper.PtrToStringUTF8(config.WsUrl) ?? string.Empty,
                    Token = MarshalHelper.PtrToStringUTF8(config.Token) ?? string.Empty,
                    EnableMediaReporting = config.EnableMediaReporting
                };

                NativeMethods.SmConfigFree(configPtr);
                return result;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Error loading config: {ex.Message}");
                NativeMethods.SmConfigFree(configPtr);
                return null;
            }
        }

        /// <summary>
        /// Save configuration to native storage
        /// </summary>
        public bool SaveConfig(ReporterConfig config)
        {
            IntPtr wsUrlPtr = MarshalHelper.StringToPtrUTF8(config.WsUrl);
            IntPtr tokenPtr = MarshalHelper.StringToPtrUTF8(config.Token);

            try
            {
                var nativeConfig = new SmConfig
                {
                    Enabled = config.Enabled,
                    WsUrl = wsUrlPtr,
                    Token = tokenPtr,
                    EnableMediaReporting = config.EnableMediaReporting
                };

                IntPtr configPtr = Marshal.AllocHGlobal(Marshal.SizeOf(nativeConfig));
                Marshal.StructureToPtr(nativeConfig, configPtr, false);

                bool result = NativeMethods.SmConfigSave(configPtr);
                Marshal.FreeHGlobal(configPtr);
                return result;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Error saving config: {ex.Message}");
                return false;
            }
            finally
            {
                if (wsUrlPtr != IntPtr.Zero)
                    Marshal.FreeCoTaskMem(wsUrlPtr);
                if (tokenPtr != IntPtr.Zero)
                    Marshal.FreeCoTaskMem(tokenPtr);
            }
        }

        #endregion

        #region Reporter Lifecycle Methods

        /// <summary>
        /// Start the reporter with the given configuration
        /// </summary>
        public bool StartReporter(ReporterConfig config)
        {
            System.Diagnostics.Debug.WriteLine($"[RustBridge] StartReporter called: WsUrl={config.WsUrl}, Token length={config.Token?.Length}");
            
            if (IsRunning())
            {
                System.Diagnostics.Debug.WriteLine("[RustBridge] Reporter already running!");
                return false;
            }

            IntPtr wsUrlPtr = MarshalHelper.StringToPtrUTF8(config.WsUrl);
            IntPtr tokenPtr = MarshalHelper.StringToPtrUTF8(config.Token);

            try
            {
                var nativeConfig = new SmConfig
                {
                    Enabled = config.Enabled,
                    WsUrl = wsUrlPtr,
                    Token = tokenPtr,
                    EnableMediaReporting = config.EnableMediaReporting
                };

                IntPtr configPtr = Marshal.AllocHGlobal(Marshal.SizeOf(nativeConfig));
                Marshal.StructureToPtr(nativeConfig, configPtr, false);

                _reporterHandle = NativeMethods.SmReporterStart(configPtr);
                Marshal.FreeHGlobal(configPtr);

                return _reporterHandle != IntPtr.Zero;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Error starting reporter: {ex.Message}");
                return false;
            }
            finally
            {
                if (wsUrlPtr != IntPtr.Zero)
                    Marshal.FreeCoTaskMem(wsUrlPtr);
                if (tokenPtr != IntPtr.Zero)
                    Marshal.FreeCoTaskMem(tokenPtr);
            }
        }

        /// <summary>
        /// Stop the reporter
        /// </summary>
        public bool StopReporter()
        {
            if (_reporterHandle == IntPtr.Zero)
                return false;

            bool result = NativeMethods.SmReporterStop(_reporterHandle);
            _reporterHandle = IntPtr.Zero;
            return result;
        }

        /// <summary>
        /// Check if the reporter is running
        /// </summary>
        public bool IsRunning()
        {
            return NativeMethods.SmReporterIsRunning();
        }

        /// <summary>
        /// Get the current status of the reporter
        /// </summary>
        public ReporterStatus GetStatus()
        {
            if (_reporterHandle == IntPtr.Zero)
                return new ReporterStatus { IsRunning = false, IsConnected = false };

            SmStatus status = NativeMethods.SmReporterGetStatus(_reporterHandle);

            return new ReporterStatus
            {
                IsRunning = status.IsRunning,
                IsConnected = status.IsConnected,
                LastError = MarshalHelper.PtrToStringUTF8(status.LastError)
            };
        }

        #endregion

        #region Callback Methods

        /// <summary>
        /// Set log callback to receive formatted logs from backend
        /// </summary>
        public void SetLogCallback()
        {
            System.Diagnostics.Debug.WriteLine("[RustBridge] Setting log callback...");
            _logCallbackDelegate = LogCallbackWrapper;
            NativeMethods.SmReporterSetLogCallback(_logCallbackDelegate, UIntPtr.Zero);
            System.Diagnostics.Debug.WriteLine("[RustBridge] Log callback set.");
        }

        /// <summary>
        /// Set window data callback to receive window information
        /// </summary>
        public void SetWindowCallback()
        {
            System.Diagnostics.Debug.WriteLine("[RustBridge] Setting window callback...");
            _windowCallbackDelegate = WindowCallbackWrapper;
            NativeMethods.SmReporterSetWindowCallback(_windowCallbackDelegate, UIntPtr.Zero);
            System.Diagnostics.Debug.WriteLine("[RustBridge] Window callback set.");
        }

        /// <summary>
        /// Set media data callback to receive media playback information
        /// </summary>
        public void SetMediaCallback()
        {
            _mediaCallbackDelegate = MediaCallbackWrapper;
            NativeMethods.SmReporterSetMediaCallback(_mediaCallbackDelegate, UIntPtr.Zero);
        }

        /// <summary>
        /// Clear all callbacks to prevent memory leaks
        /// </summary>
        public void ClearCallbacks()
        {
            // Set dummy callbacks to prevent crashes from dangling pointers
            _logCallbackDelegate = DummyLogCallback;
            _windowCallbackDelegate = DummyWindowCallback;
            _mediaCallbackDelegate = DummyMediaCallback;

            NativeMethods.SmReporterSetLogCallback(_logCallbackDelegate, UIntPtr.Zero);
            NativeMethods.SmReporterSetWindowCallback(_windowCallbackDelegate, UIntPtr.Zero);
            NativeMethods.SmReporterSetMediaCallback(_mediaCallbackDelegate, UIntPtr.Zero);

            OnLog = null;
            OnWindowData = null;
            OnMediaData = null;
            OnClearState = null;
        }

        #endregion

        #region Permission Methods

        /// <summary>
        /// Check if accessibility permission is granted
        /// </summary>
        public bool CheckAccessibilityPermission()
        {
            return NativeMethods.SmCheckAccessibilityPermission();
        }

        /// <summary>
        /// Request accessibility permission
        /// </summary>
        public bool RequestAccessibilityPermission()
        {
            return NativeMethods.SmRequestAccessibilityPermission();
        }

        /// <summary>
        /// Check if media API is available
        /// </summary>
        public bool CheckMediaPermission()
        {
            return NativeMethods.SmCheckMediaPermission();
        }

        /// <summary>
        /// Reset media permission check
        /// </summary>
        public void ResetMediaPermissionCheck()
        {
            NativeMethods.SmResetMediaPermissionCheck();
        }

        #endregion

        #region UI Update Control

        /// <summary>
        /// Enable or disable UI updates from callbacks
        /// Set to false when window is hidden to save memory/CPU
        /// </summary>
        public void SetUpdatesEnabled(bool enabled)
        {
            _updatesEnabled = enabled;
            if (!enabled)
            {
                // Notify UI to clear state when disabling
                OnClearState?.Invoke();
            }
        }

        #endregion

        #region Callback Wrappers

        private void LogCallbackWrapper(SmLogLevel level, IntPtr message, UIntPtr userData)
        {
            System.Diagnostics.Debug.WriteLine($"[RustBridge] LogCallback called: level={level}, updatesEnabled={_updatesEnabled}");
            
            if (!_updatesEnabled)
                return;

            string? msg = MarshalHelper.PtrToStringUTF8(message);
            System.Diagnostics.Debug.WriteLine($"[RustBridge] Log message: {msg}");
            if (msg != null)
            {
                OnLog?.Invoke((SmLogLevel)level, msg);
            }
        }

        private void WindowCallbackWrapper(IntPtr title, IntPtr processName, uint pid, IntPtr iconData, UIntPtr iconSize, UIntPtr userData)
        {
            System.Diagnostics.Debug.WriteLine($"[RustBridge] WindowCallback called: pid={pid}, updatesEnabled={_updatesEnabled}");
            
            if (!_updatesEnabled)
                return;

            string? titleStr = MarshalHelper.PtrToStringUTF8(title);
            string? processNameStr = MarshalHelper.PtrToStringUTF8(processName);
            System.Diagnostics.Debug.WriteLine($"[RustBridge] Window: {titleStr} ({processNameStr})");

            var windowData = new WindowData
            {
                Title = titleStr ?? "Unknown",
                ProcessName = processNameStr ?? "Unknown",
                Pid = pid,
                Icon = GetWindowIcon(pid) // Fetch icon natively
            };

            OnWindowData?.Invoke(windowData);
        }

        private void MediaCallbackWrapper(IntPtr title, IntPtr artist, IntPtr album, double duration, double elapsedTime, bool playing, IntPtr artworkData, UIntPtr artworkSize, UIntPtr userData)
        {
            if (!_updatesEnabled)
                return;

            string? titleStr = MarshalHelper.PtrToStringUTF8(title);
            string? artistStr = MarshalHelper.PtrToStringUTF8(artist);
            string? albumStr = MarshalHelper.PtrToStringUTF8(album);

            var mediaData = new MediaData
            {
                Title = titleStr ?? "Unknown",
                Artist = artistStr ?? "Unknown",
                Album = albumStr ?? "Unknown",
                Duration = duration,
                ElapsedTime = elapsedTime,
                Playing = playing,
                Artwork = GetArtworkImage(artworkData, artworkSize)
            };

            OnMediaData?.Invoke(mediaData);
        }

        // Dummy callbacks to prevent crashes
        private void DummyLogCallback(SmLogLevel level, IntPtr message, UIntPtr userData) { }
        private void DummyWindowCallback(IntPtr title, IntPtr processName, uint pid, IntPtr iconData, UIntPtr iconSize, UIntPtr userData) { }
        private void DummyMediaCallback(IntPtr title, IntPtr artist, IntPtr album, double duration, double elapsedTime, bool playing, IntPtr artworkData, UIntPtr artworkSize, UIntPtr userData) { }

        #endregion

        #region Image Helpers

        /// <summary>
        /// Get window icon from system (native Windows API)
        /// </summary>
        private BitmapImage? GetWindowIcon(uint pid)
        {
            try
            {
                // Use Windows API to get application icon
                // This is a placeholder - actual implementation would use Windows APIs
                // For now, return null which will use a default icon in the UI
                return null;
            }
            catch
            {
                return null;
            }
        }

        /// <summary>
        /// Convert artwork data to BitmapImage
        /// </summary>
        private BitmapImage? GetArtworkImage(IntPtr data, UIntPtr size)
        {
            if (data == IntPtr.Zero || size == UIntPtr.Zero)
                return null;

            try
            {
                // Limit size to 2MB to prevent memory issues
                if (size.ToUInt64() > 2_000_000)
                    return null;

                byte[]? buffer = MarshalHelper.PtrToByteArray(data, size);
                if (buffer == null)
                    return null;

                var image = new BitmapImage();
                using (var stream = new Windows.Storage.Streams.InMemoryRandomAccessStream())
                {
                    stream.WriteAsync(buffer.AsBuffer()).AsTask().Wait();
                    stream.Seek(0);
                    image.SetSource(stream);
                }
                return image;
            }
            catch
            {
                return null;
            }
        }

        #endregion
    }
}
