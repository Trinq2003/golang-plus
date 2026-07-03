<p align="center">
  <img src="assets/logo.png" alt="GoPlus Logo" width="200" />
</p>

# goplus

`goplus` is a surface language for Go, implemented as a Rust transpiler (`*.gp -> *.go`).

The project goal is to keep the full Go ecosystem (toolchain, runtime, packages) while improving developer ergonomics: cleaner error flow, better enum/match support, and safe compile-time metaprogramming.

## Vision

`goplus` does not replace Go.

`goplus` is a productivity layer on top of Go that produces readable, debuggable, and review-friendly generated Go code.

Long-term vision:
- Become a practical language layer for real Go teams.
- Keep generated Go transparent to avoid lock-in.
- Build a strong contributor community around compiler, diagnostics, tooling, and language design.

## Core Goals

- Practical compatibility with Go toolchain.
- Go-like syntax with low learning curve (standard Go syntax is also supported).
- Concise error handling via `!` and `?`.
- Strong enum/match support, including exhaustive checks.
- Flexible compile-time decorators, including user-defined decorators.
- Human-readable generated code, always formatted with `gofmt`.

## Current Non-Goals

- Rust-style borrow checker.
- Free-form token-level macro system.
- Custom runtime replacing Go runtime.
- Overly complex type system that hurts simplicity.

## Compiler Architecture

- `lexer` -> `parser` -> `semantic` -> `codegen Go` -> `gofmt`.
- Semantic layer enforces key rules: decorator contracts, `?` context, exhaustive match.
- Codegen prioritizes readability and debuggability over micro-optimizations.

## Quick Start

