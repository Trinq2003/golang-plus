# CLI reference

All commands take a `.gp` file or a package directory.

| Command | What it does |
| --- | --- |
| `goplus check <src>` | Parse, semantically analyze, transpile, and run `go build` on the output to surface real type errors. Add `--diagnostic-format json` for editor-ready diagnostics. |
| `goplus transpile <src> --out-dir <dir>` | Emit Go into `<dir>`. Add `--emit-source-map` to also write a `.gp`↔`.go` source map. |
| `goplus build <src> --out-dir <dir> [--out <bin>]` | Transpile then `go build` a binary. |
| `goplus run <src> --out-dir <dir>` | Transpile then run. |
| `goplus test <src> --out-dir <dir>` | Transpile, copy sibling `*.go` / `*_test.go` into the generated package, then `go test`. |
| `goplus fmt <src>` | Format `.gp` in place. Comment-safe: it only overwrites when every comment is preserved. `--check` verifies formatting; `--stdout` previews without writing. |
| `goplus lint <src>` | Report style / suspicious-code diagnostics (`L0001`–`L0008`) without generating Go. |
| `goplus navigate --source-map <map> --file <f> --line <n> --column <c> [--reverse]` | Resolve a position between `.gp` and generated `.go` using a source map. |

## Diagnostics

Errors and warnings carry a stable code, a caret span, and a hint. Request JSON
with `--diagnostic-format json` for tooling:

```bash
goplus check examples/demo.gp --diagnostic-format json
```

Lint rules: `L0001` unused imports, `L0002` naming, `L0003` empty body, `L0004`
redundant return, `L0005` unreachable arm after `_`, `L0006` large functions,
`L0007` missing `@derive(String)`, `L0008` `_` hides unlisted enum variants.
