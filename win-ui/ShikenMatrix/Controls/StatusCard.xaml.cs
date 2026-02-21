using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using ShikenMatrix.ViewModels;

namespace ShikenMatrix.Controls
{
    public sealed partial class StatusCard : UserControl
    {
        public StatusCard()
        {
            this.InitializeComponent();
        }

        private void OnToggleSwitchToggled(object sender, RoutedEventArgs e)
        {
            if (DataContext is MainViewModel viewModel)
            {
                viewModel.ToggleReporter();
            }
        }
    }
}
