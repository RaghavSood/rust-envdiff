# dotenv-diff

Compare `.env` against `.env.example` and report missing or extra keys.

Use it to catch a forgotten variable before your app crashes at startup, or
in CI to make sure `.env.example` keeps up with the code. It's pure Rust and
has no dependencies.

## Install

```sh
cargo install --path .
```

## Usage

```text
dotenv-diff [OPTIONS] [ENV] [EXAMPLE]
```

| Argument / option  | Description                                                 |
| ------------------ | ----------------------------------------------------------- |
| `ENV`              | Env file to check (default `.env`)                          |
| `EXAMPLE`          | Template listing the expected keys (default `.env.example`) |
| `--ignore-missing` | Don't report or fail on keys missing from `ENV`             |
| `--ignore-extra`   | Don't report or fail on keys not in `EXAMPLE`               |
| `-q`, `--quiet`    | Print nothing; only set the exit code                       |
| `-h`, `--help`     | Print help                                                  |
| `-V`, `--version`  | Print version                                               |

### Exit status

| Code | Meaning                                                         |
| ---- | --------------------------------------------------------------- |
| `0`  | The files define the same keys                                  |
| `1`  | Keys are missing or extra                                       |
| `2`  | A file couldn't be read or parsed, or the arguments are invalid |

## Example

The files below are in [`examples/basic`](examples/basic).

<!-- example-file: .env.example -->
```sh
# Database
DATABASE_URL=postgres://localhost/app
REDIS_URL=redis://localhost:6379

# Auth
JWT_SECRET=
SESSION_TTL=3600

export LOG_LEVEL=info
```

<!-- example-file: .env -->
```sh
DATABASE_URL=postgres://prod-db/app
JWT_SECRET="s3cr3t"
SESSION_TTL=86400
LOG_LEVEL=debug
DEBUG=true
LEGACY_API_KEY=abc123
```

Running `dotenv-diff` in that directory:

<!-- example-run -->
```console
$ dotenv-diff
Missing from .env (1):
  - REDIS_URL  (.env.example:3)

Extra in .env (2):
  + DEBUG           (.env:5)
  + LEGACY_API_KEY  (.env:6)
```

Each key comes with the file and line where it's defined, so you can jump
straight to it. The command exits with status `1`.

If local-only variables in `.env` are fine and you only care about missing
ones:

<!-- example-run -->
```console
$ dotenv-diff --ignore-extra
Missing from .env (1):
  - REDIS_URL  (.env.example:3)
```

In CI, `--quiet` gives you just the exit code:

```sh
dotenv-diff --quiet .env.ci .env.example || exit 1
```

## Supported syntax

dotenv-diff only reads keys and ignores values. It understands:

- `KEY=value`, with optional spaces around `=`
- `export KEY=value`
- Blank lines and `#` comments
- Single, double and backtick quoted values, including quoted values that
  span multiple lines (e.g. PEM keys)
- Keys made of letters, digits, `_`, `.` and `-` that don't start with a digit

Keys are case-sensitive. If a key is defined more than once, it's reported
once, at its first line. A line that isn't a valid `KEY=value` makes the
command exit with status `2` and print the line number.

## Development

```sh
cargo test
```

The test suite includes `tests/readme.rs`, which checks that the example
files above match `examples/basic` and that each example command prints
exactly what's shown.
