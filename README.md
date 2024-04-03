Checkmate Checklist Manager
===========================

Checkmate is an app for creating, managing and following checklists,
facilitating execution of repeatable processes.

## Local development setup

You will need a Rust toolchain installed, preferably through [rustup](https://rustup.rs/).

Then you need to install dependencies:

```
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

Next steps rely on having a postgres database instance. You can use docker compose to set one up:
```
docker compose -f docker/docker-compose.yml up -d
```
After starting the db container for the first time, run:

```
source .env # to set the DATABASE_URL
sqlx database reset # to run the migrations and let sqlx create tables it needs
```

You can then just run the backend with:

```
cargo run
```

In order to produce more human-readable logs in the terminal, you can use `jq`:

```
cargo run | jq -r '[.time, .msg] | @tsv'
```

### Git hooks

This repository has git hooks prepared that check simple conditions that might otherwise trip up the CI setup. We recommend that you use them. In order to set them up, run the following command inside the repository:

```
git config core.hooksPath .githooks
```
