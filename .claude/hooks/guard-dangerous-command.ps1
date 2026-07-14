$ErrorActionPreference = "Stop"

try {
    $rawInput = [Console]::In.ReadToEnd()
    if ([string]::IsNullOrWhiteSpace($rawInput)) {
        exit 0
    }

    $payload = $rawInput | ConvertFrom-Json
    $command = [string]$payload.tool_input.command

    if ([string]::IsNullOrWhiteSpace($command)) {
        exit 0
    }

    $blockedPatterns = @(
        '(?i)\bgit\s+reset\s+--hard\b',
        '(?i)\bgit\s+clean\s+[^;&|]*-[a-z]*f[a-z]*\b',
        '(?i)\bgit\s+push\b[^;&|]*(--force|-f)\b',
        '(?i)\brm\s+-rf\s+([/~.]|\$HOME)(\s|$)',
        '(?i)\bRemove-Item\b[^;&|]*(C:\\|[A-Z]:\\|~|\$HOME)[^;&|]*-Recurse[^;&|]*-Force',
        '(?i)\bformat(\.com)?\s+[A-Z]:',
        '(?i)\b(shutdown|Stop-Computer|Restart-Computer)\b'
    )

    foreach ($pattern in $blockedPatterns) {
        if ($command -match $pattern) {
            [Console]::Error.WriteLine(
                "已阻止高风险命令。请改用更小范围、可恢复的操作；如确需执行，请由用户明确确认后手动完成。"
            )
            exit 2
        }
    }

    exit 0
}
catch {
    # Hook 自身解析失败时不应中断正常开发流程。
    [Console]::Error.WriteLine("危险命令检查 Hook 解析失败：$($_.Exception.Message)")
    exit 0
}
