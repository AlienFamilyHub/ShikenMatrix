using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Threading.Tasks;
using Microsoft.UI.Dispatching;
using ShikenMatrix.Models;
using ShikenMatrix.Services;
using ShikenMatrix.Native;

namespace ShikenMatrix.ViewModels
{
    /// <summary>
    /// Main ViewModel for the application
    /// </summary>
    public class MainViewModel : INotifyPropertyChanged
    {
        private readonly RustBridge _bridge;
        private readonly DispatcherQueue _dispatcher;
        private readonly DispatcherQueueTimer _statusTimer;

        // String constants to avoid allocations
        private const string UnknownString = "Unknown";
        
        // Monitor status
        private const string MonitorReadyStatus = "就绪";
        private const string MonitorRunningStatus = "监控中";
        private const string MonitorStoppedStatus = "已停止";
        
        // API status
        private const string ApiConnectingStatus = "连接中...";
        private const string ApiConnectedStatus = "已连接";
        private const string ApiDisconnectedStatus = "未连接";
        private const string ApiErrorStatus = "连接失败";

        // Configuration
        private ReporterConfig _config = new ReporterConfig();
        private bool _isConfigExpanded = true;

        // Status
        private bool _isRunning;
        private bool _isConnected;
        private string? _lastError;
        
        // Separate status indicators
        private string _monitorStatus = MonitorReadyStatus;
        private string _apiStatus = ApiDisconnectedStatus;

        // Permissions
        private bool _hasAccessibilityPermission = true; // Windows doesn't require explicit permission
        private bool _hasMediaPermission = true;

        // Data
        private WindowData? _currentWindow;
        private MediaData? _currentMedia;
        
        // Version
        private string _version = "Unknown";

        // UI Logic
        private string _searchText = string.Empty;
        private bool _autoScroll = true;

        public MainViewModel()
        {
            _bridge = new RustBridge();
            _dispatcher = DispatcherQueue.GetForCurrentThread();

            // Setup callbacks
            SetupCallbacks();

            // Create timers
            _statusTimer = _dispatcher.CreateTimer();
            _statusTimer.Interval = TimeSpan.FromSeconds(1);
            _statusTimer.Tick += OnStatusTimerTick;

            // Load initial config
            LoadConfig();
            
            // Get version from native library
            Version = _bridge.GetVersion();
        }

        #region Properties

        // Configuration
        public ReporterConfig Config
        {
            get => _config;
            set
            {
                if (_config != value)
                {
                    _config = value;
                    OnPropertyChanged(nameof(Config));
                }
            }
        }

        public bool IsConfigExpanded
        {
            get => _isConfigExpanded;
            set
            {
                if (_isConfigExpanded != value)
                {
                    _isConfigExpanded = value;
                    OnPropertyChanged(nameof(IsConfigExpanded));
                }
            }
        }

        // Status
        public bool IsRunning
        {
            get => _isRunning;
            set
            {
                if (_isRunning != value)
                {
                    _isRunning = value;
                    OnPropertyChanged(nameof(IsRunning));
                }
            }
        }

        public bool IsConnected
        {
            get => _isConnected;
            set
            {
                if (_isConnected != value)
                {
                    _isConnected = value;
                    OnPropertyChanged(nameof(IsConnected));
                }
            }
        }

        public string? LastError
        {
            get => _lastError;
            set
            {
                if (_lastError != value)
                {
                    _lastError = value;
                    OnPropertyChanged(nameof(LastError));
                    OnPropertyChanged(nameof(HasError));
                }
            }
        }

        public bool HasError => !string.IsNullOrEmpty(LastError);

        // Separate status indicators
        public string MonitorStatus
        {
            get => _monitorStatus;
            set
            {
                if (_monitorStatus != value)
                {
                    _monitorStatus = value;
                    OnPropertyChanged(nameof(MonitorStatus));
                    OnPropertyChanged(nameof(MonitorStatusColor));
                }
            }
        }

        public string ApiStatus
        {
            get => _apiStatus;
            set
            {
                if (_apiStatus != value)
                {
                    _apiStatus = value;
                    OnPropertyChanged(nameof(ApiStatus));
                    OnPropertyChanged(nameof(ApiStatusColor));
                }
            }
        }

