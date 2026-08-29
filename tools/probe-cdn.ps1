# Probe Soundcore CDN for all model+color combinations
# Outputs CSV: model,color,url,status

$CDN = "https://d2htfo7ft368vg.cloudfront.net"

$models = @(
    "a3004","a3027","a3028","a3029","a3030","a3031","a3033","a3035",
    "a3040","a3062","a3116","a3909","a3926","a3930","a3931","a3933",
    "a3935","a3936","a3939","a3945","a3947","a3948","a3949","a3951",
    "a3952","a3955","a3957","a3959","a3968"
)

$colors = @(
    "black","white","blue","silver","pink","green","yellow","brown",
    "red","gray","golden","sand_white","navy_blue","coral","sky_blue",
    "watermelonred","khaki","babyblue","lightgray","bluesilver",
    "redgolden","lightblue","purple","camouflage","creamywhite",
    "black_se","white_red","purple_yellow","black_golden",
    "yellow_green","black_pure","green_pure","blue_pure"
)

$results = @()

foreach ($model in $models) {
    foreach ($color in $colors) {
        $url = "$CDN/media/prod/$model/$color/${model}_${color}.zip"
        try {
            $resp = Invoke-WebRequest -Uri $url -Method Head -UseBasicParsing -TimeoutSec 5 -ErrorAction Stop
            if ($resp.StatusCode -eq 200) {
                Write-Host "FOUND: $model $color" -ForegroundColor Green
                $results += [PSCustomObject]@{ Model=$model; Color=$color; Url=$url; Status=$resp.StatusCode }
            }
        } catch {
            # 404 or error - skip silently
        }
    }
}

Write-Host "`n=== Results ===" -ForegroundColor Cyan
Write-Host "Found $($results.Count) valid image(s)"
$results | Format-Table -AutoSize
$results | Export-Csv -Path "E:\User\Documents\Repositories\soundcore\Auto-Mode\tools\cdn-results.csv" -NoTypeInformation
