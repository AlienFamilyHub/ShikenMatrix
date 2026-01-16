using Microsoft.UI.Xaml;
using ShikenMatrix.Services;
using System;

namespace ShikenMatrix
{
    /// <summary>
    /// Provides application-specific behavior to supplement the default Application class.
    /// </summary>
    public partial class App : Application
    {
        private MainWindow? _mainWindow;
        private TrayIconManager? _trayIconManager;

        /// <summary>
        /// Initializes the singleton application object.
        /// </summary>
        public App()
        {
            InitializeComponent();
            
            // Register global exception handler
            UnhandledException += App_UnhandledException;
        }

        private void App_UnhandledException(object sender, Microsoft.UI.Xaml.UnhandledExceptionEventArgs e)
        {
            System.Diagnostics.Debug.WriteLine($"[FATAL] Unhandled exception: {e.Exception}");
            System.Diagnostics.Debug.WriteLine($"[FATAL] Stack trace: {e.Exception.StackTrace}");
            e.Handled = true; // Prevent crash to see the error
        }

        /// <summary>
        /// Invoked when the application is launched.
        /// </summary>
        protected override void OnLaunched(LaunchActivatedEventArgs args)
        {
            try
            {
                // Create tray icon manager first
                _trayIconManager = new TrayIconManager();

                // Create main window
                _mainWindow = new MainWindow();
                _trayIconManager.SetWindow(_mainWindow);

                // Subscribe to window closed event
                _mainWindow.Closed += OnMainWindowClosed;

                // Activate the window
                _mainWindow.Activate();
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"[FATAL] OnLaunched exception: {ex}");
                System.Diagnostics.Debug.WriteLine($"[FATAL] Stack trace: {ex.StackTrace}");
                throw;
            }
        }

        private void OnMainWindowClosed(object sender, WindowEventArgs args)
        {
            // Clean up tray icon
            _trayIconManager?.Dispose();
        }

        /// <summary>
        /// Get the tray icon manager instance
        /// </summary>
        public TrayIconManager? TrayIconManager => _trayIconManager;
    }
}
