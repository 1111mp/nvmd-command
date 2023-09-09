# nvmd-command

`nvmd-comand` is a single, fast native executable, with no external dependencies, build with Rust. A proxy for Node and Npm, through which it can intelligently (quickly) identify the correct version of the Node engine.

Provides services for [nvm-desktop](https://github.com/1111mp/nvm-desktop)'s Node engine version management function.

### How does it work?

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

### Build nvmd-command

- First, you should have a Rust runtime installed locally. Please read the official guide: [rust get-started](https://www.rust-lang.org/learn/get-started).
- Then pull the project code locally, go to the `./` folder.
- Run `cargo build` (debug) or `cargo build --release` (release) build your executable.
- Finally, you can find the compiled executable named `nvmd` in the `./target/release/` directory. (`nvmd.exe` on Windows.)

Check out this [documentation](https://github.com/1111mp/nvm-desktop#develop-and-build) on how to package this executable with [nvm-desktop](https://github.com/1111mp/nvm-desktop).
