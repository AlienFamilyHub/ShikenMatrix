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
        private Microsoft.UI.Dispatching.DispatcherQueue? _dispatcher;

        public RustBridge()
        {
            // Capture the UI thread dispatcher
            _dispatcher = Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread();
            if (_dispatcher == null)
            {
                System.Diagnostics.Debug.WriteLine("[RustBridge] WARNING: No dispatcher available in constructor");
            }
        }

        #region Version Methods

        /// <summary>
        /// Get the native library version
        /// </summary>
        public string GetVersion()
        {
            IntPtr versionPtr = NativeMethods.SmGetVersion();
            if (versionPtr == IntPtr.Zero)
                return "Unknown";

            try
            {
                string version = MarshalHelper.PtrToStringUTF8(versionPtr) ?? "Unknown";
                NativeMethods.SmStringFree(versionPtr);
                return version;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"Error getting version: {ex.Message}");
                if (versionPtr != IntPtr.Zero)
                    NativeMethods.SmStringFree(versionPtr);
                return "Unknown";
            }
        }

        #endregion

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
            IntPtr wsUrlPtr = MarshalHelper.StringToPtrUTF8(config.WsUrl ?? string.Empty);
            IntPtr tokenPtr = MarshalHelper.StringToPtrUTF8(config.Token ?? string.Empty);

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

            IntPtr wsUrlPtr = MarshalHelper.StringToPtrUTF8(config.WsUrl ?? string.Empty);
            IntPtr tokenPtr = MarshalHelper.StringToPtrUTF8(config.Token ?? string.Empty);

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
            if (!_updatesEnabled)
                return;

            string? titleStr = MarshalHelper.PtrToStringUTF8(title);
            string? processNameStr = MarshalHelper.PtrToStringUTF8(processName);

            var windowData = new WindowData
            {
                Title = titleStr ?? "Unknown",
                ProcessName = processNameStr ?? "Unknown",
                Pid = pid,
                Icon = null // Will be loaded asynchronously
            };

            OnWindowData?.Invoke(windowData);
            
            // Load icon asynchronously on UI thread
            _ = LoadIconAsync(windowData, pid);
        }

        private async System.Threading.Tasks.Task LoadIconAsync(WindowData windowData, uint pid)
        {
            try
            {
                var process = System.Diagnostics.Process.GetProcessById((int)pid);
                string? exePath = process?.MainModule?.FileName;
                
                if (string.IsNullOrEmpty(exePath))
                    return;

                var icon = System.Drawing.Icon.ExtractAssociatedIcon(exePath);
                if (icon == null)
                    return;

                byte[] iconBytes;
                using (var bitmap = icon.ToBitmap())
                using (var memory = new System.IO.MemoryStream())
                {
                    bitmap.Save(memory, System.Drawing.Imaging.ImageFormat.Png);
                    iconBytes = memory.ToArray();
                }

                // Dispose icon to free GDI resources
                icon.Dispose();

                // Switch to UI thread to create BitmapImage
                if (_dispatcher == null)
                {
                    System.Diagnostics.Debug.WriteLine($"[RustBridge] No dispatcher available for icon");
                    return;
                }

                _dispatcher.TryEnqueue(() =>
                {
                    try
                    {
                        var bitmapImage = CreateBitmapImageFromBytes(iconBytes, 40, 40);
                        if (bitmapImage != null)
                        {
                            windowData.Icon = bitmapImage;
                            System.Diagnostics.Debug.WriteLine($"[RustBridge] Icon loaded for PID {pid}");
                        }
                    }
                    catch (Exception ex)
                    {
                        System.Diagnostics.Debug.WriteLine($"[RustBridge] Failed to create icon on UI thread: {ex.Message}");
                    }
                });
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] LoadIconAsync failed for PID {pid}: {ex.Message}");
            }
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
                Artwork = null // Will be loaded asynchronously
            };

            OnMediaData?.Invoke(mediaData);
            
            // Load artwork asynchronously on UI thread
            if (artworkData != IntPtr.Zero && artworkSize != UIntPtr.Zero)
            {
                byte[]? artworkBytes = MarshalHelper.PtrToByteArray(artworkData, artworkSize);
                if (artworkBytes != null)
                {
                    _ = LoadArtworkAsync(mediaData, artworkBytes);
                }
            }
        }

        private async System.Threading.Tasks.Task LoadArtworkAsync(MediaData mediaData, byte[] artworkBytes)
        {
            try
            {
                // Switch to UI thread to create BitmapImage
                if (_dispatcher == null)
                {
                    System.Diagnostics.Debug.WriteLine($"[RustBridge] No dispatcher available for artwork");
                    return;
                }

                _dispatcher.TryEnqueue(() =>
                {
                    try
                    {
                        var bitmapImage = CreateBitmapImageFromBytes(artworkBytes, 200, 200);
                        if (bitmapImage != null)
                        {
                            mediaData.Artwork = bitmapImage;
                            System.Diagnostics.Debug.WriteLine($"[RustBridge] Artwork loaded successfully");
                        }
                        else
                        {
                            System.Diagnostics.Debug.WriteLine($"[RustBridge] CreateBitmapImageFromBytes returned null");
                        }
                    }
                    catch (Exception ex)
                    {
                        System.Diagnostics.Debug.WriteLine($"[RustBridge] Failed to create artwork on UI thread: {ex.Message}");
                    }
                });
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] LoadArtworkAsync failed: {ex.Message}");
            }
        }

        // Dummy callbacks to prevent crashes
        private void DummyLogCallback(SmLogLevel level, IntPtr message, UIntPtr userData) { }
        private void DummyWindowCallback(IntPtr title, IntPtr processName, uint pid, IntPtr iconData, UIntPtr iconSize, UIntPtr userData) { }
        private void DummyMediaCallback(IntPtr title, IntPtr artist, IntPtr album, double duration, double elapsedTime, bool playing, IntPtr artworkData, UIntPtr artworkSize, UIntPtr userData) { }

        #endregion

        #region Image Helpers

        /// <summary>
        /// Get application icon from process ID using Windows Shell API
        /// Must be called from UI thread or use Dispatcher
        /// </summary>
        private BitmapImage? GetIconFromProcess(uint pid)
        {
            try
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Attempting to get icon for PID {pid}");
                
                var process = System.Diagnostics.Process.GetProcessById((int)pid);
                if (process == null)
                {
                    System.Diagnostics.Debug.WriteLine($"[RustBridge] Process {pid} not found");
                    return null;
                }

                string? exePath = null;
                try
                {
                    exePath = process.MainModule?.FileName;
                }
                catch (Exception ex)
                {
                    System.Diagnostics.Debug.WriteLine($"[RustBridge] Cannot access MainModule for PID {pid}: {ex.Message}");
                    return null;
                }

                if (string.IsNullOrEmpty(exePath))
                {
                    System.Diagnostics.Debug.WriteLine($"[RustBridge] No executable path for PID {pid}");
                    return null;
                }

                System.Diagnostics.Debug.WriteLine($"[RustBridge] Extracting icon from: {exePath}");
                
                // Extract icon from executable
                var icon = System.Drawing.Icon.ExtractAssociatedIcon(exePath);
                if (icon == null)
                {
                    System.Diagnostics.Debug.WriteLine($"[RustBridge] No icon found for {exePath}");
                    return null;
                }

                System.Diagnostics.Debug.WriteLine($"[RustBridge] Icon extracted, converting to BitmapImage");

                // Convert Icon to byte array
                byte[] iconBytes;
                using (var bitmap = icon.ToBitmap())
                using (var memory = new System.IO.MemoryStream())
                {
                    bitmap.Save(memory, System.Drawing.Imaging.ImageFormat.Png);
                    iconBytes = memory.ToArray();
                }

                // Create BitmapImage on UI thread
                BitmapImage? result = null;
                var dispatcher = Microsoft.UI.Dispatching.DispatcherQueue.GetForCurrentThread();
                if (dispatcher != null)
                {
                    // Already on UI thread
                    result = CreateBitmapImageFromBytes(iconBytes, 40, 40);
                }
                else
                {
                    // Need to invoke on UI thread - but we can't wait here
                    // Return null and let UI show placeholder
                    System.Diagnostics.Debug.WriteLine($"[RustBridge] Not on UI thread, cannot create BitmapImage");
                    return null;
                }
                
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Icon converted successfully for PID {pid}");
                return result;
            }
            catch (System.ComponentModel.Win32Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Access denied for PID {pid}: {ex.Message}");
                return null;
            }
            catch (ArgumentException ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Process {pid} has exited: {ex.Message}");
                return null;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Failed to get icon for PID {pid}: {ex.GetType().Name} - {ex.Message}");
                return null;
            }
        }

        /// <summary>
        /// Create BitmapImage from byte array (must be called on UI thread)
        /// </summary>
        private BitmapImage? CreateBitmapImageFromBytes(byte[] bytes, int width, int height)
        {
            Windows.Storage.Streams.InMemoryRandomAccessStream? raStream = null;
            try
            {
                var bitmapImage = new BitmapImage();
                bitmapImage.DecodePixelWidth = width;
                bitmapImage.DecodePixelHeight = height;
                
                raStream = new Windows.Storage.Streams.InMemoryRandomAccessStream();
                var writeTask = raStream.WriteAsync(bytes.AsBuffer()).AsTask();
                writeTask.Wait();
                raStream.Seek(0);
                bitmapImage.SetSource(raStream);
                
                return bitmapImage;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Failed to create BitmapImage: {ex.Message}");
                return null;
            }
            finally
            {
                // Dispose stream to free memory
                raStream?.Dispose();
            }
        }

        /// <summary>
        /// Convert icon data from callback to BitmapImage
        /// </summary>
        private BitmapImage? GetWindowIconFromData(IntPtr data, UIntPtr size)
        {
            if (data == IntPtr.Zero || size == UIntPtr.Zero)
                return null;

            try
            {
                byte[]? buffer = MarshalHelper.PtrToByteArray(data, size);
                if (buffer == null)
                    return null;

                var image = new BitmapImage();
                // Set decode pixel size for icons
                image.DecodePixelWidth = 40;
                image.DecodePixelHeight = 40;
                
                using (var stream = new Windows.Storage.Streams.InMemoryRandomAccessStream())
                {
                    stream.WriteAsync(buffer.AsBuffer()).AsTask().Wait();
                    stream.Seek(0);
                    image.SetSource(stream);
                }
                return image;
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Failed to load icon: {ex.Message}");
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
                byte[]? buffer = MarshalHelper.PtrToByteArray(data, size);
                if (buffer == null)
                    return null;

                // Create BitmapImage - must be on UI thread
                return CreateBitmapImageFromBytes(buffer, 200, 200);
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[RustBridge] Failed to load artwork: {ex.Message}");
                return null;
            }
        }

        #endregion
    }
}
