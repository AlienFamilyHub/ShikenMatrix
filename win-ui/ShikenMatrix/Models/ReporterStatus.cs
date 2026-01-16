using System.ComponentModel;

namespace ShikenMatrix.Models
{
    /// <summary>
    /// Status model for the reporter
    /// </summary>
    public class ReporterStatus : INotifyPropertyChanged
    {
        private bool _isRunning;
        private bool _isConnected;
        private string? _lastError;

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
                }
            }
        }

        public event PropertyChangedEventHandler? PropertyChanged;

        protected void OnPropertyChanged(string propertyName)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
