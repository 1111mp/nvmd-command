# nvmd-command

English | [简体中文](./README.zh-CN.md)

`nvmd-command` (binary name: `nvmd`) is a lightweight Node.js version management CLI built with Rust.

It is also the command runtime used by [nvm-desktop](https://github.com/1111mp/nvm-desktop):

- Manage Node.js versions from terminal (`install`, `use`, `list`, `uninstall`, etc.)
- Route shim commands (`node`, `npm`, `npx`, `corepack`, ...) to the correct Node version
- Support both global default version and project-level version selection

---

## Features

- Single native executable (Rust)
- Multi-version Node.js install/switch workflow
- Project-level version file support (default: `.nvmdrc`)
- Query current version and install paths
- Shared ecosystem and data layout with nvm-desktop

---

## Build from source

> End users are usually recommended to install `nvmd` from nvm-desktop distribution.

### 1) Prerequisite

Install Rust toolchain (stable): <https://www.rust-lang.org/tools/install>

### 2) Build

```bash
# debug build
cargo build

# release build
cargo build --release
```

Output binaries:

- Linux / macOS: `target/release/nvmd`
- Windows: `target/release/nvmd.exe`

---

## Quick start

```bash
# install a Node.js version
nvmd install 20.11.1

# set global default version
nvmd use 20.11.1

# show active version
nvmd current

# list installed versions
nvmd ls
```

Project-level version (writes version file in current directory, default `.nvmdrc`):

```bash
nvmd use 18.20.3 --project
```

---

## Command reference

Use `nvmd --help` for full help.

| Command | Description |
|---|---|
| `nvmd current` | Show current active Node.js version |
| `nvmd install <version>` | Install a specific version |
| `nvmd list` / `nvmd ls` | List installed versions |
| `nvmd list --group` | List project groups |
| `nvmd uninstall <version>` | Uninstall a specific version |
| `nvmd use <version>` | Set global default version |
| `nvmd use <version> --project` | Set version for current project |
| `nvmd which <version>` | Show install path for a version (Unix: `.../bin`) |

> Version input supports both `v20.11.1` and `20.11.1`.

### Version resolution priority

When resolving which Node.js version to run, `nvmd` uses this order (highest to lowest):

1. `NVMD_NODE_VERSION` environment variable
2. Project version file (default: `.nvmdrc`, searched from current directory up through parent directories)
3. Global default file (`$NVMD_HOME/default`)

`NVMD_NODE_VERSION` has the highest priority and overrides project/global settings for the current process environment.

---

## How it works (shim-based)

`nvmd-command` uses shims instead of shell hooks:

1. You run `node`, `npm`, or other related commands.
2. A shim forwards the request to `nvmd`.
3. `nvmd` resolves version from project version file (default `.nvmdrc`) or global default (`$NVMD_HOME/default`).
4. `nvmd` spawns a child process with adjusted `PATH` pointing to the target Node directory.

This keeps version switching fast, reliable, and shell-agnostic.

---

## Directory layout and data files

Default root: `$HOME/.nvmd` (override with `NVMD_HOME`).

```text
$NVMD_HOME/
├─ bin/            # shims and executable entry
├─ versions/       # installed Node.js versions
├─ default         # global default Node version
├─ setting.json    # settings
├─ projects.json   # project-to-version mapping
├─ groups.json     # project group info
└─ packages.json   # global package shim metadata
```

---

## Configuration (`setting.json`)

`$NVMD_HOME/setting.json` supports:

```json
{
  "directory": "/custom/path/to/versions",
  "mirror": "https://nodejs.org/dist",
  "node_version_file": ".nvmdrc"
}
```

- `directory`: Node.js versions install directory (default: `$NVMD_HOME/versions`)
- `mirror`: Node.js download mirror (default: `https://nodejs.org/dist`)
- `node_version_file`: project version filename (default: `.nvmdrc`)

---

## FAQ

### `nvmd use <version>` says "not installed"

Install it first:

```bash
nvmd install <version>
```

### Project switching does not work

Check:

- Current directory (or parent directories) contains version file (default `.nvmdrc`)
- The version in file is installed
- Your shell `PATH` prioritizes shims in `$NVMD_HOME/bin`

### Change download mirror

Update `mirror` in `setting.json` and run `nvmd install <version>` again.

---

## Integration with nvm-desktop

`nvmd-command` works closely with [nvm-desktop](https://github.com/1111mp/nvm-desktop):

- nvm-desktop provides GUI workflows and ecosystem integration
- nvmd-command provides high-performance CLI/runtime dispatching

For packaging/integration details:
<https://github.com/1111mp/nvm-desktop#develop-and-build>
