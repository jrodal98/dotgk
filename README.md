# dotgk

dotgk is a tool for evaluating gatekeepers, which are sets of conditions that determine whether a certain feature or functionality should be enabled. Gatekeepers can be defined using a JSON configuration file and can include conditions such as file existence, hostname matching, and more.

## Installation

Binaries can be found in the Github release page. Alternatively, run this command (at your own risk)

```sh
curl -fsSL https://raw.githubusercontent.com/jrodal98/dotgk/refs/heads/master/install.sh | sh
```

## Features

- Evaluate gatekeepers based on conditions such as file existence, hostname matching, and more
- Cache evaluation results to improve performance
- Support for multiple condition types, including equality, inequality, any, all, and none
- Support for multiple evaluator types, including file, hostname, OS, and gatekeeper

## Usage

dotgk can be used from the command line to evaluate gatekeepers and cache results. The following commands are available:

- `dotgk evaluate <name>`: Evaluate a gatekeeper and print the result
- `dotgk get <name>`: Get the cached result for a gatekeeper, evaluating if the cache is expired or missing
- `dotgk set <name> <value>`: Set a value in the cache for a gatekeeper
- `dotgk sync`: Sync all gatekeepers and cache results

## Configuration

Gatekeepers are defined using a JSON configuration file. The file should contain a `groups` array, where each group represents a set of conditions to evaluate. Each group should have an `evaluator` object, which specifies the type of evaluator to use and the arguments to pass to it. The `condition` field specifies the condition to apply to the evaluator result.

For example:

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

This configuration defines a gatekeeper with a single group that evaluates the existence of a file at `/home/user/some_file.txt`. If the file exists, the gatekeeper evaluates to `true`.

## Evaluators

dotgk supports the following evaluator types:

- `file`: Evaluates the existence of a file
- `hostname`: Evaluates the hostname of the system
- `os`: Evaluates the operating system of the system
- `gatekeeper`: Evaluates another gatekeeper

Each evaluator type has its own set of arguments and conditions that can be applied to it.

## Caching

dotgk caches evaluation results to improve performance. The cache is stored in a JSON file in the configuration directory. The cache can be synced manually using the `dotgk sync` command.

## Examples

The `examples` directory contains sample gatekeeper configurations that demonstrate the different features of dotgk.
