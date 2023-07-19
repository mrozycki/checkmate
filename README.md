Checkmate Checklist Manager
===========================

Checkmate is an app for creating, managing and following checklists,
facilitating execution of repeatable processes.

## Local development setup

You will need a Rust toolchain installed, preferably through [rustup](https://rustup.rs/).

You can then just run the backend with:

```
cargo run
```

### Git hooks

This repository has git hooks prepared that check simple conditions that might otherwise trip up the CI setup. We recommend that you use them. In order to set them up, run the following command inside the repository:

```
git config core.hooksPath .githooks
```
