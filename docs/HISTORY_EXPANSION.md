# History Expansion

History expansion is a shell feature that allows users to reference and reuse previous commands from the command history using the `!` (exclamation mark) character. This feature enables quick command repetition, modification, and recall without retyping entire commands.

## Overview

History expansion occurs immediately after reading a complete command line, before word splitting and command execution. The `!` character serves as a special operator that triggers history substitution, allowing users to:

- Repeat the previous command
- Reference commands by their position in history
- Search for commands by their starting text
- Extract specific arguments from previous commands
- Modify previous commands before re-execution

## Event Designators

Event designators select which history entry to use:

- `!!` - Refer to the previous command (most common usage)
- `!n` - Refer to history entry number `n` (e.g., `!42` executes command #42)
- `!-n` - Refer to the entry `n` commands ago (e.g., `!-2` executes the command from two commands back)
- `!string` - Most recent command starting with `string` (e.g., `!ls` executes the most recent `ls` command)
- `!?string[?]` - Most recent command containing `string` (e.g., `!?file` finds commands containing "file")
- `^string1^string2^` - Quick substitution: repeat last command replacing `string1` with `string2`
- `!#` - The entire command line typed so far

## Word Designators

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

## Escaping and Disabling

The `!` character can be escaped or inhibited:

- **Backslash (`\`)**: Escapes the `!` character, treating it literally
- **Single quotes (`'`)**: Inhibits history expansion entirely within quotes
- **Double quotes (`"`)**: History expansion still occurs, but can be escaped with backslash

### Behavior Inside Quotes

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
   If `!` is at the end of a word and not followed by a valid expansion pattern, it's typically treated literally in modern shells.

3. **Escaping inside double quotes:**
   ```bash
   $ echo "something\!"
   something!
   ```
   Using backslash before `!` inside double quotes prevents history expansion.

### Enabling and Disabling History Expansion

History expansion can be controlled via shell options:

**Disable history expansion:**

- **zsh:** `setopt NO_BANG_HIST`
- **bash / 0-shell:** `set +H`

**Enable history expansion:**

- **zsh:** `setopt BANG_HIST`
- **bash / 0-shell:** `set -H`

**Check current status:**

```bash
set -o | grep histexpand
# or
echo $-
# Look for 'H' in the output (H = history expansion enabled)
```

**Behavior when disabled:** When history expansion is disabled (`set +H`), the `!` character is treated as a literal character everywhere. All `!` characters are treated literally, and history references like `!!` will not be expanded.

**Behavior when enabled (default in interactive bash):** History expansion is active, and `!!` references the previous command.

**Note:** In bash, history expansion is enabled by default in interactive shells but disabled in non-interactive shells (scripts). The `set +H` / `set -H` commands control this behavior.

## Common Use Cases

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

## Implementation Status

A **simple subset** of history expansion is implemented in 0-shell:

- **Implemented:** `!!` (previous command), `!n` (command number n, 1-based), `!-n` / `!+n` (n commands ago / 1-based), and `set -H` / `set +H` to enable/disable expansion (default: on).
- **Respected:** No expansion inside single quotes; backslash before `!` prevents expansion.
- **Not implemented:** `!string`, `!?string`, `^old^new^`, word designators (`:n`, `:$`, etc.), `!#`.

The shell uses the same history buffer as the line editor (rustyline). Expansion runs after reading a complete line and before parsing; the expanded line is what gets executed and added to history.

**Note:** History expansion is a powerful but potentially dangerous feature. In production shells, it's often disabled by default in non-interactive shells and can be controlled via shell options (e.g., `set +H` to disable in bash).
