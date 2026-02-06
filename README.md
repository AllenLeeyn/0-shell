# 0-shell

A standalone Unix shell implementation written in Rust for embedded Linux environments. This minimalist shell handles basic navigation, file manipulation, and process control, faithfully mimicking essential shell behaviors without relying on existing shell utilities.

## Features

- **Read-Evaluate-Print Loop (REPL)**: Interactive command-line interface
- **Built-in Commands**: All commands implemented from scratch using Rust standard library
- **Command History**: Navigate through previously executed commands using arrow keys (↑/↓)
- **Quote Handling**: Supports single quotes, double quotes, and escape sequences
- **Command Chaining**: Execute multiple commands with semicolons (`;`)
- **Error Handling**: Robust error messages and graceful failure handling
- **Cross-platform**: Works on Unix-like systems (Linux, macOS) with Windows compatibility considerations

## Building and Running

### Prerequisites

- Rust toolchain (1.70+)
- Cargo package manager

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

Or run the release version:

```bash
cargo build --release
./target/release/zero-shell
```

### Exit the Shell

- Type `exit` and press Enter
- Press `Ctrl+D` (EOF) to gracefully exit

## Architecture

The project is organized into three main modules:

- `**main.rs**`: Contains the REPL loop and prompt generation
- `**command.rs**`: Implements all built-in commands and command registry
- `**command_call.rs**`: Handles command parsing, tokenization, and quote processing

## Read-Evaluate-Print Loop (REPL)

The REPL is the core of the shell, implementing a continuous cycle of reading user input, evaluating commands, and printing results. The implementation is located in `src/main.rs` in the `main()` function.

### REPL Flow

The REPL follows this cycle:

1. **Read**: Display prompt and read user input
2. **Parse**: Tokenize and parse the input into command calls
3. **Evaluate**: Execute each command in sequence
4. **Print**: Display command output (stdout/stderr)
5. **Loop**: Return to step 1

### Code Implementation

The REPL loop is implemented in `src/main.rs` starting at line 16:

```rust
loop {
    // 1. READ: Display prompt and read input
    let prompt = get_prompt();
    stdout.write_all(prompt.as_bytes())?;
    stdout.flush()?;
    
    let mut line = String::new();
    let bytes_read = io::stdin().read_line(&mut line)?;
    
    if bytes_read == 0 {
        break; // EOF (Ctrl+D) - exit gracefully
    }
    
    // 2. PARSE: Tokenize and parse the command line
    let raw_input = line.trim_end();
    let calls = parse_line(raw_input);
    
    // 3. EVALUATE & 4. PRINT: Execute commands and display output
    for call in calls {
        let result = cmds.execute(call.name, call.flags, call.args);
        
        if result.should_exit {
            return Ok(()); // Exit if 'exit' command was executed
        }
        
        // Print stdout
        if !result.stdout.is_empty() {
            stdout.write_all(result.stdout.as_bytes())?;
            stdout.flush()?;
        }
        
        // Print stderr
        if !result.stderr.is_empty() {
            stderr.write_all(format!("{}\n", result.stderr).as_bytes())?;
            stderr.flush()?;
        }
    }
}
```

### REPL Components

#### 1. Prompt Generation (`get_prompt()`)

Located at lines 58-71 in `src/main.rs`, the prompt function:

- Retrieves the current working directory using `env::current_dir()`
- Replaces the home directory path with `~` for brevity
- Formats the prompt as `{path} $`  or `~{relative_path} $`

**Example outputs:**

- `/home/user/projects $` 
- `~/projects $`  (when in a subdirectory of HOME)

#### 2. Input Reading: History-Enabled Line Editor

The shell uses **rustyline** for input handling, providing advanced line editing capabilities and command history navigation. The implementation is located in `src/main.rs` in the `read_line_with_history()` function.

**History-Enabled Input (Current Implementation)**

```rust
let mut rl = DefaultEditor::new()?;
let line = rl.readline(prompt)?;
```

**Features:**

- **Command History**: All non-empty commands are automatically saved to history
- **Arrow Key Navigation**: Use ↑ (Up) and ↓ (Down) arrow keys to navigate through previous commands
- **Line Editing**: Full line editing support (cursor movement, deletion, etc.)
- **Multi-line Support**: Handles continuation prompts for unclosed quotes (`dquote>` , `squote>` )
- **EOF Handling**: Ctrl+D gracefully exits the shell
- **Interrupt Handling**: Ctrl+C clears current input and starts fresh

