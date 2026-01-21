# 0-shell

A standalone Unix shell implementation written in Rust for embedded Linux environments. This minimalist shell handles basic navigation, file manipulation, and process control, faithfully mimicking essential shell behaviors without relying on existing shell utilities.

## Features

- **Read-Evaluate-Print Loop (REPL)**: Interactive command-line interface
- **Built-in Commands**: All commands implemented from scratch using Rust standard library
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

- **`main.rs`**: Contains the REPL loop and prompt generation
- **`command.rs`**: Implements all built-in commands and command registry
- **`command_call.rs`**: Handles command parsing, tokenization, and quote processing

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
- Formats the prompt as `{path} $ ` or `~{relative_path} $ `

**Example outputs:**
- `/home/user/projects $ `
- `~/projects $ ` (when in a subdirectory of HOME)

#### 2. Input Reading: Line-Based vs Buffer-Based

The shell uses **line-based input handling** via `io::stdin().read_line()` (line 23 in `src/main.rs`). This design choice prioritizes immediate feedback and user experience.

**Line-Based Input (Current Implementation)**

```rust
let mut line = String::new();
let bytes_read = io::stdin().read_line(&mut line)?;
```

**Characteristics:**
- **Reads until newline**: Blocks until the user presses Enter (`\n`)
- **Immediate processing**: Each line is processed as soon as Enter is pressed
- **Simple and predictable**: User sees immediate feedback after each command
- **Natural interaction**: Matches how users expect shells to work (command → Enter → result)
- **EOF handling**: Returns 0 bytes when Ctrl+D is pressed, allowing graceful exit

**Alternative: Buffer-Based Input**

A buffer-based approach would read raw bytes in chunks without waiting for newlines:

```rust
// Hypothetical buffer-based approach
let mut buffer = [0u8; 1024];
let bytes_read = io::stdin().read(&mut buffer)?;
// Process bytes as they arrive, need to manually detect newlines
```

**Trade-offs of Buffer-Based:**
- **Pros**: 
  - More control over input processing
  - Could support character-by-character processing (e.g., for advanced line editing)
  - Better for handling binary data or non-line-oriented input
- **Cons**:
  - More complex implementation (must manually detect line boundaries)
  - Requires buffering logic to reconstruct complete commands
  - Less intuitive for interactive shell use
  - Delayed feedback (commands only execute when buffer fills or newline detected)

**Why Line-Based for This Shell?**

We chose line-based input for several reasons:

1. **Immediate Feedback**: Users get instant results after pressing Enter, which is the expected shell behavior
2. **Simplicity**: `read_line()` handles newline detection automatically, reducing code complexity
3. **User Experience**: Matches standard shell conventions where commands are submitted with Enter
4. **Error Handling**: Clear boundaries (each line is a complete command) make error reporting straightforward
5. **Command Chaining**: Natural support for semicolon-separated commands on a single line

**Special Case: `cat` Without Arguments**

When `cat` is called without arguments, it reads from stdin line-by-line for immediate echo (lines 364-374 in `src/command.rs`):

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
