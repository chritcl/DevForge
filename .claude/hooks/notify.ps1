$ErrorActionPreference = "SilentlyContinue"

try {
    $rawInput = [Console]::In.ReadToEnd()
    $message = "Claude Code 已完成当前阶段或需要你的确认。"

    if (-not [string]::IsNullOrWhiteSpace($rawInput)) {
        $payload = $rawInput | ConvertFrom-Json
        $notificationType = [string]$payload.notification_type

        switch ($notificationType) {
            "permission_prompt" { $message = "Claude Code 正在等待命令或工具权限确认。" }
            "idle_prompt" { $message = "Claude Code 已完成当前工作，正在等待你的下一步指令。" }
            "agent_needs_input" { $message = "后台子智能体需要你的输入。" }
            "agent_completed" { $message = "后台子智能体已经完成或停止。" }
        }
    }

    $escapedMessage = $message.Replace("'", "''")
    $script = @"
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.MessageBox]::Show(
    '$escapedMessage',
    'Claude Code',
    [System.Windows.Forms.MessageBoxButtons]::OK,
    [System.Windows.Forms.MessageBoxIcon]::Information
) | Out-Null
"@

    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($script))

    Start-Process powershell.exe `
        -WindowStyle Hidden `
        -ArgumentList "-NoProfile -EncodedCommand $encoded" | Out-Null

    exit 0
}
catch {
    exit 0
}