**How History Works:**

1. **Storing Commands**: After reading a complete command (with balanced quotes), non-empty commands are automatically added to history
2. **Navigation**:
  - Press **↑** (Up Arrow) to cycle backward through history
  - Press **↓** (Down Arrow) to cycle forward through history
  - When at the most recent command, pressing ↓ returns to an empty line
3. **Editing**: You can edit any command from history before executing it
4. **Multi-line Commands**: Multi-line commands (with continuation prompts) are stored as complete entries

**Example Usage:**

```bash
$ echo hello
hello
$ ls -la
[directory listing]
$ pwd
/home/user
# Press ↑ to recall "pwd"
# Press ↑ again to recall "ls -la"
# Press ↑ again to recall "echo hello"
# Press ↓ to go forward again
```

**Continuation Prompts:**

When quotes are not balanced, the shell shows continuation prompts:

- `dquote>`  for unclosed double quotes
- `squote>`  for unclosed single quotes

History navigation works during continuation prompts as well, allowing you to recall and edit previous multi-line commands.

**Special Case: `cat` Without Arguments**

When `cat` is called without arguments, it reads from stdin line-by-line for immediate echo (lines 16-26 in `src/command/cat.rs`):

```rust
// In interactive mode, echo lines immediately to stdout
while let Ok(n) = handle.read_line(&mut line) {
    if n == 0 {
        break; // EOF
    }
    stdout.write_all(line.as_bytes())?;
    stdout.flush()?;
    line.clear();
}
```

This maintains the line-based approach even for streaming input, providing immediate feedback as the user types each line.

#### 3. Command Parsing (`parse_line()`)

Located in `src/command_call.rs` (lines 29-52), this function:

- Splits input by semicolons (`;`) to support command chaining
- Tokenizes each command segment with quote and escape handling
- Separates flags from positional arguments
- Returns a vector of `CommandCall` structures

**Parsing features:**

- Single quotes: Literal text (no escaping)
- Double quotes: Supports escaping of `"`, `\`, and `$`
- Backslash escapes: Outside quotes, escapes any character
- Flag expansion: `-la` expands to `-l` and `-a`

**Example:**

```bash
ls -la /tmp; echo "hello world"
```

Parses into two `CommandCall` objects:

1. `{name: "ls", flags: ["-l", "-a"], args: ["/tmp"]}`
2. `{name: "echo", flags: [], args: ["hello world"]}`

#### 4. Command Execution

The `CommandList::execute()` method (in `src/command.rs`, lines 91-132):

- Looks up the command in the registry
- Validates required arguments
- Handles `--help` and `-h` flags
- Calls the command's callback function
- Returns a `CommandResult` with stdout, stderr, and exit flag

#### 5. Output Handling

The REPL separates stdout and stderr:

- **stdout**: Written directly to terminal (line 44)
- **stderr**: Written with newline appended (line 49)
- Both streams are flushed immediately for real-time output

## Built-in Commands

All commands are implemented from scratch using Rust's standard library. No external binaries or system calls that spawn processes are used.

### `exit`

**Usage:** `exit`

**Description:** Terminates the shell and returns control to the parent process.

**Implementation:** Located in `src/command.rs` at `exit_callback()` (line 235). Returns a `CommandResult` with `should_exit` set to `true`, which causes the REPL loop to break.

**Example:**

```bash
$ exit
```

---

### `echo`

**Usage:** `echo [OPTIONS] [TEXT...]`

**Options:**

- `-e`: Interpret backslash escape sequences

**Description:** Displays a line of text. By default, prints arguments literally. With `-e`, interprets escape sequences like `\n`, `\t`, etc.

**Implementation:** Located in `src/command.rs` at `echo_callback()` (line 242). Joins all arguments with spaces and prints them. When `-e` flag is present, processes escape sequences through `map_echo_escape()` (line 291).

**Supported escape sequences (with `-e`):**

- `\a`: Alert (BEL)
- `\b`: Backspace
- `\e`: Escape
- `\f`: Form feed
- `\n`: Newline
- `\r`: Carriage return
- `\t`: Tab
- `\v`: Vertical tab
- `\\`: Backslash
- `\c`: Stop output (no newline)

**Examples:**

```bash
$ echo hello world
hello world

$ echo "hello world"
hello world

$ echo -e "hello\nworld"
hello
world

