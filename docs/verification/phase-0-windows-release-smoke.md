# Phase 0 Windows Release 冒烟验证报告

## 验证信息

- 验证日期：2026-07-16
- 基础 Git commit SHA：031bced9864cce7e57a835f286c44180e9719e07
- 构建时工作区状态：非干净
- 构建时未提交文件：apps/desktop/src-tauri/tauri.conf.json
- tauri.conf.json SHA256：4645B258CF249908629540D7413131EECC87DFAF537266421092F01BE6BB79B5

## 环境信息

- Windows 版本：10.0.26200 (Windows 11)
- 系统架构：AMD64 (x64)
- PowerShell 版本：7.6.3
- Node 版本：v22.22.2
- pnpm 版本：10.33.2
- Rust 版本：1.96.0
- Rust host：x86_64-pc-windows-msvc
- Tauri CLI 版本：tauri-cli 2.11.4
- WebView2 Runtime 状态：PASS
- WebView2 Runtime 版本：150.0.4078.65
- WebView2 检测范围：HKLM 32-bit

## 构建信息

- 构建命令：pnpm --filter @devforge/desktop tauri build --ci --no-sign
- 构建结果：PASS
- 安装包路径：C:\Users\<USER>\Desktop\Work\DevForge\target\release\bundle\nsis\DevForge_0.1.0_x64-setup.exe
- 安装包文件名：DevForge_0.1.0_x64-setup.exe
- 安装包大小：3,701,775 bytes (约 3.5 MB)
- 安装包时间：2026-07-16 17:37:51
- 安装包 SHA256：7B808167960718EF53B60E57D5D4132BE60EFB5E0E9165CB57A67E68A8A7DB44
- Authenticode 状态：NotSigned（未签名，符合预期）
- 安装包产品名称：DevForge
- 安装包产品版本：0.1.0

## 配置信息

- 安装模式：perMachine
- WebView2 策略：downloadBootstrapper
- 安装包基于 commit 031bced9864cce7e57a835f286c44180e9719e07，并包含报告中记录的未提交 tauri.conf.json 变更

## 数据保护

- 原始数据目录：C:\Users\<USER>\AppData\Local\DevForge
- 备份目录：C:\Users\<USER>\AppData\Local\DevForge.backup-20260716-173028
- 重命名目录：C:\Users\<USER>\AppData\Local\DevForge.pre-smoke-20260716-173028
- 原始目录状态：已重命名（不存在）
- 备份状态：PASS
- 卸载后测试数据库：PASS，devforge.db 保留
- 测试数据归档：C:\Users\<USER>\AppData\Local\DevForge.smoke-20260716-173028
- 原数据恢复：PASS
- 原数据目录：C:\Users\<USER>\AppData\Local\DevForge
- 安全备份保留：C:\Users\<USER>\AppData\Local\DevForge.backup-20260716-173028

## 质量检查

- pnpm check：PASS
- 绑定检查：PASS
- Rust 源码检查：PASS
- Cargo 文件检查：PASS
- CI 和依赖检查：PASS
- Git 空白检查：PASS

## 自动验证结果

| 检查项 | 状态 |
|--------|------|
| 环境版本 | PASS |
| pnpm check | PASS |
| tauri.conf JSON 有效性 | PASS |
| NSIS 构建 | PASS |
| 安装包存在 | PASS |
| 安装包大小 | PASS |
| SHA256 可计算 | PASS |
| 产品元数据 | PASS |
| 配置差异检查 | PASS |

## 卸载自动复核

- 卸载注册表状态：PASS（无 DevForge 安装项）
- Program Files 残留状态：PASS（无 DevForge 目录）
- DevForge 进程状态：PASS（无运行进程）

## 人工验证检查点

人工验证证据来源：
用户于 2026-07-16 明确确认安装、首次启动、关闭后第二次启动以及卸载验证均成功。

### 安装检查 (A)

| 检查项 | 状态 |
|--------|------|
| A1. 安装器无不可恢复错误 | PASS |
| A2. 已安装的应用中存在 DevForge | PASS |
| A3. 版本为 0.1.0 | PASS |
| A4. 安装路径位于 Program Files | PASS |
| A5. 安装路径不等于 %LOCALAPPDATA%\DevForge | PASS |
| A6. 存在主程序 | PASS |
| A7. 存在卸载程序 | PASS |
| A8. 存在开始菜单快捷方式 | PASS |

### 首次启动检查 (B)

| 检查项 | 状态 |
|--------|------|
| B1. 窗口能够打开 | PASS |
| B2. 窗口标题为 DevForge | PASS |
| B3. 页面显示 DevForge | PASS |
| B4. 版本显示 0.1.0 | PASS |
| B5. 数据目录显示 %LOCALAPPDATA%\DevForge | PASS |
| B6. 数据库状态显示"就绪（migration v1）" | PASS |
| B7. 不白屏 | PASS |
| B8. 不立即退出 | PASS |
| B9. 没有可见 React、Router、Query 或 IPC 错误 | PASS |

### 第二次启动检查 (C)

| 检查项 | 状态 |
|--------|------|
| C1. 第一次可以正常退出 | PASS |
| C2. 第二次可以正常启动 | PASS |
| C3. 版本仍为 0.1.0 | PASS |
| C4. migration 仍为 v1 | PASS |
| C5. 数据库没有重新初始化错误 | PASS |

### 卸载检查 (D)

| 检查项 | 状态 |
|--------|------|
| D1. 卸载器可以启动 | PASS |
| D2. 卸载完成 | PASS |
| D3. Program Files 安装目录被移除 | PASS |
| D4. 开始菜单快捷方式被移除 | PASS |
| D5. 已安装的应用不再显示 DevForge | PASS |
| D6. 没有 DevForge 进程残留 | PASS |
| D7. %LOCALAPPDATA%\DevForge\devforge.db 仍然存在 | PASS |

## 失败项

无

## 未验证项

无 Task 10 阻塞项。
代码签名、MSI、离线 WebView2、自动更新不属于本任务范围。

## 最终结论

PASS — Windows NSIS Release 冒烟验证完成

## 附录：tauri.conf.json 变更

```diff
--- a/apps/desktop/src-tauri/tauri.conf.json
+++ b/apps/desktop/src-tauri/tauri.conf.json
@@ -20,5 +20,18 @@
       }
     ],
     "security": {}
+  },
+  "bundle": {
+    "active": true,
+    "targets": ["nsis"],
+    "icon": ["icons/icon.ico"],
+    "windows": {
+      "webviewInstallMode": {
+        "type": "downloadBootstrapper"
+      },
+      "nsis": {
+        "installMode": "perMachine"
+      }
+    }
   }
 }
```
