# atcoder_cli

English (this file) | [日本語](./README.md)

A CLI helper for AtCoder: download problem statements, run sample test cases, and manage your login session from the command line.

## Installation

Requires Rust 1.85+ (edition 2024).

```sh
cargo build --release
# Binary: target/release/acc
```

## Subcommands

| Command | Alias | Description |
|---|---|---|
| `acc test <TASK_ID>` | `t` | Run sample cases and report AC/WA/TLE/RE verdicts |
| `acc auth login` | `a` | Save the `REVEL_SESSION` cookie copied from your browser |
| `acc auth logout` | | Delete the cookie file |
| `acc auth status` | | Check whether the saved cookie is still valid |
| `acc init` | | Create an `atcoder.toml` config file interactively |
| `acc download <TASK_ID>` | `dl` | Fetch a problem statement as Markdown to stdout (`-o` to save to a file) |

`<TASK_ID>` accepts a task ID such as `abc350_a`, or the full problem page URL.

## Typical workflow

```sh
# 1. One-time setup: config + login
acc init
acc auth login   # follow the instructions and paste REVEL_SESSION

# 2. Download the problem statement
acc dl abc350_a > problem.md
# or save directly with -o
acc dl abc350_a -o problem.md

# 3. Write your solution and run the samples
acc t abc350_a
```

## Config file (atcoder.toml)

The CLI loads `./atcoder.toml` from the working directory first, falling back to the global config (`~/.config/atcoder-cli/atcoder.toml`).

```toml
compiler = "g++"                       # optional (omit for interpreted languages)
compiler_flags = ["${TASK_ID}.cpp", "-O2", "-o", "${TASK_ID}"]
run = "./${TASK_ID}"
run_flags = []                         # optional
```

The following template variables are available in commands and flags:

| Variable | Value | Example (`abc350_a`) |
|---|---|---|
| `${CONTEST_ID}` | Contest ID | `abc350` |
| `${TASK_ID}` | Task ID | `abc350_a` |
| `${TASK_CHAR}` | Problem letter | `a` |

## Test behavior

- Verdicts are **AC / WA / TLE / RE**. Differences in line endings or leading/trailing whitespace are ignored.
- The time limit is fetched from the problem page (falls back to 2 seconds).
- With [GNU time](https://www.gnu.org/software/time/) (`/usr/bin/time` or `gtime`) installed, memory usage is reported as well.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (including all samples passing) |
| `1` | Error (network failure, compilation impossible, broken config, ...) |
| `2` | Command completed but the outcome was negative (WA/TLE, invalid cookie, ...) |

Scripts can rely on these exit codes.

## Login session

The cookie saved by `acc auth login` is stored in plaintext at `~/.config/atcoder-cli/cookies.txt` with owner-only permissions (0600). Handle it with care.

## License

MIT LICENSE