$ echo -e "hello\tworld"
hello    world
```

---

### `pwd`

**Usage:** `pwd`

**Description:** Prints the current working directory's absolute path.

**Implementation:** Located in `src/command.rs` at `pwd_callback()` (line 307). Uses `env::current_dir()` to retrieve the current directory and displays it.

**Example:**

```bash
$ pwd
/home/user/projects/0-shell
```

---

### `cd`

**Usage:** `cd [DIRECTORY]`

**Description:** Changes the current working directory. If no directory is specified, changes to the user's home directory (from `HOME` environment variable).

**Implementation:** Located in `src/command.rs` at `cd_callback()` (line 320). Uses `env::set_current_dir()` to change directories. Defaults to `HOME` environment variable if no argument is provided, or `/` if `HOME` is not set.

**Examples:**

```bash
$ cd /tmp
$ pwd
/tmp

$ cd
$ pwd
/home/user

$ cd ../parent
$ pwd
/home/user/parent
```

---

### `ls`

**Usage:** `ls [OPTIONS] [FILE...]`

**Options:**

- `-a`: List all entries, including hidden files (starting with `.`)
- `-l`: Use long listing format (permissions, size, date, name)
- `-F`: Append indicator characters (`/` for directories, `*` for executables)

**Description:** Lists directory contents. If no path is specified, lists the current directory.

**Implementation:** Located in `src/command.rs` at `ls_callback()` (line 555). Uses `fs::read_dir()` to read directory entries. Supports multiple paths, showing each path's header when multiple are specified.

**Long format details:**

- Permissions: Unix-style (e.g., `drwxr-xr-x`)
- Size: File size in bytes
- Date: Modification time in `MMM DD HH:MM` format
- Name: File/directory name with type indicators when `-F` is used

**Examples:**

```bash
$ ls
file1.txt  file2.txt  directory1

$ ls -a
.  ..  .hidden  file1.txt  file2.txt

$ ls -l
-rw-r--r--     1024 Dec 15 14:30 file1.txt
drwxr-xr-x     4096 Dec 15 14:31 directory1

$ ls -laF
drwxr-xr-x     4096 Dec 15 14:31 ./
drwxr-xr-x     4096 Dec 15 14:30 ../
-rw-r--r--     1024 Dec 15 14:30 file1.txt
drwxr-xr-x     4096 Dec 15 14:31 directory1/
```

---

### `mkdir`

**Usage:** `mkdir DIRECTORY...`

**Description:** Creates one or more directories. Supports nested paths and will create parent directories as needed.

**Implementation:** Located in `src/command.rs` at `mkdir_callback()` (line 337). Uses `fs::create_dir_all()` which creates directories recursively. If a directory already exists, it's silently skipped (no error).

**Examples:**

```bash
$ mkdir newdir
$ mkdir parent/child/grandchild
$ mkdir dir1 dir2 dir3
```

---

### `cat`

**Usage:** `cat [FILE...]`

**Description:** Concatenates and prints files to standard output. If no files are provided, reads from standard input until EOF (Ctrl+D).

**Implementation:** Located in `src/command.rs` at `cat_callback()` (line 355). Uses `File::open()` and `BufReader` to read files. When no arguments are provided, reads from stdin line by line and echoes immediately.

**Examples:**

```bash
$ cat file.txt
This is the content of file.txt

$ cat file1.txt file2.txt
Content of file1
Content of file2

