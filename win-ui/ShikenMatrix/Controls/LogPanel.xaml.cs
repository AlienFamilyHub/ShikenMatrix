using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using ShikenMatrix.ViewModels;

namespace ShikenMatrix.Controls
{
    public sealed partial class LogPanel : UserControl
    {
        public LogPanel()
        {
            this.InitializeComponent();
        }

        private void OnClearLogsClick(object sender, RoutedEventArgs e)
        {
            if (DataContext is MainViewModel viewModel)
            {
                viewModel.ClearLogs();
            }
        }
    }
}
