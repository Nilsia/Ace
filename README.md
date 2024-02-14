## Usage
The CLI can be used with these options:
```bash
Usage: editor [OPTIONS] <ACTION>

Arguments:
  <ACTION>  [possible values: install, remove, update, list]

Options:
  -c, --config <CONFIG>  Provide config toml file configuration [default: config.toml]
  -t, --tools <TOOLS>    Specify the tools you want to modify
  -g, --groups <GROUPS>  Specify the groups you want to modify
  -s, --symbolic         Temporary install with symbolic names
  -f, --force            Force action
  -v, --verbose          Verbose mode
      --only-editor      show version Only make modifications on the editor
      --except-editor    except the editor configuration works
  -h, --help             Print help
  -V, --version          Print version
```

## Configuration
Your configuration should be in the form of a [TOML](https://toml.io) file:

```toml
# Main element, your text editor

default

[editor]
name = "hx"
bin = "path/to/your/bin"
config = "optional/path/to/your/config"
lib = "optional/path/to/your/lib"

# Secondary elements, your tools

[tools.first]
name = "first"
bin = "path/to/first/bin"
config = "optional/path/to/first/config"
lib = "optional/path/to/first/lib"
dependencies = ["second", "..."]

[tools.second]
name = "..."

# Third and last elements, your groups

[groups.odd]
name = "odd"
dependencies = ["first", "third", "..."]

[groups.even]
name = "..."
```
