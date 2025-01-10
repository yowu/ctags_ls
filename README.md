
# ctags_ls

`ctags_ls` is a simple language server implementation for editors that do not natively support ctags.

> [!NOTE]
> This is not a full-featured LSP and only supports basic functionalities such as go-to definition, declaration, and implementation.

## Prerequisites

`ctags_ls` relies on `readtags` to read the tags file, so [Universal Ctags](https://github.com/universal-ctags/ctags) is required.

### Generating the Tags File

Run the following command in the root of your workspace:

```sh
ctags -R --fields=+K .
```

By default, the tags file should be named `tags` and placed in the root of the workspace. However, you can specify your tags files from the `initialization_options` configuration.

> [!NOTE]
> The `kind` field is **required** when generating the tags file.

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

