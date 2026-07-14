$ErrorActionPreference = "SilentlyContinue"

try {
    $rawInput = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($rawInput)) {
        exit 0
    }

    $payload = $rawInput | ConvertFrom-Json
    $filePath = [string]$payload.tool_input.file_path

    if ([string]::IsNullOrWhiteSpace($filePath)) {
        exit 0
    }

    $projectDir = $env:CLAUDE_PROJECT_DIR
    if ([string]::IsNullOrWhiteSpace($projectDir)) {
        $projectDir = (Get-Location).Path
    }

    if ([System.IO.Path]::IsPathRooted($filePath)) {
        $fullPath = $filePath
    }
    else {
        $fullPath = Join-Path $projectDir $filePath
    }

    if (-not (Test-Path -LiteralPath $fullPath -PathType Leaf)) {
        exit 0
    }

    $extension = [System.IO.Path]::GetExtension($fullPath).ToLowerInvariant()
    $prettierExtensions = @(
        ".js", ".jsx", ".ts", ".tsx", ".vue",
        ".json", ".jsonc", ".css", ".scss", ".less",
        ".md", ".yaml", ".yml", ".html"
    )

    Push-Location $projectDir

    if ($prettierExtensions -contains $extension) {
        $prettierCmd = Join-Path $projectDir "node_modules/.bin/prettier.cmd"
        $prettierNoExt = Join-Path $projectDir "node_modules/.bin/prettier"

        if (Test-Path -LiteralPath $prettierCmd) {
            & $prettierCmd --write -- $fullPath | Out-Null
        }
        elseif (Test-Path -LiteralPath $prettierNoExt) {
            & $prettierNoExt --write -- $fullPath | Out-Null
        }
    }
    elseif ($extension -eq ".rs") {
        if ((Test-Path -LiteralPath (Join-Path $projectDir "Cargo.toml")) -and
            (Get-Command cargo -ErrorAction SilentlyContinue)) {
            & cargo fmt --all --quiet | Out-Null
        }
    }

    Pop-Location
    exit 0
}
catch {
    try {
        Pop-Location
    }
    catch {
    }

    # 自动格式化失败不应阻断 Claude；正式验证仍由 verifier 负责。
    [Console]::Error.WriteLine("自动格式化 Hook 跳过：$($_.Exception.Message)")
    exit 0
}
