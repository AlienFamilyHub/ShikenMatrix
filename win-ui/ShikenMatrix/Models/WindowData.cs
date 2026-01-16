using System.ComponentModel;
using Microsoft.UI.Xaml.Media.Imaging;

namespace ShikenMatrix.Models
{
    /// <summary>
    /// Window data from the backend
    /// </summary>
    public class WindowData : INotifyPropertyChanged
    {
        private string _title = string.Empty;
        private string _processName = string.Empty;
        private uint _pid;
        private BitmapImage? _icon;

        public string Title
        {
            get => _title;
            set
            {
                if (_title != value)
                {
                    _title = value ?? string.Empty;
                    OnPropertyChanged(nameof(Title));
                }
            }
        }

        public string ProcessName
        {
            get => _processName;
            set
            {
                if (_processName != value)
                {
                    _processName = value ?? string.Empty;
                    OnPropertyChanged(nameof(ProcessName));
                }
            }
        }

        public uint Pid
        {
            get => _pid;
            set
            {
                if (_pid != value)
                {
                    _pid = value;
                    OnPropertyChanged(nameof(Pid));
                    OnPropertyChanged(nameof(PidDisplay));
                }
            }
        }

        public string PidDisplay => $"PID: {Pid}";

        public BitmapImage? Icon
        {
            get => _icon;
            set
            {
                if (_icon != value)
                {
                    _icon = value;
                    OnPropertyChanged(nameof(Icon));
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
