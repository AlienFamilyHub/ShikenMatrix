using System;
using System.Drawing;
using System.IO;
using System.Reflection;
using System.Windows.Forms;
using Microsoft.UI.Xaml;
using Application = Microsoft.UI.Xaml.Application;

namespace ShikenMatrix.Services
{
    /// <summary>
    /// Manages the system tray icon for the application
    /// </summary>
    public class TrayIconManager : IDisposable
    {
        private NotifyIcon? _notifyIcon;
        private Window? _mainWindow;
        private bool _isDisposed;
        private Icon? _currentIcon;

        public TrayIconManager()
        {
            // Create system tray icon
            _notifyIcon = new NotifyIcon
            {
                Text = "ShikenMatrix - 窗口上报工具",
                Visible = true
            };

            // Set icon (using a simple icon for now)
            _currentIcon = CreateIcon();
            _notifyIcon.Icon = _currentIcon;

            // Build context menu
            BuildContextMenu();

            // Double-click to show window
            _notifyIcon.DoubleClick += OnDoubleClick;
        }

        /// <summary>
        /// Set the main window reference
        /// </summary>
        public void SetWindow(Window window)
        {
            _mainWindow = window;
        }

        /// <summary>
        /// Update the status text in the tray icon tooltip
        /// </summary>
        public void UpdateStatus(bool isRunning, bool isConnected)
        {
            if (_notifyIcon == null)
                return;

            string status = isRunning
                ? (isConnected ? "状态: 已连接" : "状态: 连接中...")
                : "状态: 已停止";

            _notifyIcon.Text = $"ShikenMatrix - {status}";
        }

        /// <summary>
        /// Show the main window
        /// </summary>
        public void ShowWindow()
        {
            if (_mainWindow != null)
            {
                // Bring window to front
                _mainWindow.Activate();
            }
        }

        /// <summary>
        /// Hide the main window
        /// </summary>
        public void HideWindow()
        {
            // Window hiding is handled by the window itself
        }

        private void BuildContextMenu()
        {
            if (_notifyIcon == null)
                return;

            var contextMenu = new ContextMenuStrip();

            // Show Settings
            var showItem = new ToolStripMenuItem("显示设置");
            showItem.Click += (s, e) => ShowWindow();
            contextMenu.Items.Add(showItem);

            contextMenu.Items.Add(new ToolStripSeparator());

            // Status (read-only)
            var statusItem = new ToolStripMenuItem("状态: 已停止");
            statusItem.Name = "StatusItem";
            statusItem.Enabled = false;
            contextMenu.Items.Add(statusItem);

            contextMenu.Items.Add(new ToolStripSeparator());

            // Quit
            var quitItem = new ToolStripMenuItem("退出");
            quitItem.Click += (s, e) => ExitApplication();
            contextMenu.Items.Add(quitItem);

            _notifyIcon.ContextMenuStrip = contextMenu;
        }

        private void UpdateContextMenuStatus(bool isRunning, bool isConnected)
        {
            if (_notifyIcon?.ContextMenuStrip == null)
                return;

            var statusItem = _notifyIcon.ContextMenuStrip.Items.Find("StatusItem", false);
            if (statusItem.Length > 0 && statusItem[0] is ToolStripMenuItem menuItem)
            {
                menuItem.Text = isRunning
                    ? (isConnected ? "状态: 已连接" : "状态: 连接中...")
                    : "状态: 已停止";
            }
        }

        private void OnDoubleClick(object? sender, EventArgs e)
        {
            ShowWindow();
        }

        private void ExitApplication()
        {
            // Close the main window, which will trigger application shutdown
            Application.Current.Exit();
        }

        /// <summary>
        /// Load icon from Assets or create a fallback icon
        /// </summary>
        private Icon CreateIcon()
        {
            try
            {
                // Try to load icon from app package
                string appLocation = AppContext.BaseDirectory;
                string iconPath = Path.Combine(appLocation, "Assets", "Icon.png");

                if (File.Exists(iconPath))
                {
                    // Load PNG and convert to icon
                    using var original = new Bitmap(iconPath);
                    // Create 16x16 icon for system tray
                    using var resized = new Bitmap(original, new Size(16, 16));
                    return Icon.FromHandle(resized.GetHicon());
                }
            }
            catch (Exception)
            {
                // Fall through to create default icon
            }

            // Create a default icon if loading fails
            return CreateDefaultIcon();
        }

        /// <summary>
        /// Create a default fallback icon
        /// </summary>
        private Icon CreateDefaultIcon()
        {
            var bitmap = new Bitmap(16, 16);
            using (var g = System.Drawing.Graphics.FromImage(bitmap))
            {
                g.Clear(Color.Transparent);
                // Draw a simple chart icon
                using (var pen = new Pen(Color.DodgerBlue, 2))
                {
                    g.DrawRectangle(pen, 2, 4, 12, 10);
                    g.DrawLine(pen, 4, 12, 7, 8);
                    g.DrawLine(pen, 7, 8, 10, 10);
                    g.DrawLine(pen, 10, 10, 12, 6);
                }
            }
            return Icon.FromHandle(bitmap.GetHicon());
        }

        public void Dispose()
        {
            if (_isDisposed)
                return;

            _isDisposed = true;

            if (_notifyIcon != null)
            {
                _notifyIcon.Visible = false;
                _notifyIcon.DoubleClick -= OnDoubleClick;
                
                // Dispose context menu
                if (_notifyIcon.ContextMenuStrip != null)
                {
                    _notifyIcon.ContextMenuStrip.Dispose();
                    _notifyIcon.ContextMenuStrip = null;
                }
                
                _notifyIcon.Dispose();
                _notifyIcon = null;
            }
            
            // Dispose icon to free GDI resources
            if (_currentIcon != null)
            {
                _currentIcon.Dispose();
                _currentIcon = null;
            }
        }
    }
}
