using System;
using Microsoft.UI.Xaml;
using ShikenMatrix.ViewModels;

namespace ShikenMatrix
{
    /// <summary>
    /// Main window for ShikenMatrix application
    /// </summary>
    public sealed partial class MainWindow : Window
    {
        public MainViewModel ViewModel { get; }

        public MainWindow()
        {
            ViewModel = new MainViewModel();
            InitializeComponent();
            
            // Set DataContext for normal Bindings
            if (Content is FrameworkElement content)
            {
                content.DataContext = ViewModel;
            }

            // Subscribe to events
            Closed += OnClosed;

            // Subscribe to ViewModel status changes
            ViewModel.PropertyChanged += OnViewModelPropertyChanged;

            // Initialize ViewModel
            ViewModel.OnLoaded();
        }

        private void OnViewModelPropertyChanged(object? sender, System.ComponentModel.PropertyChangedEventArgs e)
        {
            if (e.PropertyName == nameof(ViewModel.IsRunning) || e.PropertyName == nameof(ViewModel.IsConnected))
            {
                UpdateTrayStatus();
            }
        }

        private void OnClosed(object sender, WindowEventArgs args)
        {
            // Unsubscribe from events to prevent memory leaks
            ViewModel.PropertyChanged -= OnViewModelPropertyChanged;
            Closed -= OnClosed;
            
            ViewModel.OnUnloaded();
        }

        private void UpdateTrayStatus()
        {
            if (App.Current is App app && app.TrayIconManager != null)
            {
                app.TrayIconManager.UpdateStatus(ViewModel.IsRunning, ViewModel.IsConnected);
            }
        }

        private Microsoft.UI.Xaml.Controls.Primitives.FlyoutBase? _githubFlyout;

        private void OnGitHubHoverEntered(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
        {
            if (sender is FrameworkElement element)
            {
                // Get the Flyout from the element
                _githubFlyout = Microsoft.UI.Xaml.Controls.Primitives.FlyoutBase.GetAttachedFlyout(element);
                if (_githubFlyout != null)
                {
                    _githubFlyout.ShowAt(element);
                }
            }
        }

        private void OnGitHubHoverExited(object sender, Microsoft.UI.Xaml.Input.PointerRoutedEventArgs e)
        {
            // Flyout will auto-close when pointer leaves both the trigger and the flyout
        }

        private async void OnGitHubButtonClick(object sender, RoutedEventArgs e)
        {
            var uri = new Uri("https://github.com/AlienFamilyHub/ShikenMatrix");
            await Windows.System.Launcher.LaunchUriAsync(uri);
            
            // Close the flyout after opening the link
            if (_githubFlyout != null)
            {
                _githubFlyout.Hide();
            }
        }
    }
}
