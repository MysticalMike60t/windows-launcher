Add-Type -AssemblyName PresentationFramework

[xml]$xaml = Get-Content -Raw -Path '.\window.xaml'
$reader = New-Object System.Xml.XmlNodeReader $xaml
$window = [Windows.Markup.XamlReader]::Load($reader)

$categoryList = $window.FindName('CategoryList')
$appList = $window.FindName('AppList')
$searchBox = $window.FindName('SearchBox')

$jsonText = Get-Content -Raw -Path '.\apps.json'
$data = ConvertFrom-Json $jsonText

$defaultIcon = Join-Path $PSScriptRoot 'icons\default.ico'

foreach ($category in $data.categories) {
  $catIconPath = Resolve-Path -LiteralPath $category.icon -ErrorAction SilentlyContinue
  if ($catIconPath) {
    $category.icon = $catIconPath.ProviderPath
  }
  elseif (Test-Path $defaultIcon) {
    $category.icon = $defaultIcon
  }
  else {
    $category.icon = $null
  }
  foreach ($app in $category.apps) {
    $iconPath = Resolve-Path -LiteralPath $app.icon -ErrorAction SilentlyContinue
    if ($iconPath) {
      $app.icon = $iconPath.ProviderPath
    }
    elseif (Test-Path $defaultIcon) {
      $app.icon = $defaultIcon
    }
    else {
      $app.icon = $null
    }
  }
}

$categories = $data.categories

$categoryList.ItemsSource = $categories

$categoryList.Add_SelectionChanged({
    $selectedCategory = $categoryList.SelectedItem
    if ($null -ne $selectedCategory) {
      $appList.ItemsSource = $selectedCategory.apps
    }
    else {
      $appList.ItemsSource = $null
    }
  })

$searchBox.Add_TextChanged({
    $query = $searchBox.Text.ToLower()

    if ([string]::IsNullOrWhiteSpace($query)) {
      $categoryList.ItemsSource = $categories
      $appList.ItemsSource = $null
    }
    else {
      $filteredCategories = @()
      $matchingApps = @()
      foreach ($category in $categories) {
        $filteredApps = $category.apps | Where-Object { $_.name.ToLower().Contains($query) }
        if ($category.name.ToLower().Contains($query) -or $filteredApps.Count -gt 0) {
          $catClone = $category.PSObject.Copy()
          $catClone.apps = @($filteredApps)
          $filteredCategories += $catClone
        }
        $matchingApps += $filteredApps
      }
      $categoryList.ItemsSource = $filteredCategories
      $appList.ItemsSource = $matchingApps
    }
  })

$appList.Add_MouseDoubleClick({
    $selectedApp = $appList.SelectedItem
    if ($null -ne $selectedApp) {
      $cmd = $selectedApp.command
      if (-not $cmd -and $selectedApp.path) {
        $cmd = $selectedApp.path
      }
      if ($cmd) {
        # Resolve relative path to absolute
        $exePath = Resolve-Path -LiteralPath $cmd -ErrorAction SilentlyContinue
        if ($exePath) {
          Start-Process -FilePath $exePath.ProviderPath
        } else {
          # If not found, try as-is (may be in PATH)
          Start-Process -FilePath $cmd
        }
      }
    }
  })

$closeBtn = $window.FindName('CloseButton')
$minBtn = $window.FindName('MinimizeButton')

$closeBtn.Add_Click({ $window.Close() })
$minBtn.Add_Click({ $window.WindowState = 'Minimized' })

$titleBar = $window.Content.Children[0]
$titleBar.Add_MouseLeftButtonDown({
    param($s, $e)
    if ($e.ButtonState -eq 'Pressed') {
      $window.DragMove()
    }
  })

$window.ShowDialog() | Out-Null
