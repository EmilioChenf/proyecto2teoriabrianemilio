param([switch]$SoloEjemplos)

$ErrorActionPreference = 'Stop'
$originalPath = $env:Path
$originalEncoding = [Console]::OutputEncoding
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding
Push-Location (Join-Path $PSScriptRoot '..')

try {
    $cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargoCommand) {
        $cargoDirectory = Join-Path $env:USERPROFILE '.cargo\bin'
        if (-not (Test-Path -LiteralPath (Join-Path $cargoDirectory 'cargo.exe'))) {
            throw 'No se encontro Cargo. Instale Rust con rustup.'
        }
        $env:Path = $cargoDirectory + ';' + $env:Path
    }

    function Invoke-CargoCheck {
        param([string[]]$Arguments)
        & cargo @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "Fallo cargo $($Arguments -join ' '), codigo $LASTEXITCODE"
        }
    }

    if (-not $SoloEjemplos) {
        Invoke-CargoCheck -Arguments @('fmt', '--check')
        Invoke-CargoCheck -Arguments @('check')
        Invoke-CargoCheck -Arguments @('test')
        Invoke-CargoCheck -Arguments @('clippy', '--all-targets', '--all-features', '--', '-D', 'warnings')
        Invoke-CargoCheck -Arguments @('build', '--release')
    }
    Invoke-CargoCheck -Arguments @('build')
    $executable = Join-Path (Get-Location) 'target\debug\cyk.exe'
    $cases = @()
    $accepted = @('id', 'id + id', 'id * id', 'id * id + id', '(id)', '(id + id)', '(id * id) + id')
    $rejected = @('+ id', 'id +', 'id * + id', '( id', 'id id')
    foreach ($file in @('1.txt', '1-cnf.txt')) {
        foreach ($expression in $accepted) {
            $cases += @{ File = $file; Expression = $expression; Code = 0; Flags = @() }
        }
        foreach ($expression in $rejected) {
            $cases += @{ File = $file; Expression = $expression; Code = 1; Flags = @() }
        }
    }
    $cases += @(
        @{ File = 'examples/english.txt'; Expression = 'she eats a cake with a fork'; Code = 0; Flags = @() },
        @{ File = 'examples/english.txt'; Expression = 'the cat drinks the beer'; Code = 0; Flags = @() },
        @{ File = 'examples/english.txt'; Expression = 'she a eats cake'; Code = 1; Flags = @() },
        @{ File = 'examples/english.txt'; Expression = 'the cat the beer drinks'; Code = 1; Flags = @() },
        @{ File = 'examples/english.txt'; Expression = 'She eats a cake with a fork'; Code = 0; Flags = @('--ignore-case') },
        @{ File = 'examples/epsilon.txt'; Expression = ''; Code = 0; Flags = @() },
        @{ File = 'examples/epsilon.txt'; Expression = 'a a a'; Code = 0; Flags = @() },
        @{ File = 'examples/useless.txt'; Expression = 'c'; Code = 1; Flags = @() },
        @{ File = '1.txt'; Expression = 'id @ id'; Code = 2; Flags = @() }
    )

    $rows = foreach ($case in $cases) {
        $caseArguments = @($case.Flags) + @($case.File)
        $previousErrorPolicy = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        try {
            if ($case.Expression -eq '') {
                # Una linea vacia por stdin funciona tambien en PowerShell 5.1.
                $caseOutput = '' | & $executable @caseArguments 2>&1
            } else {
                $caseArguments += $case.Expression
                $caseOutput = & $executable @caseArguments 2>&1
            }
            $code = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $previousErrorPolicy
        }
        $text = ($caseOutput | ForEach-Object { "$_" }) -join "`n"
        $validResult = switch ($case.Code) {
            0 { $text -match 'Resultado CYK: S' }
            1 { $text -match 'Resultado CYK: NO' }
            2 { $text -match 'Error:' }
        }
        if ($code -ne $case.Code -or -not $validResult) {
            throw "Resultado incorrecto: $($case.File), '$($case.Expression)', esperado $($case.Code), obtenido $code`n$text"
        }
        [PSCustomObject]@{
            Archivo = $case.File
            Entrada = $(if ($case.Expression -eq '') { '(epsilon)' } else { $case.Expression })
            Esperado = $case.Code
            Obtenido = $code
            Estado = 'PASS'
        }
    }
    $rows | Format-Table -AutoSize
    Write-Output "PASS: $($rows.Count) casos de consola correctos. Codigos: 0=SI, 1=NO, 2=error."
} finally {
    Pop-Location
    $env:Path = $originalPath
    [Console]::OutputEncoding = $originalEncoding
}
