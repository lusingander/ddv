# DDV

[![Crate Status](https://img.shields.io/crates/v/ddv.svg)](https://crates.io/crates/ddv)

Terminal DynamoDB Viewer ⚡️

<img src="./img/demo.gif">

## About

DDV is a TUI application to view Amazon DynamoDB in the terminal.

> [!WARNING]
> This application is designed to be used in a local environment or in a development environment with a small amount of data. It is not suitable for use in a production environment with large amounts of data.

### Goals

- Provide a simple way to view, search, update, and delete DynamoDB items in the terminal.

### Non-Goals

- Efficiently handling large tables for querying or updating.
- Offering full support for all DynamoDB API operations.

## Installation

### [Cargo](https://crates.io/crates/ddv)

```
$ cargo install --locked ddv
```

### [Homebrew (macOS)](https://github.com/lusingander/homebrew-tap/blob/master/ddv.rb)

```
$ brew install lusingander/tap/ddv
```

### Downloading binary

You can download pre-compiled binaries from [releases](https://github.com/lusingander/ddv/releases).

## Usage

After installation, run the following command:

```
$ ddv
```

Basically, you can use it in [the same way as the AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html).

In other words, if the default profile settings exist or [the environment variables are set](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-envvars.html), you do not need to specify any options.

### Options

```
DDV - Terminal DynamoDB Viewer ⚡️

Usage: ddv [OPTIONS]

Options:
  -r, --region <REGION>     AWS region
  -e, --endpoint-url <URL>  AWS endpoint url
  -p, --profile <NAME>      AWS profile name
  -h, --help                Print help
  -V, --version             Print version
```

### Keybindings

The basic key bindings are as follows:

| Key                                   | Description          |
| ------------------------------------- | -------------------- |
| <kbd>Ctrl-C</kbd>                     | Quit app             |
| <kbd>Enter</kbd>                      | Open selected item   |
| <kbd>Backspace</kbd>                  | Go back to previous  |
| <kbd>j/k/h/l</kbd> <kbd>↓/↑/←/→</kbd> | Select item / Scroll |
| <kbd>?</kbd>                          | Show help            |

Detailed operations on each view can be displayed by pressing `?` key.

## Screenshots

<img src="./img/table-list-list.png" width=400> <img src="./img/table-list-detail-kv.png" width=400> <img src="./img/table-list-detail-json.png" width=400> <img src="./img/table.png" width=400> <img src="./img/table-expand-attr.png" width=400> <img src="./img/item-kv.png" width=400> <img src="./img/item-plain-json.png" width=400> <img src="./img/item-raw-json.png" width=400> <img src="./img/table-insight.png" width=400>

## Related projects

- [STU](https://github.com/lusingander/stu) - TUI explorer application for Amazon S3

## License

MIT
