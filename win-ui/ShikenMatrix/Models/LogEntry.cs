using System;
using System.ComponentModel;
using ShikenMatrix.Native;

namespace ShikenMatrix.Models
{
    /// <summary>
    /// Log entry model
    /// </summary>
    public class LogEntry : INotifyPropertyChanged
    {
        private DateTime _timestamp;
        private string _message = string.Empty;
        private SmLogLevel _level;

        public DateTime Timestamp
        {
            get => _timestamp;
            set
            {
                if (_timestamp != value)
                {
                    _timestamp = value;
                    OnPropertyChanged(nameof(Timestamp));
                }
            }
        }

        public string Message
        {
            get => _message;
            set
            {
                if (_message != value)
                {
                    _message = value ?? string.Empty;
                    OnPropertyChanged(nameof(Message));
                }
            }
        }

        public SmLogLevel Level
        {
            get => _level;
            set
            {
                if (_level != value)
                {
                    _level = value;
                    OnPropertyChanged(nameof(Level));
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
