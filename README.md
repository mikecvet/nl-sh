# nl-sh: Natural Language Shell

## Introduction

`nl-sh` stands for Natural Language Shell, a novel approach to interacting with POSIX systems through natural language understanding through LLMs. This project aims to bridge the gap between traditional shell command execution and natural language interpretation, providing a user-friendly interface for executing complex system commands using simple English prompts. This is functional proof-of-concept. `nl-sh` makes working in terminals more accessible and intuitive, as the shell accepts both traditional POSIX commands as well as human-language instructions for system operations; the best command sequence satisfying that human prompt is provided back to the user for verification before execution.

The point of this shell is to eliminate the tedious manual tasks of googling Stack Overflow or grepping through man pages to figure out the right combination of commands, arguments, pipes and flags to complete complex operations from the command line. This provides a human-centered CLI experience, for those who spend time on *NIX variants but are poorly-versed in the entire command vocabulary.

This is discussed a little further [here](https://mikecvet.medium.com/nl-sh-the-natural-language-shell-ad2ddc2e13a7)

### Key Features:

- Can use in any window-based or remote terminal setting, as an overlay on top of the current envionment's shell
- Accepts either direct system commands, or expressive language-based commands, which are interpreted by an LLM
- Loads prior shell history (bash, ksh, tcsh, zsh), allows history navigation and updates underlying shell history with executed commands
- Supports GPT 3.5 Turbo, GPT 4, and any locally-available GGUF LLM model

In particular, I've had success with `Mistral 7B Instruct`, and `CodeLlama 7B`. Some other OS models, such as raw `Llama2` do not work as well; often because they're a little too chatty and helpful =)

`nl-sh` is best demonstrated with some examples (default behavior is to use GPT4 and assume `OPENAI_API_KEY` is available):

```
  ~/code/nl-sh ~>> ./target/release/nl-sh --help 
  A natural language shell for *NIX systems

  Usage: nl-sh [OPTIONS]

  Options:
      --gpt4          Use the GPT4 API as a backend, reading from the OPENAI_API_KEY environment variable. Default behavior.
      --gpt35         Use the GPT3.5 Turbo API as a backend, reading from the OPENAI_API_KEY environment variable
      --local <path>  Use a local GGUF-based model as a backend, located at the provided path
      --stateless     Disable update of external shell history (default: false)

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

  > [nl-sh] /Users/mike/code/nl-sh $ Which 3 running processes on this machine have consumed the most CPU resources?
  > ps -arx -o pid,ppid,%cpu,comm | head -n 4 Yes
    PID  PPID  %CPU COMM
    564     1  31.1 /System/Library/PrivateFrameworks/SkyLight.framework/Resources/WindowServer
  17422     1  13.8 /System/Library/Frameworks/WebKit.framework/Versions/A/XPCServices/com.apple.WebKit.WebContent.xpc/Contents/MacOS/com.apple.WebKit.WebContent
   1295     1  10.0 /System/Volumes/Preboot/Cryptexes/App/System/Applications/Safari.app/Contents/MacOS/Safari
```

Now with a local model:

```
  ~/code/nl-sh ~>> ./target/release/nl-sh --local ~/Downloads/mistral-7b-instruct-v0.2.Q8_0.gguf
  > [nl-sh] /Users/mike/code/nl-sh $ show me the top 3 largest files in the current directory, in human-readable size format
  > du -ah | sort -rh | head -n 3 (Y/n)
  79G   .
  32G   ./gemma-7b.gguf
 9.3G   ./gemma-2b.gguf
```

Here's a video demonstration:

![nl-sh](https://github.com/mikecvet/nl-sh/assets/275631/fc89f418-49e9-428f-9c33-aed00f7cb169))

## How does this work?

`nl-sh` acts like a (minimal) shell by wrapping the underlying shell with a prompt which accepts and executes text inputs from the user. Inputs which seem like actual POSIX commands are directly executed; otherwise the input is directed to an LLM to interpret the input and provide a command sequence satisfying the request, customized for the local system powering the shell. This shell collects some environmental data such as kernel and OS version details to try and generate the most accurate command-line sequence for the given POSIX variant.

Any commands suggested by the backing LLM and then executed through `nl-sh` are written to the user's underlying shell command-history file.
