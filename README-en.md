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
| `acc setup <CONTEST_ID>` | `s` | Create a contest directory with one subdirectory per problem |
| `acc download <TASK_ID>` | `dl` | Fetch a problem statement as Markdown to stdout (`-o` to save to a file) |

`<TASK_ID>` accepts a task ID such as `abc350_a`, or the full problem page URL.
`<CONTEST_ID>` accepts a contest ID such as `abc350`, or the full contest URL.

## Typical workflow

```sh
# 1. One-time setup: config + login
acc init
acc auth login   # follow the instructions and paste REVEL_SESSION

# 2. Create the contest directory (a/ b/ ... generated per problem)
acc setup abc350

# 3. Download the problem statement
acc dl abc350_a > problem.md
# or save directly with -o
acc dl abc350_a -o problem.md

# 4. Write your solution and run the samples (from the directory holding atcoder.toml)
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

## Contest directory setup (acc setup)

`acc setup <CONTEST_ID>` fetches the task list and generates a source template for every problem. It does **not** copy `atcoder.toml`: the intended style is to stay in the directory holding your config (the working root) and reference generated paths through template variables:

```toml
compiler = "g++"
compiler_flags = ["${CONTEST_ID}/${TASK_CHAR}/main.cpp", "-O2", "-o", "${TASK_ID}"]
run = "./${TASK_ID}"

[setup]
parent = "~/atcoder/contests"           # parent of the contest directory (default: current dir)
dir = "${TASK_CHAR}"
file = ["main.cpp"]
```

```sh
acc setup abc350          # created under [setup].parent, or the current directory
acc setup abc350 -d ~/cp  # override the output location for one run (-d wins over [setup].parent)
acc setup abc473 --md     # also download problem statements as Markdown
```

- Before a contest starts, the task-list page does not exist yet; the problem letters are then taken from the point-value table on the contest top page (`--md` cannot fetch anything in that state).
- **Existing files are never overwritten.** Re-running only creates what is missing.

The directory layout can be customized with a `[setup]` table:

```toml
[setup]
dir = "${TASK_CHAR}"                    # per-problem path (relative to the contest root)
file = ["main.cpp"]                     # files to create (relative to the problem dir, multiple allowed)

[setup.template]                        # inline templates keyed by extension (optional)
cpp = '''
#include <bits/stdc++.h>
using namespace std;

int main() {

}
'''
```

`dir` / `file` support the template variables (`${CONTEST_ID}` / `${TASK_ID}` / `${TASK_CHAR}`). A leading `~` in `parent` expands to the home directory. Template bodies are resolved in this order: `[setup.template]` by extension → builtin preset template for standard names (`main.cpp`, ...) → empty file.

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
