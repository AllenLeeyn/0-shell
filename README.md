# 0-shell

A minimal Unix shell written in Rust. It provides basic navigation, file operations, and built-in commands without spawning external processes or calling system binaries.

## Features

- **REPL**: Read-evaluate-print loop with prompt, line editing, and history
- **Built-in commands**: Implemented from scratch using Rust's standard library (see [Built-in Commands](docs/COMMANDS.md))
- **History**: Arrow keys (↑/↓) to navigate and re-run previous commands
- **Quoting**: Single quotes, double quotes, and backslash escapes
- **Chaining**: Multiple commands per line with semicolons (`;`)
- **Errors**: Clear messages to stderr; stdout/stderr kept separate
- **Platform**: Unix-like systems (Linux, macOS); not supported on Windows

## Building and Running

**Prerequisites:** Rust toolchain (1.70+).

```bash
cargo build
cargo run
```

Release binary:

```bash
cargo build --release
./target/release/zero-shell
```

**Exit:** `exit` or `Ctrl+D` (EOF).

## Architecture

The project is organized into three main areas:

- **`src/main.rs`**: REPL loop, prompt generation, and rustyline-based input reading
- **`src/command/`**: Built-in commands and command registry (`mod.rs`); each command in its own file (e.g. `ls.rs`, `cp.rs`, `echo.rs`)
- **`src/command_call.rs`**: Command parsing, tokenization, quote handling, and flag/arg separation

## Read-Evaluate-Print Loop (REPL)

The REPL is the core of the shell: read input, parse into command calls, execute each command, print results, repeat. Implemented in `src/main.rs` (`main()`) with parsing in `src/command_call.rs` and execution in `src/command/mod.rs`.

### REPL Flow

1. **Read**: Show prompt, read a full line (with continuation until quotes balance)
2. **Parse**: Split by `;`, tokenize with quote/escape handling, separate flags from args → `Vec<CommandCall>`
3. **Evaluate**: For each `CommandCall`, run `CommandList::execute(name, flags, args)`
4. **Print**: Write command stdout and stderr; exit if `result.should_exit`
5. **Loop**: Go back to step 1

### Command Parsing (part of REPL)

Parsing is done in `src/command_call.rs`:

- **`parse_line(input)`**: Splits on `;`, then for each segment calls `parse_chunk()` to produce one `CommandCall` (name, flags, args). Empty segments are skipped.
- **Tokenization** (`tokenize()`): Respects single quotes (literal), double quotes (with `\"`, `\\`, `\$` escapes), and backslash outside quotes (any character). Whitespace separates tokens.
- **Flags vs args** (`separate_flags_from_args()`): Tokens starting with `-` (except bare `-`) are flags. Short combos like `-la` become `["-l", "-a"]`; long flags like `--help` stay as one token. The first token that does not start with `-` and all following tokens are positional arguments.
- **Unclosed quotes**: `unclosed_quote_prompt(input)` returns `Some("> ")` when quotes are not balanced, so the REPL can show a continuation prompt and keep reading.

**Example:** `ls -la /tmp; echo "hello world"` → two `CommandCall`s: `{ name: "ls", flags: ["-l", "-a"], args: ["/tmp"] }` and `{ name: "echo", flags: [], args: ["hello world"] }`.

### REPL Components

#### 1. Prompt (`get_prompt()`)

In `src/main.rs`: uses `env::current_dir()`, abbreviates home with `~`, formats as `{path} $` or `~{suffix} $`.

#### 2. Input Reading (`read_line_with_history()`)

Uses **rustyline** (`DefaultEditor`). Reads lines until `unclosed_quote_prompt()` is `None`. Continuation prompt when quotes are unclosed is **`> `** (single prompt for any unclosed quote). Non-empty completed input is added to history. Ctrl+D returns `None` (exit) if the current buffer is empty, otherwise returns the buffer. Ctrl+C clears the current input and resets to the main prompt.

**Features:** Command history (↑/↓), line editing, multi-line input with `> ` continuation, EOF (Ctrl+D) to exit, Ctrl+C to cancel input.

#### 3. Command Execution

`CommandList::execute()` in `src/command/mod.rs`: resolves `help` to list commands; otherwise looks up the command, handles `--help`/`-h`, checks required args, then invokes the command callback. Returns `CommandResult` (stdout, stderr, `should_exit`).

#### 4. Output

`write_command_result()` in `src/main.rs`: writes stdout as-is, writes stderr then a newline; both flushed.

## [Built-in Commands](docs/COMMANDS.md)

All commands are built-in (no external binaries or process spawning). Implementations live in `src/command/<name>.rs`; the registry is in `src/command/mod.rs`. See the linked doc for the full list, usage, and options.

## [History Expansion](docs/HISTORY_EXPANSION.md)

History expansion (e.g. `!!`, `!ls`, `^old^new^`) is **not implemented** in 0-shell. The linked doc describes the feature and how it works in other shells.

## Error Handling

- **Unknown command:** `0-shell: <name>: command not found`
- **Missing arguments:** `<name>: missing operand.` plus a hint to `help` or `--help`
- **File/directory operations:** Errors from the filesystem APIs are printed to stderr

All errors go to stderr; normal output goes to stdout.

## Testing

```bash
cargo test
```

Tests cover parsing and tokenization (`command_call.rs`), command behavior, and edge cases (empty input, missing files, etc.).

## Constraints

- No external binaries or process spawning; all behavior is from built-in Rust code.
- Targets Unix-like systems only; follows common shell conventions.

## License

This project is part of an educational exercise to build a minimal shell from scratch.
