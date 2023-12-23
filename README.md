## Usage
The CLI can be used with these options:
```bash
Usage: editor [OPTIONS] <ACTION>

Arguments:
  <ACTION>  [possible values: install, remove, update]

Options:
  -c, --config <CONFIG>  Provide config toml file configuration [default: config.toml]
  -t, --tools <TOOLS>    Specify the tools you want to modify
  -g, --groups <GROUPS>  Specify the groups you want to modify
  -s, --symbolic         Temporary install with symbolic names
  -f, --force            Force action
      --only-editor      Only make modifications on the editor
      --except-editor    except the editor configuration works
  -h, --help             Print help
```

## Configuration
Your configuration should be in the form of a [TOML](https://toml.io) file:

```toml
# Main element, your text editor
[editor]
name = "hx" # This will be the name of the installed editor
bin = "path/to/your/bin" # The executable
config = "path/to/your/config" # The configuration (Optional)
lib = "path/to/your/lib" # The library your executable may depends on (Optional)

# Secondary elements, your tools
[tools.first]
name = "first" # This will be the name of the installed executable
bin = "path/to/first/bin" # Required 
config = "path/to/first/config" # Optional
lib = "path/to/first/lib" # Optional
dependencies = ["second", ...] # Tools required for this tool to work (Optional)

[tools.second]
...

# Third and last elements, your groups
[groups.odd]
name = "odd" # The name of the group
dependencies = ["first", "third", ...] # Tools that will be installed when installing this group

[groups.even]
...
```
