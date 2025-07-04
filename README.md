# dotgk

dotgk is a tool for evaluating gatekeepers - sets of conditions that determine
whether features should be enabled. Gatekeepers are defined using JSON
configuration files and can include conditions like file existence, hostname
matching, OS detection, and more.

## Installation

Binaries can be found in the Github release page. Alternatively, run this
command (at your own risk):

```sh
curl -fsSL https://raw.githubusercontent.com/jrodal98/dotgk/refs/heads/master/install.sh | sh
```

## Features

- Evaluate gatekeepers based on various conditions (file, hostname, OS, boolean,
  etc.)
- Cache evaluation results for improved performance
- Generate cache files in multiple formats (shell, lua, python) for integration
- Support for complex condition logic (equality, any, all, none)

## Basic Usage

```sh
# Evaluate a gatekeeper
dotgk evaluate my-feature

# Get cached result (or evaluate if missing/expired)
dotgk get my-feature

# Set a cached value
dotgk set my-feature true

# Sync all gatekeepers
dotgk sync

# Enable cache format generation
dotgk cache enable shell
```

Run `dotgk --help` or `dotgk <command> --help` for detailed options and usage.

## Configuration

Gatekeepers are defined in JSON files with a `groups` array. Each group has an
`evaluator` and `condition`:

```json
{
  "groups": [
    {
      "evaluator": {
        "type": "file",
        "args": {
          "path": "/home/user/some_file.txt"
        }
      },
      "condition": "eq"
    }
  ]
}
```

## Evaluator Types

- `bool`: Static boolean values
- `file`: File existence checks
- `hostname`: Hostname matching
- `os`: Operating system detection
- `gatekeeper`: Reference other gatekeepers

## Examples

See the `examples` directory for sample configurations demonstrating different
evaluator types and conditions.
