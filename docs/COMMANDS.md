# Built-in Commands

All commands are implemented from scratch using Rust's standard library. No external binaries or system calls that spawn processes are used. Each command lives in `src/command/<name>.rs`; the registry and execution logic are in `src/command/mod.rs`.

---

### `exit`

**Usage:** `exit`

**Description:** Terminates the shell and returns control to the parent process.

**Implementation:** `src/command/exit.rs`. Returns a `CommandResult` with `should_exit` set to `true`, which causes the REPL loop to break.

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

**Implementation:** `src/command/echo.rs`. Joins all arguments with spaces. When `-e` is present, processes escape sequences.

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

**Implementation:** `src/command/pwd.rs`. Uses `env::current_dir()` and displays it.

**Example:**

```bash
$ pwd
/home/user/projects/0-shell
```

---

### `cd`

**Usage:** `cd [DIRECTORY]`

**Description:** Changes the current working directory. If no directory is specified, changes to the user's home directory (from `HOME` environment variable).

**Implementation:** `src/command/cd.rs`. Uses `env::set_current_dir()`. Defaults to `HOME` if no argument is provided, or `/` if `HOME` is not set.

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
- `-F`: Append indicator characters (`/` for directories, `@` for symlinks, `*` for executables)

**Description:** Lists directory contents. If no path is specified, lists the current directory. Implementation is in `src/command/ls.rs`; long format is produced by `ls_long()`, standard format by `ls_std()`.

**Long format details:**

- Permissions: Unix-style (e.g., `drwxr-xr-x`) with optional suffix `@` (extended attributes) or `+` (ACL only). For symlinks, the suffix reflects the **target's** attributes (path is canonicalized).
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
-rw-r--r--@    1024 Dec 15 14:30 file1.txt
drwxr-xr-x     4096 Dec 15 14:31 directory1

$ ls -laF
drwxr-xr-x     4096 Dec 15 14:31 ./
drwxr-xr-x     4096 Dec 15 14:30 ../
-rw-r--r--@    1024 Dec 15 14:30 file1.txt
drwxr-xr-x     4096 Dec 15 14:31 directory1/
```

---

### `clear`

**Usage:** `clear`

**Description:** Clears the terminal screen.

**Implementation:** `src/command/clear.rs`.

**Example:**

```bash
$ clear
```

---

### `mkdir`

**Usage:** `mkdir [-p] DIRECTORY...`

**Options:**

- `-p`: Create parent directories as needed; no error if directory exists

**Description:** Creates one or more directories. With `-p`, creates nested paths recursively.

**Implementation:** `src/command/mkdir.rs`. Uses `fs::create_dir_all()` when `-p` is used.

**Examples:**

```bash
$ mkdir newdir
$ mkdir -p parent/child/grandchild
$ mkdir dir1 dir2 dir3
```

---

### `cat`

**Usage:** `cat [FILE...]`

**Description:** Concatenates and prints files to standard output. If no files are provided, reads from standard input until EOF (Ctrl+D).

**Implementation:** `src/command/cat.rs`. Uses `File::open()` and `BufReader` to read files. When no arguments are provided, reads from stdin line by line and echoes immediately.

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

**Usage:** `cp SOURCE DEST` or `cp SOURCE... DIRECTORY` or `cp -r SOURCE... DIRECTORY`

**Options:**

- `-r`, `-R`, `--recursive`: Copy directories recursively

**Description:** Copies files and directories. If the destination is a directory, copies the source(s) into that directory. With `-r`, copies directories recursively. Copying a directory into itself is allowed; the self-subdirectory is skipped.

**Implementation:** `src/command/cp.rs`. Uses `fs::copy()` for files and recursive copy for directories when `-r` is given.

**Examples:**

```bash
$ cp source.txt dest.txt
$ cp file1.txt file2.txt /tmp
$ cp -r dir1 /tmp/backup
```

---

### `mv`

**Usage:** `mv SOURCE DEST` or `mv SOURCE... DIRECTORY`

**Description:** Moves (renames) files and directories. If the destination is a directory, moves the source(s) into that directory. If multiple sources are provided, the destination must be a directory.

**Implementation:** `src/command/mv.rs`. Uses `fs::rename()` for both files and directories. Directory destinations are resolved per source.

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

**Implementation:** `src/command/rm.rs`. Uses `fs::remove_file()` for files and `fs::remove_dir_all()` for recursive directory removal.

**Examples:**

```bash
$ rm file.txt
$ rm -r directory
$ rm file1.txt file2.txt file3.txt
```

---

### `help`

**Usage:** `help` or `<command> --help` / `<command> -h`

**Description:** Lists available built-in commands, or shows usage for a specific command.

**Implementation:** Handled in `CommandList::execute()` in `src/command/mod.rs`. The `help` name prints the command list; `--help` / `-h` on any command prints that command's usage string.

**Example:**

```bash
$ help
$ ls --help
```