$ cat
Type here and press Enter
Type here and press Enter
^D
```

---

### `cp`

**Usage:** `cp SOURCE DEST` or `cp SOURCE... DIRECTORY`

**Description:** Copies files and directories. If the destination is a directory, copies the source(s) into that directory. If multiple sources are provided, the destination must be a directory.

**Implementation:** Located in `src/command.rs` at `cp_callback()` (line 423). Uses `fs::copy()` for file copying. The `resolve_destination()` helper function (line 409) handles the case where the destination is a directory by appending the source filename.

**Limitations:** Currently only supports file copying. Directory copying with `-r` flag is not yet implemented.

**Examples:**

```bash
$ cp source.txt dest.txt
$ cp file1.txt file2.txt /tmp
$ cp source.txt /tmp/dest.txt
```

---

### `mv`

**Usage:** `mv SOURCE DEST` or `mv SOURCE... DIRECTORY`

**Description:** Moves (renames) files and directories. If the destination is a directory, moves the source(s) into that directory. If multiple sources are provided, the destination must be a directory.

**Implementation:** Located in `src/command.rs` at `mv_callback()` (line 469). Uses `fs::rename()` which works for both files and directories. The `resolve_destination()` helper function handles directory destinations.

**Examples:**

```bash
$ mv old.txt new.txt
$ mv file1.txt file2.txt /tmp
$ mv olddir newdir
```

---

### `rm`

**Usage:** `rm [OPTIONS] FILE...`

**Options:**

- `-r` or `-R`: Recursively remove directories and their contents

**Description:** Removes files or directories. Without `-r`, directories cannot be removed (returns an error). With `-r`, recursively removes directories and all their contents.

**Implementation:** Located in `src/command.rs` at `rm_callback()` (line 516). Uses `fs::remove_file()` for files and `fs::remove_dir_all()` for recursive directory removal. Validates that paths exist and checks if a directory is being removed without the `-r` flag.

**Examples:**

```bash
$ rm file.txt
$ rm -r directory
$ rm file1.txt file2.txt file3.txt
```

---

## Command Parsing Details

The command parser (`src/command_call.rs`) handles complex input scenarios:

### Quote Handling

- **Single quotes (`'`)**: Everything inside is treated literally, no escaping
- **Double quotes (`"`)**: Supports escaping of `"`, `\`, and `$` with backslash
- **Backslash (`\`)**: Outside quotes, escapes any following character

**Examples:**

```bash
$ echo 'hello world'        # Single token: "hello world"
$ echo "hello world"        # Single token: "hello world"
$ echo hello\ world         # Single token: "hello world"
$ echo "hello \"world\""    # Token: 'hello "world"'
```

### Flag Parsing

Flags are automatically separated from arguments:

- Short flags can be combined: `-la` → `["-l", "-a"]`
- Long flags are preserved: `--help` → `["--help"]`
- Flags must come before positional arguments

**Examples:**

```bash
$ ls -la /tmp              # flags: ["-l", "-a"], args: ["/tmp"]
$ ls --all -l /tmp          # flags: ["--all", "-l"], args: ["/tmp"]
```

### Command Chaining

Multiple commands can be chained with semicolons:

```bash
$ ls -l; pwd; echo done
```

Each command is executed sequentially, and the shell waits for each to complete before executing the next.

## History Expansion

History expansion is a shell feature that allows users to reference and reuse previous commands from the command history using the `!` (exclamation mark) character. This feature enables quick command repetition, modification, and recall without retyping entire commands.

### Overview

History expansion occurs immediately after reading a complete command line, before word splitting and command execution. The `!` character serves as a special operator that triggers history substitution, allowing users to:

- Repeat the previous command
- Reference commands by their position in history
- Search for commands by their starting text
- Extract specific arguments from previous commands
- Modify previous commands before re-execution

### Event Designators

Event designators select which history entry to use:

- `!!` - Refer to the previous command (most common usage)
- `!n` - Refer to history entry number `n` (e.g., `!42` executes command #42)
- `!-n` - Refer to the entry `n` commands ago (e.g., `!-2` executes the command from two commands back)
- `!string` - Most recent command starting with `string` (e.g., `!ls` executes the most recent `ls` command)
- `!?string[?]` - Most recent command containing `string` (e.g., `!?file` finds commands containing "file")
- `^string1^string2^` - Quick substitution: repeat last command replacing `string1` with `string2`
- `!#` - The entire command line typed so far

### Word Designators

Word designators select specific words (arguments) from a history entry, separated by `:`:

- `0` - The command word itself
- `n` - The nth word (1-indexed)
- `^` - First argument (word 1)
- `$` - Last word
- `*` - All words except the command (words 1 through last)
- `x-y` - Range of words from x to y

**Examples:**

- `!!:$` - Gets the last word (argument) of the previous command
- `!ls:2` - Gets the second argument of the most recent `ls` command
- `!!:*` - Gets all arguments from the previous command

### Escaping and Disabling

The `!` character can be escaped or inhibited:

