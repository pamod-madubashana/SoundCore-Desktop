# Download all valid Soundcore device images from CDN
# Extracts *_com_device.png from ZIPs into ui/public/devices/

Add-Type -AssemblyName System.IO.Compression.FileSystem

$CDN = "https://d2htfo7ft368vg.cloudfront.net"
$destDir = "E:\User\Documents\Repositories\soundcore\Auto-Mode\ui\public\devices"
$tmpDir = "$env:TEMP\soundcore-img-dl"
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

$entries = @(
    "a3933,black","a3933,white","a3933,blue","a3933,pink","a3933,green",
    "a3936,black","a3936,white","a3936,blue","a3936,green","a3936,purple",
    "a3939,black","a3939,blue","a3939,red","a3939,watermelonred","a3939,khaki",
    "a3939,babyblue","a3939,lightgray","a3939,bluesilver","a3939,redgolden",
    "a3945,black","a3945,white",
    "a3947,black","a3947,white","a3947,pink","a3947,navy_blue","a3947,lightblue",
    "a3948,black","a3948,white","a3948,blue","a3948,green","a3948,gray","a3948,purple",
    "a3949,black","a3949,white","a3949,blue","a3949,pink","a3949,gray","a3949,lightblue",
    "a3955,black","a3955,blue","a3955,purple","a3955,creamywhite",
    "a3957,black","a3957,white","a3957,blue","a3957,golden","a3957,creamywhite",
    "a3959,black","a3959,white","a3959,blue","a3959,pink","a3959,green","a3959,yellow",
    "a3968,black","a3968,white","a3968,green"
)

$ok = 0; $fail = 0; $total = $entries.Count
foreach ($entry in $entries) {
    $parts = $entry.Split(",")
    $model = $parts[0]; $color = $parts[1]
    $url = "$CDN/media/prod/$model/$color/${model}_${color}.zip"
    $zipPath = "$tmpDir\${model}_${color}.zip"
    $pngName = "${model}_${color}_com_device.png"
    $pngPath = "$destDir\$pngName"

    if (Test-Path $pngPath) {
        Write-Host "SKIP (exists): $pngName" -ForegroundColor DarkGray
        $ok++
        continue
    }

    try {
        $wc = New-Object System.Net.WebClient
        $wc.Headers.Add("User-Agent", "okhttp/4.9.3")
        $wc.DownloadFile($url, $zipPath)

        $zip = [System.IO.Compression.ZipFile]::OpenRead($zipPath)
        $found = $false
        foreach ($e in $zip.Entries) {
            if ($e.Name -like "*_com_device.png" -and $e.FullName -notlike "*__MACOSX*") {
                [System.IO.Compression.ZipFileExtensions]::ExtractToFile($e, $pngPath, $true)
                $found = $true
                break
            }
        }
        if (-not $found) {
            foreach ($e in $zip.Entries) {
                if ($e.Name -like "*.png" -and $e.FullName -notlike "*__MACOSX*" -and $e.FullName -notlike "*._") {
                    [System.IO.Compression.ZipFileExtensions]::ExtractToFile($e, $pngPath, $true)
                    $found = $true
                    break
                }
            }
        }
        $zip.Dispose()
        Remove-Item $zipPath -Force -ErrorAction SilentlyContinue

        if ($found) {
            Write-Host "OK: $pngName" -ForegroundColor Green
            $ok++
        } else {
            Write-Host "NO PNG in zip: $model/$color" -ForegroundColor Yellow
            $fail++
        }
    } catch {
        Write-Host "FAIL: $model/$color - $($_.Exception.Message)" -ForegroundColor Red
        $fail++
    }
    Start-Sleep -Milliseconds 300
}

Write-Host "`n=== Done: $ok OK, $fail failed out of $total ===" -ForegroundColor Cyan
Write-Host "Images saved to: $destDir"
Get-ChildItem $destDir -Filter "*.png" | Measure-Object | Select-Object -ExpandProperty Count | ForEach-Object { Write-Host "$_ PNG files in devices/" }
