# Architecture

GoPlus is a straight-line transpiler pipeline:

```text
.gp source
   │  lexer      (src/lexer.rs)            tokens
   ▼
   │  parser     (src/parser/)             AST  (src/ast.rs)
   ▼
   │  semantic   (src/sema/)               validated AST + model
   ▼
   │  codegen    (src/codegen/)            Go source text
   ▼
   │  gofmt                                formatted .go
   ▼
Go toolchain (go build / go test / go run)
```

Each stage has a single job:

- **Lexer** — turns source into tokens (`logos`-based).
- **Parser** — builds the AST. Language constructs (`struct`, `enum`, `impl`,
  `fn`, `if`/`else`, `match`, `for`/`switch`/`select`, imports) are structured;
  a few line-level constructs (`defer`, `go`, assignments) are kept as raw
  pass-through text. Unfamiliar shapes fall back to raw text rather than failing.
- **Semantic layer** — enforces the rules that make GoPlus safe: exhaustive
  `match`, `?` only in error-capable functions, decorator contracts, and
  duplicate/collision checks. It recurses into every block, including the bodies
  of `for`/`switch`/`select`.
- **Codegen** — emits readable Go. Enums become integer or tagged-struct types,
  `match` becomes a `switch`, `?` becomes explicit `if err != nil`, decorators
  become named wrapper functions, and `@derive` emits `String`/`Debug`/`Equal`/
  `Clone`/JSON methods.
- **gofmt** — the generated Go is always run through `gofmt`, so it reads like
  code a person wrote.

Two cross-cutting facilities support tooling:

- **Source maps** (`--emit-source-map`) record `.gp`↔`.go` ranges for
  declarations, functions, match arms, and statements, powering `goplus
  navigate` and editor "go to definition".
- A **content-hash package cache** lets incremental builds skip packages whose
  sources have not changed.

The design goal throughout: keep the generated Go transparent, so there is no
lock-in and every line remains debuggable.
