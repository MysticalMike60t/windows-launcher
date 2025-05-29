using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Collections.Generic;
using Windows.System;
using Microsoft.UI.Windowing;
using Windows.Graphics;
using Windows.UI;

namespace windows_launcher
{
    public sealed partial class MainWindow : Window
    {
        private ObservableCollection<Category> categories = new();
        private ObservableCollection<Category> filteredCategories = new();
        private ObservableCollection<AppItem> matchingApps = new();
        private string defaultIcon;

        public MainWindow()
        {
            this.InitializeComponent();

            // Set dark theme
            RootGrid.RequestedTheme = ElementTheme.Dark;

            // Enable extending content into the title bar for custom drag region and background effect
            var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
            var windowId = Microsoft.UI.Win32Interop.GetWindowIdFromWindow(hwnd);
            var appWindow = AppWindow.GetFromWindowId(windowId);

            // Set size and position as you already do
            appWindow.Resize(new SizeInt32(900, 600));
            var displayArea = DisplayArea.GetFromWindowId(windowId, DisplayAreaFallback.Primary);
            appWindow.Move(new PointInt32(
                (displayArea.WorkArea.Width - 900) / 2,
                (displayArea.WorkArea.Height - 600) / 2
            ));

            // Style the title bar for Windows 11 look with transparent background
            var titleBar = appWindow.TitleBar;
            titleBar.ExtendsContentIntoTitleBar = true;

            var transparent = Color.FromArgb(0, 0, 0, 0);
            var white = Color.FromArgb(255, 255, 255, 255);
            var gray = Color.FromArgb(255, 128, 128, 128);

            titleBar.ButtonBackgroundColor = transparent;
            titleBar.ButtonInactiveBackgroundColor = transparent;
            titleBar.BackgroundColor = transparent;
            titleBar.InactiveBackgroundColor = transparent;
            titleBar.ButtonForegroundColor = white;
            titleBar.ButtonInactiveForegroundColor = gray;

            // Set your custom draggable region from XAML for the window titlebar
            this.SetTitleBar(CustomDragRegion);

            defaultIcon = Path.Combine(AppContext.BaseDirectory, "icons", "default.ico");
            LoadData();

            CategoryList.ItemsSource = filteredCategories;
            AppList.ItemsSource = null;

            CategoryList.SelectionChanged += CategoryList_SelectionChanged;
            SearchBox.TextChanged += SearchBox_TextChanged;
            AppList.DoubleTapped += AppList_DoubleTapped;
        }

        private void LoadData()
        {
            string jsonPath = Path.Combine(AppContext.BaseDirectory, "apps.json");
            if (!File.Exists(jsonPath))
            {
                // Optionally show a message or log
                return;
            }

            var json = File.ReadAllText(jsonPath);
            var data = JsonSerializer.Deserialize<LauncherData>(json);

            categories.Clear();
            if (data?.categories != null)
            {
                foreach (var category in data.categories)
                {
                    category.icon = ResolveIcon(category.icon);
                    foreach (var app in category.apps)
                    {
                        app.icon = ResolveIcon(app.icon);
                    }
                    categories.Add(category);
                }
            }
            filteredCategories.Clear();
            foreach (var cat in categories)
                filteredCategories.Add(cat);
        }

        private string? ResolveIcon(string? iconPath)
        {
            if (string.IsNullOrWhiteSpace(iconPath))
                return File.Exists(defaultIcon) ? defaultIcon : null;

            string fullPath = iconPath;
            if (!Path.IsPathRooted(iconPath))
                fullPath = Path.Combine(AppContext.BaseDirectory, iconPath.TrimStart('.', '\\', '/'));

            if (File.Exists(fullPath))
                return fullPath;
            if (File.Exists(defaultIcon))
                return defaultIcon;
            return null;
        }

        private void CategoryList_SelectionChanged(object sender, SelectionChangedEventArgs e)
        {
            if (CategoryList.SelectedItem is Category selectedCategory)
                AppList.ItemsSource = selectedCategory.apps;
            else
                AppList.ItemsSource = null;
        }

        private void SearchBox_TextChanged(object sender, TextChangedEventArgs e)
        {
            var query = SearchBox.Text?.ToLowerInvariant() ?? "";
            if (string.IsNullOrWhiteSpace(query))
            {
                filteredCategories.Clear();
                foreach (var cat in categories)
                    filteredCategories.Add(cat);
                AppList.ItemsSource = null;
            }
            else
            {
                filteredCategories.Clear();
                matchingApps.Clear();
                foreach (var category in categories)
                {
                    var filteredApps = category.apps
                        .Where(a => a.name?.ToLowerInvariant().Contains(query) == true)
                        .ToList();
                    if ((category.name?.ToLowerInvariant().Contains(query) == true) || filteredApps.Count > 0)
                    {
                        filteredCategories.Add(new Category
                        {
                            name = category.name,
                            icon = category.icon,
                            apps = new List<AppItem>(filteredApps)
                        });
                    }
                    foreach (var app in filteredApps)
                        matchingApps.Add(app);
                }
                CategoryList.ItemsSource = filteredCategories;
                AppList.ItemsSource = matchingApps;
            }
        }

        private async void AppList_DoubleTapped(object sender, DoubleTappedRoutedEventArgs e)
        {
            if (AppList.SelectedItem is AppItem selectedApp && !string.IsNullOrWhiteSpace(selectedApp.path))
            {
                try
                {
                    if (selectedApp.path.EndsWith(".exe", StringComparison.OrdinalIgnoreCase) && File.Exists(selectedApp.path))
                    {
                        System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
                        {
                            FileName = selectedApp.path,
                            UseShellExecute = true
                        });
                    }
                    else
                    {
                        await Launcher.LaunchUriAsync(new Uri(selectedApp.path));
                    }
                }
                catch
                {
                    // Optionally handle errors
                }
            }
        }
    }

    public class LauncherData
    {
        public List<Category> categories { get; set; }
    }

    public class Category
    {
        public string name { get; set; }
        public string? icon { get; set; }
        public List<AppItem> apps { get; set; }
    }

    public class AppItem
    {
        public string name { get; set; }
        public string? icon { get; set; }
        public string? path { get; set; }
        public string? description { get; set; }
    }
}