        public string MonitorStatusColor =>
            MonitorStatus == MonitorStoppedStatus ? "Gray" :
            MonitorStatus == MonitorRunningStatus ? "Green" : "Orange";

        public string ApiStatusColor =>
            ApiStatus == ApiDisconnectedStatus ? "Gray" :
            ApiStatus == ApiErrorStatus ? "Red" :
            ApiStatus == ApiConnectingStatus ? "Orange" :
            ApiStatus == ApiConnectedStatus ? "Green" : "Gray";

        // Permissions
        public bool HasAccessibilityPermission
        {
            get => _hasAccessibilityPermission;
            set
            {
                if (_hasAccessibilityPermission != value)
                {
                    _hasAccessibilityPermission = value;
                    OnPropertyChanged(nameof(HasAccessibilityPermission));
                }
            }
        }

        public bool HasMediaPermission
        {
            get => _hasMediaPermission;
            set
            {
                if (_hasMediaPermission != value)
                {
                    _hasMediaPermission = value;
                    OnPropertyChanged(nameof(HasMediaPermission));
                }
            }
        }

        // Data
        public WindowData? CurrentWindow
        {
            get => _currentWindow;
            set
            {
                if (_currentWindow != value)
                {
                    _currentWindow = value;
                    OnPropertyChanged(nameof(CurrentWindow));
                    OnPropertyChanged(nameof(HasData));
                }
            }
        }

        public MediaData? CurrentMedia
        {
            get => _currentMedia;
            set
            {
                if (_currentMedia != value)
                {
                    _currentMedia = value;
                    OnPropertyChanged(nameof(CurrentMedia));
                    OnPropertyChanged(nameof(HasData));
                }
            }
        }

        public bool HasData => CurrentWindow != null || CurrentMedia != null;

        // Version
        public string Version
        {
            get => _version;
            set
            {
                if (_version != value)
                {
                    _version = value;
                    OnPropertyChanged(nameof(Version));
                }
            }
        }

        // UI Logic
        public string SearchText
        {
            get => _searchText;
            set
            {
                if (_searchText != value)
                {
                    _searchText = value;
                    OnPropertyChanged(nameof(SearchText));
                }
            }
        }

        public bool AutoScroll
        {
            get => _autoScroll;
            set
            {
                if (_autoScroll != value)
                {
                    _autoScroll = value;
                    OnPropertyChanged(nameof(AutoScroll));
                }
            }
        }

        // Collections
        public ObservableCollection<LogEntry> Logs { get; } = new ObservableCollection<LogEntry>();

        public ObservableCollection<LogEntry> FilteredLogs
        {
            get
            {
                if (string.IsNullOrWhiteSpace(SearchText))
                    return Logs;

                // Create filtered collection only when needed
                var filtered = new ObservableCollection<LogEntry>();
                var searchLower = SearchText.ToLowerInvariant();
                foreach (var log in Logs)
                {
                    if (log.Message.Contains(searchLower, StringComparison.OrdinalIgnoreCase))
                        filtered.Add(log);
                }
                return filtered;
            }
        }

        #endregion
        #region Commands

        public void ToggleReporter()
        {
            if (IsRunning)
            {
                StopReporter();
            }
            else
            {
                StartReporter();
            }
        }

        public void ClearLogs()
        {
            Logs.Clear();
        }

        public void CheckPermissions()
        {
            HasAccessibilityPermission = _bridge.CheckAccessibilityPermission();
            if (Config.EnableMediaReporting)
            {
                HasMediaPermission = _bridge.CheckMediaPermission();
            }
        }

        #endregion

        #region Private Methods

        private void LoadConfig()
        {
            var config = _bridge.LoadConfig();
            if (config != null)
            {
                Config = config;
            }
        }

        private void StartReporter()
        {
            if (string.IsNullOrWhiteSpace(Config.WsUrl) || string.IsNullOrWhiteSpace(Config.Token))
            {
                AddLog("配置无效：请填写 WebSocket 地址和 Token", SmLogLevel.Error);
                return;
            }

            Config.Enabled = true;
            if (!_bridge.SaveConfig(Config))
            {
                AddLog("保存配置失败", SmLogLevel.Error);
                return;
            }

            if (_bridge.StartReporter(Config))
            {
                IsRunning = true;
                MonitorStatus = MonitorRunningStatus;
                ApiStatus = ApiConnectingStatus;
                LastError = null;

                // Setup callbacks after starting (reporter must exist first)
                SetupCallbacks();

                // Start status timer
                _statusTimer.Start();
            }
            else
            {
                AddLog("启动 Reporter 失败", SmLogLevel.Error);
                Config.Enabled = false;
                MonitorStatus = MonitorReadyStatus;
                ApiStatus = ApiErrorStatus;
            }
        }

