# Generates iconpush-test.mobileconfig: a few Web Clips with different settings,
# to find out how iOS 27 handles app URL schemes before building the real tool.
Add-Type -AssemblyName System.Drawing

function New-Icon([string]$text, [string]$hex) {
    $bmp = New-Object System.Drawing.Bitmap 180, 180
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'
    $g.TextRenderingHint = 'AntiAliasGridFit'
    $g.Clear([System.Drawing.ColorTranslator]::FromHtml($hex))
    $font = New-Object System.Drawing.Font 'Segoe UI', 54, ([System.Drawing.FontStyle]::Bold)
    $fmt = New-Object System.Drawing.StringFormat
    $fmt.Alignment = 'Center'
    $fmt.LineAlignment = 'Center'
    $g.DrawString($text, $font, [System.Drawing.Brushes]::White, (New-Object System.Drawing.RectangleF 0, 0, 180, 180), $fmt)
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $bmp.Dispose()
    [Convert]::ToBase64String($ms.ToArray())
}

# Label, URL, FullScreen, icon text, color, note (shown in the profile details)
$clips = @(
    @('Spot A',  'spotify://',             $false, 'S-A', '#1DB954', 'Schema simple'),
    @('Spot B',  'spotify://',             $true,  'S-B', '#168d40', 'Schema + FullScreen'),
    @('Insta',   'instagram://',           $false, 'IG',  '#C13584', 'Schema simple'),
    @('WhatsApp','whatsapp://',            $false, 'WA',  '#25D366', 'Schema simple'),
    @("R$([char]0xE9)glages", 'App-prefs:', $false, 'Set', '#8E8E93', 'Icone systeme'),
    @('Photos',  'photos-redirect://',     $false, 'Ph',  '#FF9500', 'App Apple')
)

$payloads = foreach ($c in $clips) {
    $uuid = [guid]::NewGuid().ToString().ToUpper()
    $full = if ($c[2]) { '<true/>' } else { '<false/>' }
    $label = [System.Security.SecurityElement]::Escape($c[0])
    $url = [System.Security.SecurityElement]::Escape($c[1])
@"
		<dict>
			<key>FullScreen</key>
			$full
			<key>Icon</key>
			<data>$(New-Icon $c[3] $c[4])</data>
			<key>IsRemovable</key>
			<true/>
			<key>Label</key>
			<string>$label</string>
			<key>PayloadDescription</key>
			<string>$($c[5])</string>
			<key>PayloadDisplayName</key>
			<string>$label</string>
			<key>PayloadIdentifier</key>
			<string>dev.iconpush.test.$uuid</string>
			<key>PayloadType</key>
			<string>com.apple.webClip.managed</string>
			<key>PayloadUUID</key>
			<string>$uuid</string>
			<key>PayloadVersion</key>
			<integer>1</integer>
			<key>Precomposed</key>
			<true/>
			<key>URL</key>
			<string>$url</string>
		</dict>
"@
}

$root = [guid]::NewGuid().ToString().ToUpper()
$xml = @"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>PayloadContent</key>
	<array>
$($payloads -join "`n")
	</array>
	<key>PayloadDescription</key>
	<string>Profil de test iconpush : icones personnalisees qui ouvrent des apps.</string>
	<key>PayloadDisplayName</key>
	<string>iconpush - test</string>
	<key>PayloadIdentifier</key>
	<string>dev.iconpush.test</string>
	<key>PayloadOrganization</key>
	<string>iconpush</string>
	<key>PayloadRemovalDisallowed</key>
	<false/>
	<key>PayloadType</key>
	<string>Configuration</string>
	<key>PayloadUUID</key>
	<string>$root</string>
	<key>PayloadVersion</key>
	<integer>1</integer>
</dict>
</plist>
"@

$out = Join-Path $PSScriptRoot 'iconpush-test.mobileconfig'
[IO.File]::WriteAllText($out, $xml, (New-Object System.Text.UTF8Encoding $false))
"OK -> $out ($([math]::Round((Get-Item $out).Length / 1KB, 1)) Ko)"
