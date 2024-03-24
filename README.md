# nl-sh: Natural Language Shell

## Introduction

`nl-sh` stands for Natural Language Shell, a novel approach to interacting with POSIX systems through natural language understanding through LLMs. This project aims to bridge the gap between traditional shell command execution and natural language interpretation, providing a user-friendly interface for executing complex system commands using simple English prompts. This is functional proof-of-concept. `nl-sh` makes working in terminals more accessible and intuitive, as the shell accepts both traditional POSIX commands as well as human-language instructions for system operations; the best command sequence satisfying that human prompt is provided back to the user for verification before execution.

The point of this shell is to eliminate the tedious manual tasks of googling Stack Overflow or grepping through man pages to figure out the right combination of commands, arguments, pipes and flags to complete complex operations from the command line. This provides a human-centered CLI experience, for those who spend time on *NIX variants but are poorly-versed in the entire command vocabulary.

This is discussed a little further [here](https://mikecvet.medium.com/nl-sh-the-natural-language-shell-ad2ddc2e13a7)

### Key Features:

- Can use in any window-based or remote terminal setting, as an overlay on top of the current envionment's shell
- Accepts either direct system commands, or expressive language-based commands, which are interpreted by an LLM
- Loads prior shell history (bash, ksh, tcsh, zsh), allows history navigation and updates underlying shell history with executed commands
- Supports `GPT 3.5 Turbo`, `GPT 4`, `Claude 2.1`, and any locally-available open-source `GGUF` formatted LLM

I've had some success with `Mistral 7B Instruct`, and `CodeLlama 7B`, though they do not work as well as Claude or OpenAI's models. Some other open-source models, such as `Llama2` do not work as well at all; often because they're a little too chatty and helpful =)

`nl-sh` is best demonstrated with some examples (default behavior is to use GPT4 and assume `OPENAI_API_KEY` is available):

```
  ~/code/nl-sh ~>> ./target/release/nl-sh --help
  A natural language shell for *NIX systems
  
  Usage: nl-sh [OPTIONS]
  
  Options:
        --gpt4          Use the GPT4 API as a backend, reading from the OPENAI_API_KEY environment variable. Default behavior.
        --gpt35         Use the GPT3.5 Turbo API as a backend, reading from the OPENAI_API_KEY environment variable
        --claude        Use the Anthropic Claude API as a backend (default: Claude 3 Sonnet), reading from the CLAUDE_API_KEY environment variable
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

  > [nl-sh] /Users/mike/code/nl-sh $ Which 5 running processes on this machine have consumed the most CPU resources? Sanitize the process names by stripping out flags and pat
  hs where possible. Order the results by descending CPU usage, and ensure their CPU usage is emitted as a column next to the process name
  > ps aux | sort -nrk3 | head -5 | awk '{print $3, $11}' Yes
  
  4.3 /System/Library/PrivateFrameworks/SkyLight.framework/Resources/WindowServer
  2.8 /Applications/Visual
  2.4 /sbin/launchd
  2.3 /System/Applications/Utilities/Activity
  2.1 /Applications/Visual
```


`nl-sh` will also make an attempt to correct any failed commands based on a misunderstanding of the native system, or even user error, by collecting command outputs and reasoning about the failure. For example, on a Darwin UNIX system, the operator uses "iptables" as a shorthand for "firewall":

```
  > [nl-sh] /Users/mike/code/nl-sh $ list all iptables rules on this system
  > sudo iptables -L -v Yes
  Password:
  Executed [sudo iptables -L -v] and got error: sudo: iptables: command not found

  Retrying command formulation...
  > sudo pfctl -sa Yes
  success:
  TRANSLATION RULES:
  nat-anchor "com.apple/*" all
  rdr-anchor "com.apple/*" all

  FILTER RULES:
  scrub-anchor "com.apple/*" all fragment reassemble
  anchor "com.apple/*" all

  DUMMYNET RULES:
  dummynet-anchor "com.apple/*" all
  [...]
```

Local LLMs sometimes work as well:

```
  ~/code/nl-sh ~>> ./target/release/nl-sh --local ./mistral-7b-instruct-v0.2.Q8_0.gguf
  > [nl-sh] /Users/mike/code/nl-sh $ show me the top 3 largest files in the current directory, in human-readable size format
  > du -ah | grep gguf | sort -rh | head -n 3 Yes
   32G	./gemma-7b.gguf
  9.3G	./gemma-2b.gguf
  8.5G	./gemma-7b-it-Q8_0.gguf (Y/n)
```

Here's a video demonstration:

![nl-sh](https://github.com/mikecvet/nl-sh/assets/275631/685f4642-f331-4ac8-8e3e-b6d95c3b4d6f)

Another demonstrating automatic command reformulation upon error:

![nl-sh](https://github.com/mikecvet/nl-sh/assets/275631/f852c67e-c04c-4b08-8ccd-355b8ef70c61)

## How does this work?

`nl-sh` acts like a (minimal) shell by wrapping the underlying shell with a prompt which accepts and executes text inputs from the user. Inputs which seem like actual POSIX commands are directly executed; otherwise the input is directed to an LLM to interpret the input and provide a command sequence satisfying the request, customized for the local system powering the shell. This shell collects some environmental data such as kernel and OS version details to try and generate the most accurate command-line sequence for the given POSIX variant.

Any commands suggested by the backing LLM and then executed through `nl-sh` are written to the user's underlying shell command-history file.

## TODO

 - [ ] Respect underlying shell color configurations for `ls` and related outputs
 - [ ] Figure out how to support output-rewriting for commands such as `top`
 - [ ] Build reverse-incremental history search (for example, `cmd-r`)
 - [ ] Support pagination of lengthy outputs (ie piping through `more` or eqiuvalent
