# 🍳eggsecutor 🥚

A friendly file based stateful daemon tracker.

# Table of contents

- [Installation](#installation)
- [Usage](#usage)
    - [Common usage example](#common-usage-example)
- [Customization](#customization)

# Installation

- Install from [crates.io](https://crates.io/crates/eggsecutor)

    - `cargo install eggsecutor`

- Build manually from source
    ```sh
    $ git clone https://github.com/astherath/eggsecutor
    $ cd eggsecutor
    $ cargo install --path=.
    ```

# Usage

`eggsecutor` works best when launching single-file binaries that are meant to run as background processes.

```
eggsecutor 1.0

astherath <me@felipearce.dev>

A friendly background process task manager

USAGE:
    eggsecutor [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    clear    stops all of the processes being tracked and clears the tracking list
    hatch    start managing a binary process
    help     Print this message or the help of the given subcommand(s)
    list     list all managed processes
    stop     stop a process by name or pid
```

## Common usage example

A simple example that should run as-is to showcase the main usage loop (*flask and python3 required*)

```sh
# create a simple flask server daemon in a file named "FLASK_SERVER"
cat << EOT >> FLASK_SERVER
#!/usr/bin/python3
from flask import Flask

app = Flask(__name__)

@app.route("/")
def index():
  return {"status": 200}

if __name__ == "__main__":
  app.run()
EOT

# make the file executable
chmod +x FLASK_SERVER

# start the process from an executable file
eggsecutor hatch FLASK_SERVER
> Hatching process "FLASK_SERVER" and starting to track...
> egg hatched, tracking process with pid: "3670"

# check the process is healthy
eggsecutor list
> Process name    pid     status
> -----------------------------------
> FLASK_SERVER    3670    Running

# once ready shut down the server by name (or pid)
# the following are equivalent
eggsecutor stop FLASK_SERVER
eggsecutor stop 3670
> stopping process with pid: 3670

# or, if you want to stop ALL running processes being tracked
eggsecutor clear
```

# Customization

By design, `eggsecutor` is meant to be a low-maintenance (and therefore, low-option) tool.

The only user-defined variable is the location of the JSON state tracking file, which defaults to

`~/.eggsecutor.state`



If need be, this path can be overwritten by setting the `EGGSECUTOR_STATE_FILE` environment variable to a valid file path (if the path does not exist, it will be created upon first usage).
