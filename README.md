standalone unix shell for an embedded Linux environment that handles:
- basic navigation
- file manipulation
- process control
- faithfully mimicking essential sjell behaviors without relying on existing shell utilities

- manage I/O within a shell loop
- robust error handling

Your minimalist shell must:
- Display a prompt (`$ `) and wait for user input
- Parse and execute user commands
- Return to the prompt only after command execution completes
- Handle `Ctrl+D` (EOF) gracefully to exit the shell

Must implement following commands **from scratch**, using system-level Rust abstractions:
- `echo`
- `cd`
- `ls` (supporting `-l`, `-a`, `-F`)
- `pwd`
- `cat`
- `cp`
- `rm` (supporting `-r`)
- `mv`
- `mkdir`
- `exit`

Additional constraints:

- Do **not** use any external binaries or system calls that spawn them
- If a command is unrecognized, print:  
  `Command '<name>' not found`
- Shell behavior should align with Unix conventions

Implementation plan
```
go into loop
ctrl+D to exit
listen for input 
register the different commands (long, short, help, options, callback)
0. exit
1. echo
2. pwd

```