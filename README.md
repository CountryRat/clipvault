# ClipVault

> 本地优先的剪贴板历史管理器。轻量、离线、隐私安全。

## 功能

- **自动记录** — 复制文本自动存入本地 SQLite，blake3 哈希去重
- **快速搜索** — `Ctrl+Shift+V` 呼出面板（可自定义），输入即过滤
- **收藏置顶** — 常用内容固定到列表顶部
- **键盘操作** — ↑↓ 导航 / Enter 粘贴 / Esc 隐藏
- **系统托盘** — 左键切换面板，右键菜单（显示/隐藏/退出）
- **可拖拽** — 按住搜索栏区域拖动窗口
- **外观定制** — 背景色 + 不透明度可调
- **首次引导** — 启动时询问是否开机自启
- **窗口记忆** — 记住上次窗口位置
- **单实例** — 防止重复启动
- **零联网** — 所有数据存在本机，不发送任何信息

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2.x |
| 后端 | Rust (arboard, rusqlite, blake3, tokio, chrono) |
| 前端 | React 19 + TypeScript + Tailwind CSS v4 |
| 快捷键 | 全局热键，动态注册，前端录制 |
| 存储 | SQLite + JSON 配置文件 |
| 打包 | MSI / NSIS 安装包 |

## 安装

```bash
git clone https://github.com/CountryRat/clipvault.git
```

双击仓库根目录的 `ClipVault.exe`，无需安装任何依赖。Windows 10/11 自带所需组件。

## 使用说明

首次启动会询问是否开机自启，之后进入主界面，程序常驻系统托盘。

| 操作 | 方式 |
|---|---|
| 呼出搜索面板 | `Ctrl+Shift+V`（可在设置中修改） |
| 搜索历史 | 输入文字即过滤 |
| 粘贴到当前窗口 | 点击条目或按 Enter |
| 收藏/取消收藏 | 悬停条目 → 点书签图标 |
| 删除 | 悬停条目 → 点垃圾桶图标 |
| 切换面板 | 左键托盘图标 |
| 退出程序 | 右键托盘图标 → 退出 |
| 移动窗口 | 按住搜索栏区域拖动 |
| 修改外观 | 齿轮图标 → 背景色/不透明度 |
| 开机自启 | 齿轮图标 → 开关 |

## 项目结构

```
clipboard-manager/
├── ClipVault.exe             # 发布产物，12MB，可独立分发
├── src/
│   ├── hooks/
│   │   ├── useClips.ts       # 剪贴板数据 & 轮询
│   │   └── useKeyboard.ts    # 键盘监听
│   ├── components/
│   │   ├── SearchBar.tsx     # 搜索栏 + 拖拽
│   │   ├── ClipList.tsx      # 剪贴板列表
│   │   ├── SettingsPanel.tsx # 设置（快捷键/外观/自启）
│   │   └── StatusFooter.tsx  # 底部状态栏
│   ├── App.tsx               # 主入口
│   ├── main.tsx
│   └── index.css
└── src-tauri/src/
    ├── main.rs               # 程序入口
    ├── lib.rs                # 插件、托盘、窗口、单实例
    ├── clipboard.rs          # 剪贴板读取 & 哈希去重
    ├── commands.rs           # 全部 Tauri IPC 命令
    ├── config.rs             # 配置读写 & 快捷键解析
    └── db.rs                 # SQLite 存储层（含单元测试）
```

## 许可

MIT
