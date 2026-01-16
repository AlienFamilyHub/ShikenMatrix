using System.ComponentModel;

namespace ShikenMatrix.Models
{
    /// <summary>
    /// Configuration model for the reporter
    /// </summary>
    public class ReporterConfig : INotifyPropertyChanged
    {
        private bool _enabled;
        private string _wsUrl = string.Empty;
        private string _token = string.Empty;
        private bool _enableMediaReporting;

        public bool Enabled
        {
            get => _enabled;
            set
            {
                if (_enabled != value)
                {
                    _enabled = value;
                    OnPropertyChanged(nameof(Enabled));
                }
            }
        }

        public string WsUrl
        {
            get => _wsUrl;
            set
            {
                if (_wsUrl != value)
                {
                    _wsUrl = value ?? string.Empty;
                    OnPropertyChanged(nameof(WsUrl));
                }
            }
        }

        public string Token
        {
            get => _token;
            set
            {
                if (_token != value)
                {
                    _token = value ?? string.Empty;
                    OnPropertyChanged(nameof(Token));
                }
            }
        }

        public bool EnableMediaReporting
        {
            get => _enableMediaReporting;
            set
            {
                if (_enableMediaReporting != value)
                {
                    _enableMediaReporting = value;
                    OnPropertyChanged(nameof(EnableMediaReporting));
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
