[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$FirmwareRoot,

    [Parameter(Mandatory = $true)]
    [ValidateLength(1, 16)]
    [ValidatePattern('(?-i)^[a-z][a-z0-9]*(_[a-z0-9]+)*$')]
    [string]$Name
)

$ErrorActionPreference = 'Stop'
$rustKeywords = ('as break const continue crate else enum extern false fn for if impl in let ' +
    'loop match mod move mut pub ref return self static struct super trait true type unsafe use ' +
    'where while async await dyn abstract become box do final gen macro override priv try typeof ' +
    'unsized virtual yield union') -split ' '
if ($rustKeywords -contains $Name) {
    throw "'$Name' cannot be used as a Rust module name."
}

$resolvedRoot = (Resolve-Path -LiteralPath $FirmwareRoot).ProviderPath
$driverDirectory = Join-Path $resolvedRoot 'src/datalogger/src/drivers'
if (-not (Test-Path -LiteralPath (Join-Path $driverDirectory 'types.rs') -PathType Leaf)) {
    throw 'FirmwareRoot must be the firmware repository root containing src/datalogger/src/drivers/types.rs.'
}

$driverType = (($Name -split '_') | ForEach-Object {
    $_.Substring(0, 1).ToUpperInvariant() + $_.Substring(1)
}) -join ''
$templatePath = Join-Path $PSScriptRoot 'sensor.rs'
$text = [IO.File]::ReadAllText($templatePath)
$text = $text.Replace('ExampleSensor', $driverType).Replace('example_sensor', $Name)
$targetPath = Join-Path $driverDirectory ($Name + '.rs')

# CreateNew refuses to overwrite an existing driver, including in a race.
$stream = [IO.File]::Open($targetPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write)
try {
    $bytes = [Text.UTF8Encoding]::new($false).GetBytes($text)
    $stream.Write($bytes, 0, $bytes.Length)
} finally {
    $stream.Dispose()
}

Write-Output "Created $targetPath"
Write-Output "Driver: $driverType; settings: $($driverType)SpecialConfiguration; type name: $Name"
Write-Output 'The module declaration and registry entries still need to be added.'
