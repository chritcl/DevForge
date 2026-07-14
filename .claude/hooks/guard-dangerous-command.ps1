# 防御层次说明：
#
# 第一层：settings.json 的 deny（硬阻止）和 ask（需确认）规则。
#         这是主要的安全控制，基于命令前缀匹配。
#
# 第二层：本 Hook，基于正则模式匹配高风险命令组合。
#         作为 defense-in-depth 补充，不是完整沙箱。
#
# 已知绕过方式（本 Hook 无法覆盖）：
#   - cmd /c 包装
#   - PowerShell 别名和函数重定义
#   - 调用外部脚本文件
#   - 编码命令（Base64、EncodedCommand）
#   - 间接 package script（npm/pnpm run）
#   - 其他删除工具（del、rd、sdelete 等）
#
# 因此：不要依赖本 Hook 作为唯一安全边界。
# 安全决策应始终在 Rust 策略层做出。

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
    # 解析失败时 fail-closed：阻止命令执行并输出醒目警告。
    # 本 Hook 是 defense-in-depth，解析异常不应静默放行。
    # 如需临时放行，用户可手动执行命令。
    [Console]::Error.WriteLine("========================================")
    [Console]::Error.WriteLine("[安全警告] 危险命令检查 Hook 解析失败")
    [Console]::Error.WriteLine("原因：$($_.Exception.Message)")
    [Console]::Error.WriteLine("命令已被阻止。如确认安全，请用户手动执行。")
    [Console]::Error.WriteLine("========================================")
    exit 2
}
