# DotGK README

## Introduction

DotGK is a gatekeeper system that allows you to define and evaluate conditions for various use cases. This README provides an overview of the project, its features, and how to use it.

## Features

* **Gatekeeper system**: Define and evaluate conditions using a simple JSON configuration file.
* **Cache support**: Cache evaluation results to improve performance and reduce the load on the system.
* **Flexible condition system**: Use various condition types, such as equality, inequality, any, all, and none, to define complex conditions.
* **Evaluator system**: Use different evaluators, such as hostname, file, gatekeeper, and OS, to evaluate conditions.
* **Command-line interface**: Use the `dotgk` command to evaluate gatekeepers, get cache values, set cache values, and sync cache.

## Usage

### Command-line interface

The `dotgk` command provides the following subcommands:

* `evaluate`: Evaluate a gatekeeper and print the result.
* `get`: Get a cache value for a gatekeeper.
* `set`: Set a cache value for a gatekeeper.
* `sync`: Sync the cache with the gatekeeper definitions.

### Gatekeeper configuration

Gatekeepers are defined in JSON files located in the `~/.config/dotgk` directory. Each file represents a single gatekeeper and contains the following fields:

* `groups`: A list of condition groups.
* `on_no_match`: A boolean indicating what to return if no condition matches.
* `ttl`: An optional TTL (time-to-live) value for cache entries.

### Condition groups

Condition groups are defined in the `groups` field of a gatekeeper configuration file. Each group contains the following fields:

* `evaluator`: An evaluator object that defines the condition to evaluate.
* `on_match`: A boolean indicating what to return if the condition matches.

### Evaluators

Evaluators are used to evaluate conditions. The following evaluators are supported:

* `HostnameEvaluator`: Evaluates a condition based on the hostname.
* `FileEvaluator`: Evaluates a condition based on the existence of a file.
* `GatekeeperEvaluator`: Evaluates a condition based on the result of another gatekeeper.
* `OSEvaluator`: Evaluates a condition based on the operating system.

## Examples

### Gatekeeper configuration file

```json
{
    "groups": [
        {
            "evaluator": {
                "type": "hostname",
                "args": {
                    "target": "example.com"
                }
            },
            "on_match": true
        }
    ],
    "on_no_match": false
}
```

### Evaluating a gatekeeper

```bash
dotgk evaluate example
```

### Getting a cache value

```bash
dotgk get example
```

### Setting a cache value

```bash
dotgk set example true
```

### Syncing the cache

```bash
dotgk sync
```

## Contributing

Contributions are welcome! Please submit a pull request with your changes and a brief description of what you've added or fixed.

## License

DotGK is licensed under the MIT License. See the LICENSE file for details.
