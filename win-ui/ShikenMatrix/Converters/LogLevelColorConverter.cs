using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Data;
using ShikenMatrix.Models;
using ShikenMatrix.Native;

namespace ShikenMatrix.Converters
{
    /// <summary>
    /// Convert log level to color brush
    /// </summary>
    public class LogLevelColorConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is SmLogLevel level)
            {
                var color = level switch
                {
                    SmLogLevel.Info => Microsoft.UI.ColorHelper.FromArgb(255, 0, 120, 215),   // Blue
                    SmLogLevel.Warning => Microsoft.UI.ColorHelper.FromArgb(255, 255, 140, 0),  // Orange
                    SmLogLevel.Error => Microsoft.UI.ColorHelper.FromArgb(255, 232, 17, 35),    // Red
                    _ => Microsoft.UI.Colors.Gray
                };
                return new Microsoft.UI.Xaml.Media.SolidColorBrush(color);
            }
            return new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Gray);
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// Convert log level to icon symbol
    /// </summary>
    public class LogLevelIconConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is SmLogLevel level)
            {
                return level switch
                {
                    SmLogLevel.Info => "\uE946",      // Circle
                    SmLogLevel.Warning => "\uE9BE",   // Triangle
                    SmLogLevel.Error => "\uE948",     // XCircle
                    _ => "\uE946"
                };
            }
            return "\uE946";
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// Convert bool to visibility
    /// </summary>
    public class BoolToVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool boolValue)
            {
                // If parameter is "Invert", reverse the logic
                if (parameter?.ToString()?.Equals("Invert", StringComparison.OrdinalIgnoreCase) == true)
                {
                    return boolValue ? Visibility.Collapsed : Visibility.Visible;
                }
                return boolValue ? Visibility.Visible : Visibility.Collapsed;
            }
            return Visibility.Collapsed;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            if (value is Visibility visibility)
            {
                bool invert = parameter?.ToString()?.Equals("Invert", StringComparison.OrdinalIgnoreCase) == true;
                if (invert)
                {
                    return visibility != Visibility.Visible;
                }
                return visibility == Visibility.Visible;
            }
            return false;
        }
    }

    /// <summary>
    /// Convert string color name to brush
    /// </summary>
    public class ColorNameToBrushConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is string colorName)
            {
                return colorName.ToLowerInvariant() switch
                {
                    "green" => new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Green),
                    "orange" => new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Orange),
                    "gray" or "grey" => new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Gray),
                    "red" => new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Red),
                    _ => new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Gray)
                };
            }
            return new Microsoft.UI.Xaml.Media.SolidColorBrush(Microsoft.UI.Colors.Gray);
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// Convert timestamp to formatted time string
    /// </summary>
    public class TimestampFormatConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is DateTime dateTime)
            {
                return dateTime.ToString("HH:mm:ss.fff");
            }
            return string.Empty;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// Invert a boolean value
    /// </summary>
    public class BoolInvertConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool boolValue)
            {
                return !boolValue;
            }
            return true;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            if (value is bool boolValue)
            {
                return !boolValue;
            }
            return false;
        }
    }

    /// <summary>
    /// Convert null to visibility (Collapsed if null, Visible if not null)
    /// Supports "Inverse" parameter to reverse logic (Visible if null, Collapsed if not null)
    /// </summary>
    public class NullToVisibilityConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            bool isNull = value == null;

            if (parameter?.ToString()?.Equals("Inverse", StringComparison.OrdinalIgnoreCase) == true)
            {
                return isNull ? Visibility.Visible : Visibility.Collapsed;
            }

            return isNull ? Visibility.Collapsed : Visibility.Visible;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// Check if string is not empty
    /// </summary>
    public class StringNotEmptyConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is string str)
            {
                return !string.IsNullOrWhiteSpace(str);
            }
            return false;
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }

    /// <summary>
    /// Convert boolean to Accent color brush
    /// </summary>
    public class BoolToAccentConverter : IValueConverter
    {
        public object Convert(object value, Type targetType, object parameter, string language)
        {
            if (value is bool b && b)
            {
                return Application.Current.Resources["SystemControlHighlightAccentBrush"];
            }
            return Application.Current.Resources["SystemControlForegroundBaseHighBrush"];
        }

        public object ConvertBack(object value, Type targetType, object parameter, string language)
        {
            throw new NotImplementedException();
        }
    }
}

