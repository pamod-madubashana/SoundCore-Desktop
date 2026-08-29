# Quick probe: black color only for all models
$CDN = "https://d2htfo7ft368vg.cloudfront.net"

$models = @(
    "a3004","a3027","a3028","a3029","a3030","a3031","a3033","a3035",
    "a3040","a3062","a3116","a3909","a3926","a3930","a3931","a3933",
    "a3935","a3936","a3939","a3945","a3947","a3948","a3949","a3951",
    "a3952","a3955","a3957","a3959","a3968"
)

$found = @()
$notfound = @()

foreach ($model in $models) {
    $url = "$CDN/media/prod/$model/black/${model}_black.zip"
    try {
        $resp = Invoke-WebRequest -Uri $url -Method Head -UseBasicParsing -TimeoutSec 3 -ErrorAction Stop
        Write-Host "OK:   $model" -ForegroundColor Green
        $found += $model
    } catch {
        $code = $_.Exception.Response.StatusCode.value__
        Write-Host "MISS: $model ($code)" -ForegroundColor Red
        $notfound += $model
    }
}

Write-Host "`nFound: $($found.Count) / $($models.Count)" -ForegroundColor Cyan
Write-Host "Models with images: $($found -join ', ')" -ForegroundColor Green
Write-Host "Models without: $($notfound -join ', ')" -ForegroundColor Yellow

# Save found models
$found | Out-File "E:\User\Documents\Repositories\soundcore\Auto-Mode\tools\models-with-images.txt"