First, download and install `goplus` from the [Releases](https://github.com/Trinq2003/golang-plus/releases) page (Windows users can use the `.msi` installer). Then run:

```bash
goplus check examples/demo.gp
goplus transpile examples/demo.gp --out-dir .goplusgen
goplus run examples/demo.gp --out-dir .goplusgen
goplus test examples/demo.gp --out-dir .goplusgen
```

`goplus` runs a standalone `.gp` path outside a Go module as the selected file.
If you point it at a directory, or at a file inside a Go module package, it will compile every `*.gp` file in that package and link any sibling `*.go` files.

Mixed-source example:

```bash
goplus run examples/link-source/main.gp --out-dir .goplusgen
```

For packages with Go tests, use `goplus test <file-or-dir> --out-dir .goplusgen`.
It transpiles `.gp` sources first, copies sibling `.go`/`*_test.go` files into the
generated package, then invokes `go test` against the generated Go package.

The `examples/link-source` sample now includes a real `go.mod` plus nested `pkg/...` and `internal/...` packages, so it shows both same-package linking and normal imported package boundaries.
Most of that example is written in `.gp`; it keeps only one `.go` bridge file to demonstrate GoPlus/Go interop explicitly.

## Short Example

```go
package main

import "fmt"

func trace(next func(name string) (string, error), label string) (func(name string) (string, error)) {
    return func(name string) (string, error) {
        fmt.Println("trace:", label)
        return next(name)
    }
}

@trace("custom")
func greet(name string) (string, error) {
    return "hello " + name, nil
}

func main() error {
    msg := greet("goplus")?
    fmt.Println(msg)
    return
}
```

Same example but with rust-style function type:
```gp
package main

import "fmt"

@derive(String)
enum Status {
    Pending
    Running
    Done
}

@log
@retry(3, 10)
fn readName() -> string! {
    return "goplus"
}

fn main() -> ! {
    name := readName()?
    fmt.Println(name)
    fmt.Println(Status::Running)
    return
}
```

## Custom Decorators

A custom decorator is a function that takes `next` (the previous function in the decorator chain) and returns a function with the same signature.

```gp
fn trace(next: func(path string) (string, error), label: string) -> func(path string) (string, error) {
    return func(path string) (string, error) {
        fmt.Println("trace:", label)
        return next(path)
    }
}

@trace("io")
fn load(path: string) -> string! {
    return "ok"
}
```

## VSCode Extension

GoPlus provides a native-like development experience in VSCode, powered by the `goplus` CLI.

### Features
- **Diagnostics**: Real-time error reporting with severity levels, precise spans, and hints.
- **Hover Information**: Documentation and signatures for keywords, variables, parameters, and decorators.
- **Auto-completion**: Suggestions for keywords, built-in decorators, enum variants, and derive options.
- **Formatting**: Automatic code formatting using `goplus fmt`.
- **Linting**: Code quality checks using `goplus lint`.
- **Go to Definition**: Bidirectional source navigation (`.gp` ↔ `.go`) powered by source maps.
- **Snippets**: Quick templates for `fn`, `struct`, `enum`, and decorators.

### Installation

The extension requires the `goplus` CLI to be available in your system `$PATH` (or `%PATH%` on Windows).

1. **Install CLI**:
   - **Windows**: Download the `.msi` installer from the [Releases](https://github.com/Trinq2003/golang-plus/releases) page and run it. The installer will automatically add `goplus` to your system `$PATH`!
   - **Other OS / From Source**:
     ```bash
     cargo install --path .
     ```

2. **Install Extension**:
   - Navigate to the `editors/vscode` directory.
   - Run `npm install` and `npm run package` (requires `vsce`) to build the extension, or use the pre-built `.vsix` file in the directory.
   - Install the generated `.vsix` file in VSCode (`Extensions: Install from VSIX...` from the command palette).

## Language & CLI Features

- Syntax: `fn` (and standard Go `func`), `struct`, `enum` (simple + tagged generic), `impl`.
- Error sugar: `-> T!`, `-> !`, `expr?`.
- `match` with enum exhaustive checking.
- `@derive(String, Debug, Equal, Clone, JsonMarshal, JsonUnmarshal)` for struct/enum (`Clone` deep-copies slice/map fields).
- Package compilation:
  - Standalone `.gp` files outside a Go module compile as the selected file.
  - Directories and Go module packages compile all sibling `.gp` files together.
  - Sibling `.go` files are copied into the generated package so Go and GoPlus code can link together.
- Compile-time decorators:
  - Built-in: `@log`, `@retry(times[, backoff_ms])`, `@memoize`.
  - Custom decorators (Python-like factory style: `next -> wrapped`).
- CLI:
  - `goplus check`
  - `goplus transpile`
  - `goplus build`
  - `goplus run`
  - `goplus test`
  - `goplus fmt` (and `goplus fmt --check`)
  - `goplus lint`
  - `goplus navigate`

## v2 Tooling & DevEx

The compiler is organized into scalable frontend, semantic, codegen, and
project orchestration submodules under `src/parser`, `src/sema`, `src/codegen`,
and `src/compiler`.

Tooling that exists today (see [ROADMAP.md](ROADMAP.md) for exact status and known gaps):
- **IDE Diagnostics**: severity levels (`Error`, `Warning`, `Info`), codes, precise caret spans, and hints via `goplus check --diagnostic-format json`.
- **Formatter**: `goplus fmt --check`, in-place `goplus fmt`, and `goplus fmt --stdout`. The rewriting formatter **preserves comments** (it reattaches them by span) and is round-trip/idempotency tested on every example; in-place mode overwrites only when every comment is safely preserved, otherwise it refuses (so it never silently deletes a comment).
- **Linter**: `goplus lint` reports stylistic/suspicious-code rules on the analyzed AST without compiling to Go.
- **Source Navigation**: `goplus navigate` and emitted source maps resolve `.gp` ↔ `.go` in both directions.
- **Topological Builds**: the package graph uses Kahn's algorithm to order transpilation and catch cycles early, with content-hash caching of unchanged packages.
- **Decorator Validation**: custom decorators referencing a local top-level function are checked for arity and `next` param-count/return-shape compatibility (heuristic), on both free functions and methods. Package-qualified decorators are not signature-checked.

## Roadmap & Status

[ROADMAP.md](ROADMAP.md) is the **single source of truth** for per-feature status.
This README intentionally does not restate feature status, so the two cannot drift.

At a glance: the compiler frontend, package build graph, linter, source maps,
built-in derives (`String`, `Debug`, `Equal`, JSON), and a comment-preserving
formatter are in place. The main remaining frontend gap is **structured parsing
of `for`/`switch`/`select` bodies** (they are raw pass-through today, so
`match`/`if` nested inside them are not analyzed). The
VSCode extension (`editors/vscode/`, `goplus-lang-0.2.0.vsix`) ships syntax
highlighting, diagnostics, lint-on-save, inlay hints, snippets, hover, and
command-based `.gp` ↔ `.go` navigation.