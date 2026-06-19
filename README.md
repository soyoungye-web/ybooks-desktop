# Ybooks Desktop

Ybooks 工作看板 Windows 桌面应用，基于 Tauri 2.0。

## 功能

- 🔔 系统托盘常驻（关闭按钮最小化到托盘，右键菜单）
- ⚡ 全局快捷键 `Ctrl+Shift+Y` 唤出/隐藏窗口
- 🚀 开机自启（可选）
- 📌 单实例锁（防止重复打开）
- 💾 窗口位置/大小记忆
- 🔔 原生系统通知

## 架构

```
Tauri 2.0 (Rust) 壳 → 加载远程页面 http://81.71.6.207:86/
```

桌面壳提供原生能力（托盘、快捷键、自启），业务逻辑全部在远程 Web 服务上。

## 本地构建（Windows）

```powershell
# 安装 Rust: https://rustup.rs
# 安装 Node.js 20+

cd desktop
npm install
npm run tauri build
# 安装包输出: src-tauri/target/release/bundle/nsis/
```

## CI 构建（GitHub Actions）

```bash
git tag v1.0.0
git push origin v1.0.0
# 自动触发 Windows 构建，生成 .exe 安装包并创建 Release
```
