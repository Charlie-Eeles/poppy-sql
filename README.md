[![Crates.io](https://img.shields.io/crates/v/poppy-sql.svg)](https://crates.io/crates/poppy-sql)

## Poppy-sql
Very fast parser and formatter for standalone and embedded PostgreSQL

### What is this for?
Raw SQL doesn't only live in SQL files, it's often embedded in application code.

Poppy-sql is useful for:
- Finding all of the queries in a directory to return them as a parsed SQL result to be consumed as a library.
- Formatting all of the queries found in a directory with an easy to manage configuration as a binary.

### How to use?

#### As a formatter (binary)
Poppy-sql is available to install through cargo using: `cargo install poppy-sql`

Run `poppy-sql` in a directory with the files you want formatted in it, `poppy-sql -f {target_file}` to format a specific file, or `poppy-sql -f {target_file1} {target_file2}` to format multiple files.

You can ignore queries on a per-query basis by adding a comment anywhere in the query like: `-- poppy-ignore`

It can be integrated into your `.pre-commit-config.yaml` like:

```yaml
repos:
  - repo: https://github.com/starflower-sh/poppy-sql
    rev: v0.8.0
    hooks:
      - id: poppy-sql
```

You can set configuration by creating a `.poppy.toml` file.\
Reference the `default.poppy.toml` file for the expected configuration format.\
The default file will be used to populate values that aren't specified in your `.poppy.toml`.\
Poppy-sql will reference the nearest ancestor config file at or below the directory the format command is run, so you can have different config files apply in the same run of `poppy-sql`.\
Note: Formatting will _not_ run against filetypes not specified in the config even if directly targeted.

#### As a parser (library)
Poppy-sql is also available as a library for your rust projects where you can parse files using the parsing modules.

### What file types are supported?
Currently: ["sql", "py", "rs", "ts", "js", "mjs", "vue"]

Other file types will be ignored if Poppy-sql is run in the containing folder, or print an error message if specifically targeted.

Dotfiles and some misc directories are skipped (like node_modules), exact matches can be found in the constants module.

### Is it ready to use?
Poppy-sql is in active development, but you should be able to use its latest releases with a touch of caution.

Be aware that Poppy-sql follows semantic versioning to represent backwards incompatible changes and always double check the formatted results.


## Acknowledgments

The formatting module is based largely on the hard work done on the sqlformat-rs library and by extension sql-formatter-plus that it was based on.

https://github.com/shssoichiro/sqlformat-rs

https://github.com/kufii/sql-formatter-plus

There is a more in-depth acknowledgment in the formatting module.
