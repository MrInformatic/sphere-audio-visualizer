function Install-Software {
	[CmdletBinding()]
	param(
		[Parameter(Mandatory)]
		[string]$Uri
	)

	Invoke-WebRequest -OutFile "installer.msi" -Uri "$Uri"

    $Args = @(
        "/i" 
        "installer.msi"
        "/qn"
    )
    Start-Process "msiexec.exe" -ArgumentList $Args -Wait -NoNewWindow

    Remove-Item "installer.msi"
}

Install-Software -Uri "https://gstreamer.freedesktop.org/data/pkg/windows/$Env:GST_VERSION/msvc/gstreamer-1.0-msvc-x86_64-$Env:GST_VERSION.msi"
Install-Software -Uri "https://gstreamer.freedesktop.org/data/pkg/windows/$Env:GST_VERSION/msvc/gstreamer-1.0-msvc-x86-$Env:GST_VERSION.msi"
Install-Software -Uri "https://gstreamer.freedesktop.org/data/pkg/windows/$Env:GST_VERSION/msvc/gstreamer-1.0-devel-msvc-x86_64-$Env:GST_VERSION.msi"
Install-Software -Uri "https://gstreamer.freedesktop.org/data/pkg/windows/$Env:GST_VERSION/msvc/gstreamer-1.0-devel-msvc-x86-$Env:GST_VERSION.msi"

$Env:Path = "C:\gstreamer\1.0\msvc_x86_64\bin;$Env:Path"
$Env:PKG_CONFIG_PATH = "C:\gstreamer\1.0\msvc_x86_64\lib\pkgconfig;$Env:PKG_CONFIG_PATH"

Add-Content -Path "$Env:GITHUB_ENV" -Value "Path=$Env:Path"
Add-Content -Path "$Env:GITHUB_ENV" -Value "PKG_CONFIG_PATH=$Env:PKG_CONFIG_PATH"