        private void StopReporter()
        {
            if (_bridge.StopReporter())
            {
                IsRunning = false;
                IsConnected = false;
                MonitorStatus = MonitorStoppedStatus;
                ApiStatus = ApiDisconnectedStatus;
                CurrentWindow = null;
                CurrentMedia = null;
                LastError = null;

                // Stop status timer
                _statusTimer.Stop();

                Config.Enabled = false;
                _bridge.SaveConfig(Config);
            }
        }

        private void SetupCallbacks()
        {
            _bridge.OnLog = (level, message) =>
            {
                _dispatcher.TryEnqueue(() =>
                {
                    AddLog(message, level);
                });
            };

            _bridge.OnWindowData = (window) =>
            {
                _dispatcher.TryEnqueue(() =>
                {
                    CurrentWindow = window;
                });
            };

            _bridge.OnMediaData = (media) =>
            {
                _dispatcher.TryEnqueue(() =>
                {
                    CurrentMedia = media;
                });
            };

            _bridge.OnClearState = () =>
            {
                _dispatcher.TryEnqueue(() =>
                {
                    CurrentWindow = null;
                    CurrentMedia = null;
                });
            };

            // Set native callbacks
            _bridge.SetLogCallback();
            _bridge.SetWindowCallback();
            _bridge.SetMediaCallback();
            System.Diagnostics.Debug.WriteLine("[MainViewModel] All callbacks set.");
        }

        private void AddLog(string message, SmLogLevel level)
        {
            var log = new LogEntry
            {
                Timestamp = DateTime.Now,
                Message = message,
                Level = level
            };

            Logs.Add(log);

            // Keep last 200 logs - remove oldest when limit exceeded
            if (Logs.Count > 200)
            {
                int removeCount = Logs.Count - 200;
                for (int i = 0; i < removeCount; i++)
                {
                    Logs.RemoveAt(0);
                }
            }

            // Only notify if search is active
            if (!string.IsNullOrWhiteSpace(SearchText))
            {
                OnPropertyChanged(nameof(FilteredLogs));
            }
        }

        private void OnStatusTimerTick(object? sender, object e)
        {
            if (!IsRunning)
                return;

            var status = _bridge.GetStatus();

            // Update connection status based on actual WebSocket state
            if (status.IsConnected != IsConnected)
            {
                IsConnected = status.IsConnected;
                ApiStatus = IsConnected ? ApiConnectedStatus : ApiConnectingStatus;
            }

            // Handle errors
            if (status.LastError != null && status.LastError != LastError)
            {
                LastError = status.LastError;
                ApiStatus = ApiErrorStatus;
                AddLog($"错误: {status.LastError}", SmLogLevel.Error);
            }
        }

        #endregion

        #region Lifecycle

        public void OnLoaded()
        {
            // Check if reporter was running before
            if (_bridge.IsRunning())
            {
                IsRunning = true;
                MonitorStatus = MonitorRunningStatus;
                ApiStatus = ApiConnectingStatus;
                SetupCallbacks();
                _statusTimer.Start();
                CheckPermissions();
            }
            else
            {
                MonitorStatus = MonitorReadyStatus;
                ApiStatus = ApiDisconnectedStatus;
            }
        }

        public void OnUnloaded()
        {
            // Stop timers
            if (_statusTimer != null)
            {
                _statusTimer.Stop();
                _statusTimer.Tick -= OnStatusTimerTick;
            }

            // Clear callbacks to prevent memory leaks
            _bridge.ClearCallbacks();
            
            // Clear collections to free memory
            Logs.Clear();
            
            // Force final GC
            GC.Collect(2, GCCollectionMode.Forced);
            GC.WaitForPendingFinalizers();
        }

        #endregion

        #region INotifyPropertyChanged

        public event PropertyChangedEventHandler? PropertyChanged;

        protected void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }

        #endregion
    }
}
