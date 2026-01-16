using System.ComponentModel;
using Microsoft.UI.Xaml.Media.Imaging;

namespace ShikenMatrix.Models
{
    /// <summary>
    /// Media data from the backend
    /// </summary>
    public class MediaData : INotifyPropertyChanged
    {
        private string _title = string.Empty;
        private string _artist = string.Empty;
        private string _album = string.Empty;
        private double _duration;
        private double _elapsedTime;
        private bool _playing;
        private BitmapImage? _artwork;

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

        public string Artist
        {
            get => _artist;
            set
            {
                if (_artist != value)
                {
                    _artist = value ?? string.Empty;
                    OnPropertyChanged(nameof(Artist));
                }
            }
        }

        public string Album
        {
            get => _album;
            set
            {
                if (_album != value)
                {
                    _album = value ?? string.Empty;
                    OnPropertyChanged(nameof(Album));
                }
            }
        }

        public double Duration
        {
            get => _duration;
            set
            {
                if (_duration != value)
                {
                    _duration = value;
                    OnPropertyChanged(nameof(Duration));
                }
            }
        }

        public double ElapsedTime
        {
            get => _elapsedTime;
            set
            {
                if (_elapsedTime != value)
                {
                    _elapsedTime = value;
                    OnPropertyChanged(nameof(ElapsedTime));
                }
            }
        }

        public bool Playing
        {
            get => _playing;
            set
            {
                if (_playing != value)
                {
                    _playing = value;
                    OnPropertyChanged(nameof(Playing));
                }
            }
        }

        public BitmapImage? Artwork
        {
            get => _artwork;
            set
            {
                if (_artwork != value)
                {
                    _artwork = value;
                    OnPropertyChanged(nameof(Artwork));
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
