<h1 align="center">krabby</h1>
<div align="center">
  <strong>
    A tiny project manager! :crab:
  </strong>
</div>

<br />

<div align="center">
<!-- Build Status -->
  <a href="https://github.com/jpedrodelacerda/krabby/actions/workflows/code-health.yml"><img alt="Build Status" src="https://img.shields.io/github/actions/workflow/status/jpedrodelacerda/krabby/code-health"></a>
<!-- License -->
  <img src="https://img.shields.io/github/license/jpedrodelacerda/krabby" alt="License: MIT/Apache-2.0">
</div>

<br />

## What is `krabby`

> Or should I say what it aims to be?

Well, `krabby` is my take on what project management should be.
I was never a huge fan of `make` and the only thing I like about npm/yarn is their way of handling script stuff.
I also am a spoiled programmer and want to access my projects easily without setting every alias on my shell.
I missed things in the Rust ecosystem so I thought "Why not handle this myself?". So here it goes!

## Installation

You can build the project locally or get the binary at the releases page.

After placing the binary somewhere in your `PATH`, you should add the following to your `.bashrc` or `.zshrc` file:

```bash
eval "$(krabby shell bash)"
```

> Currently it only supports `bash` and `zsh`.

## Features

### Project database

Register all your projects and teleport yourself to them whenever you feel like!

Check `krabby.example.db` to see what it's like.

> Some metadata might be useful here, but I have no idea right now.

### Project scripts

Language-agnostic project scripts: create, delete and remove scripts to project file.

> You cannot compose scripts _yet_.

You can see an example at `krabby.example.toml`.

### Project hook

Create a hook so it runs automatically after entering the project with `kb` command!
A hook can be a plain command or a sequence of scripts.

```toml
name = "krabby"
hook = ["hello", "world"]

[scripts]
hello = "echo hello"
world = "echo world"
```

and

```toml
name = "krabby"
hook = "echo hello; echo world"
```

are equivalent!

> The project hook must be defined **before** the script session.
> See [this issue](https://github.com/toml-rs/toml-rs/issues/142) for more info!

## Examples

- [x] Manage project: manage project entries in your database (`~/.krabby.db`).
  ```bash
  kb project add PROJECT PATH
  kb project remove PROJECT
  ```
- [x] Jump to project: go straight to your project directory.
  ```bash
  kb PROJECT
  ```
- [x] Run scripts: execute your scripts
  ```bash
  kb run SCRIPT
  ```
- [x] Define hooks: set scripts to run after loading project
  ```bash
  # Run `setup` script after you check in the project with `kb PROJECT`.
  kb hook set SCRIPT1,SCRIPT2,SCRIPT3
  # or
  kb hook set COMMAND
  ```

### Roadmap

- [ ] Script composition: I think it would be nice to use scripts inside other scripts, but could not figure out a way to make this work yet.
- [ ] Improve argument parsing: I know that `clap` can parse the value directly, but this means I'd have to rewrite the `parse` functions so it returns a `Result` instead of `Self`.
> This was based on @LukeMathWalker's `zero2prod` chapter on Parsing vs Validation. I'm still getting the hang of it (I hope).
> But if you do not know this, I highly recommend it!
> [Zero To Production In Rust](https://www.zero2prod.com/index.html)
- [ ] Improve feedback: message system seems to have a lot of room for improvement. Also, I'd like to make sure the user knows what's happening.
- [ ] Write better tests: I think the code looks like a huge mess and I'd like to tidy up the place!
- [ ] Provide completion: it would be nice to have it complete the script names when using `run` command.
- [ ] Improve logs: use `log` crate to improve verbose output
- [ ] Create new projects and register directly at `krabby.db`
