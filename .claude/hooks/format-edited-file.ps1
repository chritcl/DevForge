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

    # 解析绝对路径
    if ([System.IO.Path]::IsPathRooted($filePath)) {
        $fullPath = $filePath
    }
    else {
        $fullPath = Join-Path $projectDir $filePath
    }

    # 路径安全检查：规范化后必须仍在项目目录内
    $normalizedPath = [System.IO.Path]::GetFullPath($fullPath)
    $normalizedProject = [System.IO.Path]::GetFullPath($projectDir)
    if (-not $normalizedPath.StartsWith($normalizedProject, [System.StringComparison]::OrdinalIgnoreCase)) {
        [Console]::Error.WriteLine("格式化 Hook 跳过：路径超出项目目录")
        exit 0
    }

    if (-not (Test-Path -LiteralPath $normalizedPath -PathType Leaf)) {
        exit 0
    }

    $extension = [System.IO.Path]::GetExtension($normalizedPath).ToLowerInvariant()

    # 前端文件使用 Prettier 单文件格式化
    $prettierExtensions = @(
        ".js", ".jsx", ".ts", ".tsx", ".vue",
        ".json", ".jsonc", ".css", ".scss", ".less",
        ".md", ".yaml", ".yml", ".html"
    )

    if ($prettierExtensions -contains $extension) {
        Push-Location $projectDir

        $prettierCmd = Join-Path $projectDir "node_modules/.bin/prettier.cmd"
        $prettierNoExt = Join-Path $projectDir "node_modules/.bin/prettier"

        if (Test-Path -LiteralPath $prettierCmd) {
            & $prettierCmd --write -- $normalizedPath | Out-Null
        }
        elseif (Test-Path -LiteralPath $prettierNoExt) {
            & $prettierNoExt --write -- $normalizedPath | Out-Null
        }

        Pop-Location
    }

    # Rust 文件不在编辑时自动格式化整个 Workspace。
    # 格式化由 verify-task Skill 或手动 cargo fmt 负责。

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
