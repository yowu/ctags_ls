
# ctags_ls

`ctags_ls` is a simple language server implementation to use `ctags` in the editors that do not natively support it.

> [!NOTE]
> This is not a full-featured language server and only supports basic functionalities such as goto definition, declaration, and implementation so far.

## Prerequisites

`ctags_ls` relies on `readtags` to read the tags file, so [Universal Ctags](https://github.com/universal-ctags/ctags) is required.

### Generating the Tags File

You need to generate tags file for your project before using the `ctags_ls`. By default, the tags file should be named `tags` and placed in the root of the workspace. However, you can specify your tags files from the `initialization_options` configuration.


## Editor Configuration

### Helix Editor

To configure `ctags_ls` with Helix, add the following to your `languages.toml`:

```toml
[language-server.ctags_ls]
command = "path/to/ctags_ls"
config = { tags = ["tags", ".tags", "other tags file name"] }

[[language]]
name = "cpp"
language-servers = ["ctags_ls"]
```

### Zed Editor

To configure `ctags_ls` with Zed, add the following to your `settings.json` to hijack the clangd configuration:

```json
{
  "lsp": {
    "clangd": {
      "binary": {
        "path": "path/to/ctags_ls",
        "arguments": []
      },
      "initialization_options": {
        "tags": [
          "tags",
          ".tags",
          "other tags file name"
        ]
      }
    }
  }
}
```

