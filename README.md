# dev-scaffold

`dev-scaffold` is a CLI that creates a new service repo from a standard template: FastAPI or Axum API, or a Python worker, with Docker, CI, and secure defaults already in place.

This is not a general template engine, and it is not a security product. It is an opinionated bootstrap so new services start from the same reviewed baseline.

![dev-scaffold demo](docs/dev-scaffold.gif)

## Install

```bash
git clone https://github.com/EGJJR/dev-scaffold.git
cd dev-scaffold
cargo install --path .
```

The global command is `dev-scaffold`. Color and animation run only in an interactive terminal. Set `NO_COLOR=1` to disable color.

## Usage

```bash
dev-scaffold list
dev-scaffold payment-service --type api --dry-run
dev-scaffold payment-service --type api
```

Run it from the directory where you keep services, or pass `--output`.

### Options

- `name` (required): kebab-case service name, used as the directory name and in generated files
- `--type` / `-t`: `api` (default prompt), `api-rust`, or `worker`. If omitted and stdin is a terminal, the CLI prompts
- `--output` / `-o`: destination directory (default: `./<name>`)
- `--dry-run`: print the file tree and write nothing
- `--no-git`: skip `git init`

Names that contain path separators or `..` are rejected. An existing destination is refused.

## Templates

| Type | Stack | What you get |
| --- | --- | --- |
| `api` | FastAPI | HTTP service, Pydantic validation, production-safe docs/CORS/hosts, uv |
| `api-rust` | Axum | HTTP service, body limit, timeouts, CORS allow-list |
| `worker` | Python | Background loop, fail-closed config, SIGTERM shutdown, uv |

Generated layout (FastAPI):

```text
payment-service/
├── .github/workflows/ci.yml
├── .github/dependabot.yml
├── app/main.py
├── app/config.py
├── tests/
├── Dockerfile
├── justfile
├── pyproject.toml
├── .env.example
├── .gitignore
└── .dockerignore
```

After a successful run (FastAPI example):

```bash
cd payment-service
uv sync --extra dev
SECRET_KEY=dev-only-not-for-production ENV=test uv run pytest
SECRET_KEY=dev-only-not-for-production ENV=development uv run uvicorn app.main:app --reload --host 127.0.0.1 --port 8000
```

Rust services use `cargo test` / `cargo run` with `SECRET_KEY` set. Python services use [uv](https://docs.astral.sh/uv/). A `justfile` is included as an optional shortcut.

## Secure defaults

These controls are baked into every template. They prevent the usual day-zero holes; they do not make application logic secure by themselves.

- Non-root, multi-stage Docker image with a pinned base tag (not `:latest`)
- `.gitignore` and `.dockerignore` that keep `.env`, keys, and `.git` out of git and the image
- Required secrets via environment variables; the process exits if `SECRET_KEY` is missing
- Placeholder-only `.env.example` (`change-me`), never a real key
- GitHub Actions with `permissions: contents: read`, lint, tests, dependency audit, and `docker build`
- Dependabot for the language ecosystem, Actions, and Docker
- HTTP templates: no wildcard CORS in production; FastAPI disables OpenAPI docs when `ENV=production`; `/health` does not dump configuration

## What this is not

- Not authentication, authorization, or a secret manager
- Not Kubernetes policies, image signing, or a full SAST suite
- Not a replacement for Cookiecutter/Copier if you need arbitrary template packs

## Development

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

Distributed under the MIT License. See `LICENSE`.
