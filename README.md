# nvmd-command

`nvmd-comand` is a single, fast native executable, with no external dependencies, build with Rust. A proxy for Node and Npm, through which it can intelligently (quickly) identify the correct version of the Node engine.

Provides services for [nvm-desktop](https://github.com/1111mp/nvm-desktop)'s Node engine version management function.

You can also manage all versions of node directly from the command line. But if you need to download and install a new version of node, you should open the `nvm-desktop` application.

## Command tools intro

`nvmd` allows you to quickly manage different versions of node via the command line.

```shell
$ nvmd use 18.17.1
Now using node v18.17.1
$ node -v
v18.17.1
$ nvmd use v20.5.1 --project
Now using node v20.5.1
$ node -v
v20.5.1
$ nvmd ls
v20.6.1
v20.5.1 (currently)
v18.17.1
$ nvmd current
v20.5.1
```

Simple as that!

## Usage

Please download and install the latest release of node in the `nvm-desktop` application.

```shell
$ nvmd --help
nvmd (2.2.0)
The1111mp@outlook.com
command tools for nvm-desktop

Usage: nvmd [COMMAND]

Commands:
  current  Get the currently used version
  list     List the all installed versions of Node.js
  ls       List the all installed versions of Node.js
  use      Use the installed version of Node.js (default is global)
  which    Get the path to the executable to where Node.js was installed
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version

Please download new version of Node.js in nvm-desktop.
```

You can list all installed versions using `list` or `ls`:

```shell
nvmd list
# or
nvmd ls
```

And then in any new shell just use the installed version:

```shell
nvmd use node_version
```

Or you can run it directly using the node version for your project:

```shell
nvmd use node_version --project
```

You can get the version you are currently using:

```shell
nvmd current
```

You can also get the path to the executable to where it was installed:

```shell
nvmd which 18.17.1
```

## How does it work?

`nvmd-comand` does not use any fancy OS features or shell-specific hooks. It’s built on the simple, proven approach of shims.

When you run a Node or Npm command, `nvmd-comand` will quickly identify the correct Node engine version and run it.

```
Run Node or Npm script commands

    V   V   V

Identify the correct Node engine version, then start a new process running the above command.
The installation path of Node will be injected into the environment variable of the new process as PATH.

    V   V   V

Get the output of the new process, wait for it to execute and finally exit.
```

But the commands for Npm global installation packages require special handling：

When using npm to install packages globally, `nvmd-command` will add a shim and record the corresponding Node version. This information will be stored in a file with the content like this (example for `npm install @vue/cli typescript -g`):

```json
{ "tsc": ["20.5.1"], "tsserver": ["20.5.1"], "vue": ["20.5.1"] }
```

When uninstalling global packages, this information will be used to determine whether these shims need to be removed. After uninstalling, the content of the file will become like this (example for `npm uninstall @vue/cli typescript -g`):

```json
{ "tsc": [], "tsserver": [], "vue": [] }
```

This ensures the independence of each version of the Node engine, and they will not affect each other.

## Build nvmd-command

- First, you should have a Rust runtime installed locally. Please read the official guide: [rust get-started](https://www.rust-lang.org/learn/get-started).
- Then pull the project code locally, go to the `./` folder.
- Run `cargo build` (debug) or `cargo build --release` (release) build your executable.
- Finally, you can find the compiled executable named `nvmd` in the `./target/release/` directory. (`nvmd.exe` on Windows.)

Check out this [documentation](https://github.com/1111mp/nvm-desktop#develop-and-build) on how to package this executable with [nvm-desktop](https://github.com/1111mp/nvm-desktop).
