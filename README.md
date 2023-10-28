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

After installing nvm-desktop and starting it for the first time, the following files will be added to the `$HOME/.nvmd/bin` directory (Taking the macOS platform as an example, it is roughly the same on the Windows platform):

<img width="793" alt="Screenshot 2023-10-28 at 16 22 30" src="https://github.com/1111mp/nvmd-command/assets/31227919/7d8a1de8-3c60-4e93-a875-657b47df9aeb">

When you run a `node` or `npm` command, it will eventually be executed by `nvmd`. `nvmd` will quickly identify the correct node engine version internally and then create a new process to execute the command you entered.

Examples (Assume that your node version is `v20.6.1` at this time):

> The version number of the set `node` will be saved in the `$HOME/.nvmd/default` file.

When you enter the `node --version` command in the terminal:

```shell
node --version
```

- `nvmd` will actually be called to execute.
- Inside nvmd, it will quickly find the installation directory of the corresponding version node through the set version number: `$HOME/.nvmd/versions/20.6.1`

```rust
fn get_version() -> String {
    // Find the version number set for the project
    let mut nvmdrc = match env::current_dir() {
        Err(_) => PathBuf::from(""),
        Ok(dir) => dir,
    };
    nvmdrc.push(".nvmdrc");

    let project_version = match read_to_string(&nvmdrc) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    if !project_version.is_empty() {
        return project_version;
    }

    // Find the version number set for the system
    let mut default_path = NVMD_PATH.clone();
    default_path.push("default");

    let default_version = match read_to_string(&default_path) {
        Err(_) => String::from(""),
        Ok(v) => v,
    };

    return default_version;
}
```

- Then nvmd will create a new process and add the directory where the executable file of this version of node is located to the environment variable of the new process.
- Add env `$HOME/.nvmd/versions/20.6.1/bin`
- Execute the `node --version` command by this new process

```rust
let child = Command::new(exe)
    .env("PATH", ENV_PATH.clone()) // $HOME/.nvmd/versions/20.6.1/bin:$PATH
    .args(args)
    .status();
```

- Then get the output of the new process, wait for it to execute and finally exit.

```rust
match child {
    Ok(status) => Ok(status),
    Err(_) => Err(String::from("failed to execute process")),
}
```

- Finally output:

```shell
node --version
v20.6.1
```

But the commands for `npm install packages --global` or `npm uninstall packages --global` require special handling：

- When using `npm` to install packages globally, `nvmd-command` will add a shim and record the corresponding `node` version. These information will be stored in `$HOME/.nvmd/packages.json` file with the content like this (example for `npm install @vue/cli typescript -g`):

![image](https://github.com/1111mp/nvmd-command/assets/31227919/b82f8ba5-dff1-4c36-a9a1-c1fbe130383e)

```json
// "18.17.1", "20.5.1" Installed in both versions
// When `npm install @vue/cli typescript -g` is executed successfully, the following information will be recorded
// will not be added repeatedly
{"tsc":["18.17.1", "20.5.1"],"tsserver":["18.17.1", "20.5.1"],"vue":["18.17.1", "20.5.1"]}

// Uninstall the "20.5.1" version
// npm uninstall @vue/cli typescript -g
// The shim will not be removed because it is still referenced by "18.17.1"
{"tsc":["18.17.1"],"tsserver":["18.17.1"],"vue":["18.17.1"]}

// Continue to uninstall the "18.17.1" version
// npm uninstall @vue/cli typescript -g
// The shim will be removed as it is not referenced by any version
{"tsc":[],"tsserver":[],"vue":[]}
```

This ensures the independence of each version of the Node engine, and they will not affect each other.

## Build nvmd-command

- First, you should have a Rust runtime installed locally. Please read the official guide: [rust get-started](https://www.rust-lang.org/learn/get-started).
- Then pull the project code locally, go to the `./` folder.
- Run `cargo build` (debug) or `cargo build --release` (release) build your executable.
- Finally, you can find the compiled executable named `nvmd` in the `./target/release/` directory. (`nvmd.exe` on Windows.)

Check out this [documentation](https://github.com/1111mp/nvm-desktop#develop-and-build) on how to package this executable with [nvm-desktop](https://github.com/1111mp/nvm-desktop).
