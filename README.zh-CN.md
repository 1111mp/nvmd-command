# nvmd-command

简体中文 | [English](./README.md)

`nvmd-command`（可执行文件名：`nvmd`）是一个使用 Rust 构建的轻量级 Node.js 版本管理命令行工具。它同时也是 [nvm-desktop](https://github.com/1111mp/nvm-desktop) 的核心命令组件：

- 通过 `nvmd` 在命令行切换、安装、卸载 Node.js 版本。
- 通过 `node` / `npm` / `npx` / `corepack` 等 shim 自动路由到正确版本。
- 支持全局默认版本 + 项目级版本（基于项目目录中的版本文件）。

---

## 功能特性

- ✅ 单一原生可执行文件（Rust）。
- ✅ 支持多版本 Node.js 安装与切换。
- ✅ 支持项目级版本锁定（默认写入 `.nvmdrc`）。
- ✅ 支持查询当前版本与安装路径。
- ✅ 与 nvm-desktop 生态一致（配置与目录结构共享）。

---

## 安装与构建

> 如果你是普通用户，建议直接使用 nvm-desktop 分发的 `nvmd`，无需手动构建。

### 1) 环境准备

- 安装 Rust 工具链（建议 stable）：<https://www.rust-lang.org/tools/install>

### 2) 本地构建

```bash
# 调试构建
cargo build

# 发行构建
cargo build --release
```

构建成功后可执行文件位于：

- Linux / macOS: `target/release/nvmd`
- Windows: `target/release/nvmd.exe`

---

## 快速开始

```bash
# 安装一个 Node.js 版本
nvmd install 20.11.1

# 设置全局默认版本
nvmd use 20.11.1

# 查看当前使用版本
nvmd current

# 查看已安装版本
nvmd ls
```

项目级切换（会在当前目录写入版本文件，默认 `.nvmdrc`）：

```bash
nvmd use 18.20.3 --project
```

---

## 命令总览

你可以通过 `nvmd --help` 获取完整帮助，核心命令如下：

| 命令 | 说明 |
|---|---|
| `nvmd current` | 显示当前生效的 Node.js 版本 |
| `nvmd install <version>` | 安装指定版本 |
| `nvmd list` / `nvmd ls` | 列出已安装版本 |
| `nvmd list --group` | 列出项目分组信息 |
| `nvmd uninstall <version>` | 卸载指定版本 |
| `nvmd use <version>` | 设置全局默认版本 |
| `nvmd use <version> --project` | 为当前项目设置版本 |
| `nvmd which <version>` | 显示指定版本 Node 安装目录（Unix 为 `.../bin`） |

> 版本号支持 `v20.11.1` 或 `20.11.1` 两种输入形式。

### 版本解析优先级

`nvmd` 在决定当前使用哪个 Node.js 版本时，按以下优先级（从高到低）解析：

1. `NVMD_NODE_VERSION` 环境变量
2. 项目版本文件（默认 `.nvmdrc`，会从当前目录向上查找父目录）
3. 全局默认文件（`$NVMD_HOME/default`）

其中 `NVMD_NODE_VERSION` 优先级最高，会覆盖项目与全局配置（仅对当前进程环境生效）。

---

## 工作机制（Shim）

`nvmd-command` 采用 shim 机制而非 shell hook：

1. 你在终端执行 `node` / `npm` 等命令。
2. shim 程序把请求交给 `nvmd`。
3. `nvmd` 根据当前目录的项目版本文件（默认 `.nvmdrc`）或全局默认版本（`$NVMD_HOME/default`）解析目标版本。
4. `nvmd` 调整子进程 `PATH`，指向目标版本目录后再执行真实命令。

这样可以在无侵入 shell 配置的情况下，实现稳定、快速的版本切换体验。

---

## 目录结构与数据文件

默认根目录：`$HOME/.nvmd`（可通过环境变量 `NVMD_HOME` 覆盖）。

常见结构如下：

```text
$NVMD_HOME/
├─ bin/            # shim 与可执行入口
├─ versions/       # Node.js 版本安装目录
├─ default         # 全局默认 Node 版本
├─ setting.json    # 配置文件
├─ projects.json   # 项目与版本映射
├─ groups.json     # 项目分组信息
└─ packages.json   # 全局包 shim 元数据
```

---

## 配置说明（setting.json）

`$NVMD_HOME/setting.json` 支持以下字段：

```json
{
  "directory": "/custom/path/to/versions",
  "mirror": "https://nodejs.org/dist",
  "node_version_file": ".nvmdrc"
}
```

- `directory`: Node.js 版本安装目录（默认 `$NVMD_HOME/versions`）
- `mirror`: Node.js 下载镜像地址（默认 `https://nodejs.org/dist`）
- `node_version_file`: 项目版本文件名（默认 `.nvmdrc`）

---

## 常见问题

### 1) `nvmd use <version>` 提示未安装

请先执行：

```bash
nvmd install <version>
```

### 2) 项目切换不生效

请检查：

- 当前目录（或父目录）是否存在版本文件（默认 `.nvmdrc`）。
- 文件中的版本号是否已安装。
- 终端 `PATH` 中是否优先使用了 `$NVMD_HOME/bin` 下的 shim。

### 3) 想更换镜像源

在 `setting.json` 中调整 `mirror` 字段后重新执行 `nvmd install <version>`。

---

## 与 nvm-desktop 集成

`nvmd-command` 与 [nvm-desktop](https://github.com/1111mp/nvm-desktop) 协同工作：

- nvm-desktop 负责 GUI 体验、下载与管理流程集成。
- nvmd-command 提供高性能 CLI 与运行时路由能力。

如需了解完整打包/集成流程，可参考：
<https://github.com/1111mp/nvm-desktop#develop-and-build>