- **Backslash (`\`)**: Escapes the `!` character, treating it literally
- **Single quotes (`'`)**: Inhibits history expansion entirely within quotes
- **Double quotes (`"`)**: History expansion still occurs, but can be escaped with backslash

#### Behavior Inside Quotes

**Important:** History expansion behavior differs significantly between single and double quotes:

1. **Single quotes (`'`) - History expansion is disabled:**
  ```bash
   $ echo 'Hello!'
   Hello!
  ```
   The `!` inside single quotes is treated as a literal character. No history expansion occurs.
2. **Double quotes (`"`) - History expansion is enabled:**
  ```bash
   $ echo "something!"
  ```
   **With history expansion ON:** The shell will attempt to expand `!` as a history reference. If `!` is followed by a space, newline, or other non-expandable character, it may be treated literally, but if followed by valid history expansion syntax (like `!!`, `!n`, `!string`), it will trigger expansion.
   **Example with valid expansion:**
   **Example with literal `!`:**
   If `!` is at the end of a word and not followed by a valid expansion pattern, it's typically treated literally in modern shells.
3. **Escaping inside double quotes:**
  ```bash
   $ echo "something\!"
   something!
  ```
   Using backslash before `!` inside double quotes prevents history expansion.

#### Enabling and Disabling History Expansion

History expansion can be controlled via shell options:

**Disable history expansion:**

##### zsh
```zsh
setopt NO_BANG_HIST
```

##### bash
```bash
set +H
```

**Enable history expansion:**

##### zsh
```zsh
setopt BANG_HIST
```

##### bash
```bash
set -H
```

**Check current status:**

```bash
set -o | grep histexpand
# or
echo $-
# Look for 'H' in the output (H = history expansion enabled)
```

**Behavior when disabled:**
When history expansion is disabled (`set +H`), the `!` character is treated as a literal character everywhere:

```bash
$ set +H
$ echo "something!"
something!
$ echo 'Hello!'
Hello!
$ !!
!!
```

All `!` characters are treated literally, and history references like `!!` will not be expanded.

**Behavior when enabled (default in interactive bash):**

```bash
$ set -H
$ echo "something!"
something
$ ls -la
$ !!
ls -la
```

History expansion is active, and `!!` references the previous command.

**Note:** In bash, history expansion is enabled by default in interactive shells but disabled in non-interactive shells (scripts). The `set +H` / `set -H` commands control this behavior.

### Common Use Cases

1. **Repeat last command:**
  ```bash
   $ ls -la /tmp
   file1.txt  file2.txt
   $ !!
   ls -la /tmp
   file1.txt  file2.txt
  ```
2. **Repeat with modification:**
  ```bash
   $ cat file1.txt
   $ ^file1^file2^
   cat file2.txt
  ```
3. **Reuse arguments:**
  ```bash
   $ ls -l /home/user/documents
   $ cd !!:$
   cd /home/user/documents
  ```
4. **Search by prefix:**
  ```bash
   $ ls -la /tmp
   $ mkdir newdir
   $ !ls
   ls -la /tmp
  ```

### Implementation Status

History expansion is currently **not implemented** in 0-shell (see tasklist). When implemented, it would require:

- Maintaining a command history buffer
- Parsing `!` sequences before command execution
- Resolving history references to actual command strings
- Handling edge cases (empty history, invalid references, etc.)

**Note:** History expansion is a powerful but potentially dangerous feature. In production shells, it's often disabled by default in non-interactive shells and can be controlled via shell options (e.g., `set +H` to disable in bash).

## Error Handling

The shell provides clear error messages:

- **Unknown command**: `0-shell: <command>: command not found`
- **Missing arguments**: `<command>: missing operand`
- **File operations**: Standard error messages from Rust's filesystem operations
- **Directory operations**: Clear messages when operations fail

All errors are written to stderr, while normal output goes to stdout.

## Testing

The project includes comprehensive unit tests. Run tests with:

```bash
cargo test
```

Tests cover:

- Command parsing and tokenization
- Individual command functionality
- Error handling
- Edge cases (empty input, missing files, etc.)

## Constraints

- **No external binaries**: All functionality is implemented using Rust standard library
- **No process spawning**: Commands are built-in functions, not external programs
- **Unix conventions**: Shell behavior aligns with standard Unix shell conventions

## License

This project is part of an educational exercise to build a minimal shell from scratch.

```

#### Tasklist
- [o] add handling of ! (Sergei)
- [o] add proper formatting for ls (Gigi)
- [o] add ls -laF to list similar to bash (Allen)
- [o] add command history
- [ ] add clear command (Sergei)
- [ ] normalize ordering with ls. currently based on ascii values (Gigi)
- [ ] fix inconsistent xattr (Allen)
```

