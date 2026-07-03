# Getting started

## Install

- **Windows**: download the `.msi` installer from the
  [Releases](https://github.com/Trinq2003/golang-plus/releases) page and run it —
  it adds `goplus` to your `PATH`.
- **From source** (any OS, requires Rust):

  ```bash
  cargo install --path .
  ```

You also need the [Go toolchain](https://go.dev/dl/) on your `PATH`, since
`goplus` builds and runs the Go it generates.

## First commands

```bash
goplus check     examples/demo.gp                       # type-check via the Go compiler
goplus transpile examples/demo.gp --out-dir .goplusgen  # emit Go
goplus run       examples/demo.gp --out-dir .goplusgen  # transpile + run
goplus test      examples/demo.gp --out-dir .goplusgen  # transpile + go test
goplus fmt       examples/demo.gp                        # format in place (comment-safe)
goplus lint      examples/demo.gp                        # style / suspicious-code checks
```

A standalone `.gp` file outside a Go module is compiled as the selected file. If
you point `goplus` at a directory, or a file inside a Go module package, it
compiles every `*.gp` file in that package and links any sibling `*.go` files —
so GoPlus and Go code interoperate directly.

See the full command list in the [CLI reference](cli.md), and a real, multi-file
service in [`examples/rest-api-demo`](https://github.com/Trinq2003/golang-plus/tree/main/examples/rest-api-demo).
