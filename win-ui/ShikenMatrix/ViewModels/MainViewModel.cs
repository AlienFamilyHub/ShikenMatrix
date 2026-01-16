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
        private readonly DispatcherQueueTimer _logCleanupTimer;

        // Configuration
        private ReporterConfig _config = new ReporterConfig();
        private bool _isConfigExpanded = true;

        // Status
        private bool _isRunning;
        private bool _isConnected;
        private string _statusMessage = "就绪";
        private string? _lastError;

        // Permissions
        private bool _hasAccessibilityPermission = true; // Windows doesn't require explicit permission
        private bool _hasMediaPermission = true;

        // Data
        private WindowData? _currentWindow;
        private MediaData? _currentMedia;

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

            _logCleanupTimer = _dispatcher.CreateTimer();
            _logCleanupTimer.Interval = TimeSpan.FromSeconds(15);
            _logCleanupTimer.Tick += OnLogCleanupTimerTick;

            // Load initial config
            LoadConfig();
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
                    OnPropertyChanged(nameof(StatusIndicatorColor));
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
                    OnPropertyChanged(nameof(StatusIndicatorColor));
                }
            }
        }

        public string StatusMessage
        {
            get => _statusMessage;
            set
            {
                if (_statusMessage != value)
                {
                    _statusMessage = value;
                    OnPropertyChanged(nameof(StatusMessage));
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

                var filtered = new ObservableCollection<LogEntry>();
                foreach (var log in Logs)
                {
                    if (log.Message.Contains(SearchText, StringComparison.OrdinalIgnoreCase))
                        filtered.Add(log);
                }
                return filtered;
            }
        }

        // Computed Properties
        public string StatusIndicatorColor =>
            !IsRunning ? "Gray" : (IsConnected ? "Green" : "Orange");

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
                StatusMessage = "启动中...";
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
            }
        }

        private void StopReporter()
        {
            if (_bridge.StopReporter())
            {
                IsRunning = false;
                IsConnected = false;
                StatusMessage = "已停止";
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
            System.Diagnostics.Debug.WriteLine($"[MainViewModel] AddLog: [{level}] {message}");
            
            var log = new LogEntry
            {
                Timestamp = DateTime.Now,
                Message = message,
                Level = level
            };

            Logs.Add(log);

            // Aggressive cleanup: keep only last 200 logs
            if (Logs.Count > 200)
            {
                for (int i = 0; i < 100; i++)
                {
                    Logs.RemoveAt(0);
                }
            }

            OnPropertyChanged(nameof(FilteredLogs));
        }

        private void OnStatusTimerTick(object? sender, object e)
        {
            if (!IsRunning)
                return;

            var status = _bridge.GetStatus();

            if (status.IsConnected != IsConnected)
            {
                IsConnected = status.IsConnected;
                StatusMessage = IsConnected ? "运行中" : "连接中断";
            }

            if (status.LastError != null && status.LastError != LastError)
            {
                LastError = status.LastError;
                AddLog($"错误: {status.LastError}", SmLogLevel.Error);
            }
        }

        private void OnLogCleanupTimerTick(object? sender, object e)
        {
            // Keep only last 200 logs
            if (Logs.Count > 200)
            {
                int removeCount = Logs.Count - 200;
                for (int i = 0; i < removeCount; i++)
                {
                    Logs.RemoveAt(0);
                }
            }

            // Remove logs older than 2 minutes
            var cutoff = DateTime.Now.AddMinutes(-2);
            for (int i = Logs.Count - 1; i >= 0; i--)
            {
                if (Logs[i].Timestamp < cutoff)
                {
                    Logs.RemoveAt(i);
                }
            }

            OnPropertyChanged(nameof(FilteredLogs));
        }

        #endregion

        #region Lifecycle

        public void OnLoaded()
        {
            // Check if reporter was running before
            if (_bridge.IsRunning())
            {
                IsRunning = true;
                SetupCallbacks();
                _statusTimer.Start();
                CheckPermissions();
            }

            // Start log cleanup timer
            _logCleanupTimer.Start();
        }

        public void OnUnloaded()
        {
            // Stop timers
            _statusTimer.Stop();
            _logCleanupTimer.Stop();

            // Clear callbacks to prevent memory leaks
            _bridge.ClearCallbacks();
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
