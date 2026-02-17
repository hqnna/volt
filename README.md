# volt ![license] ![status]

[status]: https://img.shields.io/github/actions/workflow/status/hqnna/volt/release.yml?label=build&labelColor=4a414e
[license]: https://img.shields.io/github/license/hqnna/volt?labelColor=4a414e&color=3373cc

An ergonomic terminal settings editor for the [Amp](https://ampcode.com) coding agent.

## Installation

You can download pre-built binaries from the [releases] page.

[releases]: https://github.com/hqnna/volt/releases

## Build From Source

If you prefer to build things from source you can do so via Nix.

```console
$ nix build github:hqnna/volt#volt-static
```

Or if you would rather use local toolchains, you can use `cargo` instead.

```console
cargo install --git https://github.com/hqnna/volt
```
