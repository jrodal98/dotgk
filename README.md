# dotgk

dotgk is a tool for evaluating gatekeepers - sets of conditions that determine
whether features should be enabled. Gatekeepers are defined using Lua scripts
and can include conditions like file existence, hostname matching, OS detection, and more.

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

# Sync all gatekeepers (i.e. regenerate caches)
dotgk sync

# Enable cache format generation
dotgk cache enable shell
```

Run `dotgk --help` or `dotgk <command> --help` for detailed options and usage.

## Integrations

### Shell

Run this command (only required once)
```shell
dotgk cache enable shell
```

Then, use in shell config like so:

```shell
source ~/.config/dotgk/caches/dotgk.sh
if dotgk_check "<gatekeeper name>"; then
  <logic here>
fi
```

### Lua

Run this command (only required once)
```shell
dotgk cache enable lua
```

Then, use it neovim like this:

```lua
-- do this part at the top of your init.lua (only required once)
package.path = package.path .. ";" .. vim.fn.expand "~/.config/dotgk/caches/?.lua"

local dotgk = require "dotgk"
if dotgk.check "<gatekeeper name>" then
   <logic here>
end
```

I haven't looked beyond neovim yet, but I'll update this once I play around with my wezterm config.


## Configuration

Gatekeepers are defined as Lua scripts in `~/.config/dotgk/gatekeepers/`. Each gatekeeper should return a boolean value.

### Simple Examples

**Check if a file exists:**
```lua
return file_exists("/etc/devserver.owners")
```

**Check operating system:**
```lua
return os("linux")
```

**Check hostname:**
```lua
return hostname("my-laptop")
```

### Combining Conditions

Use `any`, `all`, or `none` combinators for complex logic:

**Any (OR logic):**
```lua
return any({
  file_exists("/var/chef/outputs/cpe_info.json"),
  file_exists("C:/chef/outputs/cpe_info.json"),
  file_exists("/mnt/c/chef/outputs/cpe_info.json"),
})
```

**All (AND logic):**
```lua
return all({
  os("linux"),
  file_exists("/etc/debian_version"),
})
```

**None (NOR logic):**
```lua
return none({
  os("windows"),
  file_exists("/windows/system32"),
})
```

### Composing Gatekeepers

Reference other gatekeepers using Lua's native `require()`:

```lua
return any({
  require("meta.devserver"),
  hostname("iris"),
})
```

**Note:** Use dot notation for module names:
- File: `~/.config/dotgk/gatekeepers/meta/devserver.lua`
- Require: `require("meta.devserver")` (dot notation)
- CLI: `dotgk get meta/devserver` (slash notation still works)

### Directory Aggregates with init.lua

Directories can have an `init.lua` file that acts as the default module, following standard Lua convention.

**Use `dir()` to automatically aggregate all files in a directory:**

```lua
-- meta/init.lua (automatic aggregation)
return any(dir())         -- Loads all files in meta/ (excluding init.lua itself)
```

This is equivalent to manually listing all files:
```lua
return any({
  require("meta.devserver"),
  require("meta.laptop"),
  require("meta.linux"),
  require("meta.mac"),
  require("meta.windows"),
  require("meta.wsl"),
})
```

**Reference other directories:**
```lua
-- Complex aggregation
return all({
  any(dir()),             -- At least one meta/* is true
  any(dir("os")),         -- AND at least one os/* is true
})
```

Now you can use the directory name directly:
```lua
require("meta")              -- Loads meta/init.lua
require("meta.devserver")    -- Loads meta/devserver.lua

-- Example in server.lua:
return any({
  require("meta"),           -- Clean! Loads the aggregate
  hostname("iris"),
})
```

### Using Variables

Lua's full power is available:

```lua
local is_meta_laptop = any({
  file_exists("/var/chef/outputs/cpe_info.json"),
  file_exists("C:/chef/outputs/cpe_info.json"),
})

local is_personal = hostname("home-desktop")

return is_meta_laptop or is_personal
```

### TTL (Cache Time-To-Live)

Specify cache TTL in seconds using a comment:

```lua
-- ttl: 3600
return file_exists("/tmp/cache")
```

## Available Functions

- `file_exists(path: string) -> bool` - Check if a file exists
- `hostname(target: string) -> bool` - Match against system hostname
- `os(name: string) -> bool` - Check operating system ("linux", "macos", "windows", "unix")
- `require(name: string) -> bool` - Load another gatekeeper (standard Lua, use dot notation)
- `dir(path?: string) -> table<bool>` - Load all gatekeepers in a directory (defaults to current dir in init.lua)
- `any(checks: table) -> bool` - OR logic (at least one must be true)
- `all(checks: table) -> bool` - AND logic (all must be true)
- `none(checks: table) -> bool` - NOR logic (all must be false)

## Examples

See the `examples` directory for sample configurations demonstrating different
evaluator types and conditions.
