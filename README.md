# nl-sh: Natural Language Shell

## Introduction

`nl-sh` stands for Natural Language Shell, a novel approach to interacting with POSIX systems through natural language processing. This project aims to bridge the gap between traditional shell command execution and natural language interpretation, providing a user-friendly interface for executing complex shell commands using simple English prompts. This is a proof-of-concept. `nl-sh` makes working in terminals more accessible and intuitive, as the shell accepts both traditional POSIX commands as well as human-language instructions for system operations; the best command sequence satisfying that human prompt is provided back to the user for verification before execution.

The point of this shell is to elimiate the tedious tasks of googling Stack Overflow or grepping man pages to figure out the right combination of commands, pipes and flags to complete complex operations from the command line. This provides a human-centered CLI experience, for those who spend time on *NIX variants but are poorly-versed in the entire command vocabulary.

`nl-sh` is best demonstrated with some examples:

```
  ~/code/nl-sh ~>> ./target/release/nl-sh 
  > [nl-sh] /Users/mike/code/nl-sh $ whoami
  mike
  
  > [nl-sh] /Users/mike/code/nl-sh $ pwd
  /Users/mike/code/nl-sh
  
  > [nl-sh] /Users/mike/code/nl-sh $ ps aux |grep git
  mike             30936   0.0  0.0 408490720    656 s008  R+    9:01AM   0:00.00 grep git
  mike             30934   0.0  0.0 408636976   2416 s008  S+    9:01AM   0:00.01 /bin/zsh -c ps aux |grep git

  > [nl-sh] /Users/mike/code/nl-sh $ copy remote directory /home/mike/log_data on remote host "log-server" to the local machine; store the data in a new directory called "log-server-data"
  > scp -r log-server:/home/mike/log_data log-server-data Yes
  mike@log-server's password: 

  > [nl-sh] /Users/mike/code/nl-sh $ list the contents of the "log-server-data" directory with human-readable file sizes
  > ls -lh log-server-data Yes
  total 953984
  -rw-r--r--  1 mike  staff    82M Feb 13 09:08 0.log
  -rw-r--r--  1 mike  staff    82M Feb 13 09:08 1.log
  -rw-r--r--  1 mike  staff    82M Feb 13 09:08 2.log
  -rw-r--r--  1 mike  staff   163M Feb 13 09:08 3.log

  > [nl-sh] /Users/mike/code/nl-sh $ ls -lhs log-server-data
  total 953984
  196672 -rw-r--r--  1 mike  staff    82M Feb 13 09:08 0.log
  198464 -rw-r--r--  1 mike  staff    82M Feb 13 09:08 1.log
  196672 -rw-r--r--  1 mike  staff    82M Feb 13 09:08 2.log
  362176 -rw-r--r--  1 mike  staff   163M Feb 13 09:08 3.log

  > [nl-sh] /Users/mike/code/nl-sh $ Which 3 running processes on this machine have consumed the most CPU resources?
  > ps -arx -o pid,ppid,%cpu,comm | head -n 4 Yes
    PID  PPID  %CPU COMM
    564     1  31.1 /System/Library/PrivateFrameworks/SkyLight.framework/Resources/WindowServer
  17422     1  13.8 /System/Library/Frameworks/WebKit.framework/Versions/A/XPCServices/com.apple.WebKit.WebContent.xpc/Contents/MacOS/com.apple.WebKit.WebContent
   1295     1  10.0 /System/Volumes/Preboot/Cryptexes/App/System/Applications/Safari.app/Contents/MacOS/Safari
```

Here's a video:

https://github.com/mikecvet/nl-sh/assets/275631/6ab96b9e-6e6d-411b-ab59-2cd3986208fa

![nl-sh](https://github.com/mikecvet/nl-sh/assets/275631/1f45f93a-a5e3-45ad-a4af-e3c92aa90cbb)


## What is this?

- **POSIX Command Execution:** Run regular shell commands
- **Natural Language Processing:** Interprets natural language prompts to generate and then conditionally execute complex compositions of system commands and arguments
- **Rust-Based:** Written in Rust, leveraging some excellent crates for UI prompts and OpenAI APIs.

## How does this work?

`nl-sh` acts like a (minimal) shell by wrapping the underlying shell with a prompt which accepts and executes text inputs from the user. Inputs which seem like actual POSIX commands are directly executed; otherwise the input is directed to a model (currently just GPT4) to interpret the input and provide a command sequence satisfying the request, customized for the local system powering the shell. This shell collects some environmental data such as kernal and OS version details to try and generate the most accurate command-line sequence for the given POSIX variant.
