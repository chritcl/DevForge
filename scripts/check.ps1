Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# 切换到仓库根目录
$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Push-Location $RepoRoot

try {
    # 原生命令辅助函数：执行命令并检查退出码
    function Invoke-NativeStep {
        param(
            [Parameter(Mandatory)]
            [string]$Title,

            [Parameter(Mandatory)]
            [string]$Command,

            [string[]]$Arguments = @(),

            [string]$FailureMessage
        )

        Write-Host "`n=== $Title ===" -ForegroundColor Cyan

        & $Command @Arguments
        $exitCode = $LASTEXITCODE

        if ($exitCode -ne 0) {
            if ($FailureMessage) {
                throw "$FailureMessage（退出码：$exitCode）"
            }

            throw "$Title 失败（退出码：$exitCode）"
        }
    }

    # Windows MSVC 工具链预检
    function Assert-WindowsRustHost {
        if (-not $IsWindows) {
            return
        }

        Write-Host "`n=== Windows Rust 工具链检查 ===" -ForegroundColor Cyan

        $rustVersionOutput = & rustc --version --verbose
        $exitCode = $LASTEXITCODE

        if ($exitCode -ne 0) {
            throw "无法读取 Rust 工具链信息（退出码：$exitCode）"
        }

        $hostLine = $rustVersionOutput |
            Where-Object { $_ -match "^host:\s+(.+)$" } |
            Select-Object -First 1

        if (-not $hostLine) {
            throw "无法从 rustc --version --verbose 中识别 Rust host"
        }

        $rustHost = ($hostLine -replace "^host:\s+", "").Trim()

        if (-not $rustHost.EndsWith("-pc-windows-msvc")) {
            throw @"
当前 Windows Rust host 为：$rustHost

DevForge Windows 构建需要 MSVC Rust 工具链。
请执行：

rustup toolchain install 1.96.0-x86_64-pc-windows-msvc --profile minimal --component rustfmt --component clippy
rustup set default-host x86_64-pc-windows-msvc
rustup default 1.96.0-x86_64-pc-windows-msvc

重新打开 PowerShell 后再次运行 pnpm check。
"@
        }

        Write-Host "Rust host: $rustHost" -ForegroundColor Green
    }

    Assert-WindowsRustHost

    Invoke-NativeStep -Title "Rust 格式检查" -Command "cargo" -Arguments @("fmt", "--check")

    Invoke-NativeStep -Title "Rust Clippy" -Command "cargo" -Arguments @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")

    Invoke-NativeStep -Title "Rust 测试" -Command "cargo" -Arguments @("test", "--workspace")

    Invoke-NativeStep -Title "Rust 编译检查" -Command "cargo" -Arguments @("check", "--workspace")

    Invoke-NativeStep -Title "重新生成 Specta 绑定" -Command "pnpm" -Arguments @("bindings:generate")

    Invoke-NativeStep -Title "检查绑定是否最新" -Command "git" -Arguments @("diff", "--exit-code", "--", "apps/desktop/src/bindings.ts") -FailureMessage "apps/desktop/src/bindings.ts 与 Rust 类型不同步，请重新生成并提交最新 bindings"

    Invoke-NativeStep -Title "ESLint" -Command "pnpm" -Arguments @("--filter", "@devforge/desktop", "lint")

    Invoke-NativeStep -Title "TypeScript 类型检查" -Command "pnpm" -Arguments @("--filter", "@devforge/desktop", "typecheck")

    Invoke-NativeStep -Title "前端测试" -Command "pnpm" -Arguments @("--filter", "@devforge/desktop", "test")

    Invoke-NativeStep -Title "前端构建" -Command "pnpm" -Arguments @("--filter", "@devforge/desktop", "build")

    Invoke-NativeStep -Title "Git 空白检查" -Command "git" -Arguments @("diff", "--check")

    Write-Host "`n=== 全部质量检查通过 ===" -ForegroundColor Green
}
finally {
    Pop-Location
}
