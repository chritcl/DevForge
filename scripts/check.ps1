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
