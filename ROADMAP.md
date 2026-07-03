# GoPlus Roadmap

This file is the **single source of truth** for project status and direction.
README and other docs must not restate feature status; they link here instead,
so the two can never drift apart again.

Status labels:

- **Done**: implemented, tested, and expected to remain stable.
- **Partial**: usable foundation exists, but the feature is not complete (see the note for the exact gap).
- **Planned**: not implemented yet.

> This table was last reconciled against the source code on 2026-07-03. When a
> status changes, update it here in the same change that touches the code.

## Current State

GoPlus is an early compiler/transpiler for `.gp -> .go`. The frontend
(lexer/parser/sema/codegen) and a package/project orchestration layer are in
place, with a growing fixture suite. Most v2 tooling **exists and is wired to the
CLI**. `for`, `switch`, and `select` bodies are now structured, so nested
`match`/`if` are analyzed and desugared at any depth; only `defer` / `go` /
assignment remain line-level raw captures (they contain no nested blocks).

The rewriting formatter now **preserves comments** and is round-trip/idempotency
tested on every example; it refuses to overwrite a file only if a comment cannot
be safely reattached (so it can never silently delete one).

| Area | Status | Notes |
| --- | --- | --- |
| Compiler module split | Done | Core modules live under `src/parser`, `src/sema`, `src/codegen`, and `src/compiler`. |
| CLI `check`, `transpile`, `build`, `run`, `test` | Done | Existing workflows are preserved. |
| Diagnostics codes/source excerpts | Done | Human and JSON diagnostics include stable codes, source excerpts, spans, and hints for parser, decorator, match, and package errors. |
| Parser coverage | Done | Structured: imports, `struct`/`enum`/`impl`/`fn`, `if`/`else`, `match`, `for`, `switch`, `select`, `return`, `var :=`. `defer`/`go`/assignment remain line-level raw captures (no nested blocks to analyze). |
| Semantic checks | Done | Duplicate declarations, enum variant/name collisions, generated-name collisions, match arity/duplicates, and decorator checks — but only outside raw statement bodies. |
| Source maps + navigation | Done | `--emit-source-map` writes JSON `.gp`↔`.go` ranges; `goplus navigate` resolves both directions. |
| Formatter | Done | `fmt --check`, in-place `fmt`, and `fmt --stdout` rebuild from the AST and **reattach comments**. In-place `fmt` overwrites only when every comment is preserved (multiset check), else it refuses. Round-trip/idempotency + comment-preservation are tested over all examples. |
| Linter | Done | `goplus lint` ships 8 rules with stable codes; runs on the analyzed AST without generating Go. Match-based rules recurse into structured for/switch/select bodies. |
| CI example coverage | Done | CI runs `goplus check` on every `.gp` example, runs executable examples, and builds selected example packages. |

## v1.x Stabilization

Goal: make the current language reliable, predictable, well-tested, and
performance-conscious without breaking existing `.gp` syntax.

| Task | Status | Acceptance Criteria |
| --- | --- | --- |
| Keep current syntax backward-compatible | Done | Existing examples and tests keep passing. |
| CI checks every example source | Done | GitHub Actions runs `goplus check` on every `examples/**/*.gp` outside generated output. |
| CI builds representative example packages | Done | CI builds `examples/link-source` and `examples/complex-app`. |
| Add complex integration examples | Done | `examples/complex-app` covers grouped imports, block comments, derive, impl, tagged enums, match, decorators, memoize, error sugar, and raw Go-like statements. |
| Expand diagnostic precision | Done | Focused spans, stable codes, and hints cover parser recovery, decorator errors, match errors, and package/module errors. |
| Complete real source map mappings | Done | `--emit-source-map` records useful `.gp` to generated `.go` ranges for functions, declarations, match arms, and statements. |
| Add fixture/golden test matrix | Done | Fixtures cover parser, diagnostics, generated Go, tagged enum edge cases, source maps, build interop, and formatter round-trip/idempotency + comment preservation over all examples. |
| Structure `for` bodies | Done | `for` parses a raw header + a structured body block; nested `match`/`if` are analyzed and desugared, source maps reach inside, and the formatter re-indents the body. Fixtures + a runnable `examples/for_match_state.gp` cover it. |
| Structure `switch` bodies | Done | `switch` parses a raw header + structured `case`/`default` clauses; nested `match`/`if` are analyzed and desugared, source maps reach inside, and the formatter re-indents the clauses. |
| Structure `select` bodies | Done | `select` parses structured comm-clause cases (`case v := <-ch:`); clause parsing is shared with `switch`, nested `match`/`if` are analyzed, and the formatter re-indents the clauses. |
| Structure assignments / `defer` / `go` | Planned | Currently raw line/`RawStmt` captures with no LHS/RHS breakdown. |
| Make generic tagged enums build-clean | Done | Constructors and type references emit Go type arguments with `[...]`, not GoPlus `<...>` syntax. |
| Improve generated Go robustness | Done | Generated Go is built in more fixtures, enum/generated-name collisions are caught earlier, and output stays gofmt-readable. |
| Improve transpile performance | Done | Benchmarks exist and unchanged packages are skipped using a content-hash package cache manifest. |

## v2 Tooling And Devex

Goal: production-grade tooling on top of the stabilized compiler frontend. The
foundations exist and are CLI-wired; the remaining work is comment-safety,
structured control flow, and test coverage.

| Task | Status | Acceptance Criteria |
| --- | --- | --- |
| Formatter command shape | Done | `fmt --check`, in-place `fmt`, and `fmt --stdout` all exist. |
| Comment-safe rewriting formatter | Done | The formatter reattaches comments (leading, trailing, verbatim in raw regions) by scanning the source and placing them against AST node spans; it verifies the comment multiset survived before overwriting. Golden idempotency (`fmt(fmt(x)) == fmt(x)`) + comment-preservation are tested over all examples. |
| Linter | Done | `goplus lint` reports style/suspicious-code diagnostics without generating Go. Rules: `L0001` unused imports, `L0002` naming, `L0003` empty body, `L0004` redundant return, `L0005` unreachable arm after `_`, `L0006` large functions, `L0007` missing `@derive(String)`, `L0008` `_` hides unlisted enum variants. |
| Linter rule expansion | Done | Added `L0005` (unreachable match arm after a `_` wildcard) and `L0008` (`_` on an enum match that hides unlisted variants, defeating exhaustiveness). The earlier "misordered decorators" and "`?` outside error-capable fn" ideas were dropped as not lint-worthy: `@memoize`+`@retry` is already a hard error (conflicting return-type requirements), and `?`-context is already a semantic error caught before lint runs. |
| Lint CI gate | Planned | Run `goplus lint` in CI once the example suite is warning-clean (a hard gate today would fail on pre-existing warnings). |
| IDE diagnostics | Done | JSON diagnostics carry severity (`Error`/`Warning`/`Info`), codes, caret spans, and hints via `goplus check --diagnostic-format json`. |
| Source navigation | Done | `goplus navigate` and emitted source maps resolve `.gp`↔`.go` in both directions. |
| Built-in derives (`String`, `Debug`, `Equal`, JSON) | Done | All five (`String`, `Debug`, `Equal`, `JsonMarshal`, `JsonUnmarshal`) are generated for structs and enums, including tagged enums. |
| Richer derives (`Clone`, `Ord`, `Hash`) | Partial | `@derive(Clone)` deep-copies slice/map fields on structs (avoiding Go's silent shared-reference copy) and value-copies enums. `Ord`/`Hash` are not implemented yet. |
| User-defined derives | Planned | Analogous to user-defined decorators; no mechanism today. |
| Decorator signature validation (functions) | Partial | Local top-level decorators get `next`-param-count + return-shape checks (heuristic, string-based), beyond basic arity. Package-qualified (`pkg.dec`) decorators are skipped. |
| Decorator signature validation (methods) | Done | Custom decorators on methods are validated against known top-level decorator functions (arity + `next`-function shape), the same as on free functions; `analyze_method` now receives the function table instead of an empty map. |
| Package graph improvements | Partial | Topological build (Kahn) with cycle detection and content-hash caching exist; large-project discovery/caching can improve further. |

## v3 VSCode Extension

Goal: a first-party VSCode extension so `.gp` development feels native, leveraging
v2 tooling under the hood. Packaged at `editors/vscode/` (source `package.json`
version `0.2.0`; prefer the `goplus-lang-0.2.0.vsix` artifact — `0.1.0` is stale).

| Task | Status | Acceptance Criteria |
| --- | --- | --- |
| Syntax highlighting | Done | TextMate grammar covers keywords, types, strings, comments, decorators, enum variants. |
| Inline diagnostics | Done | `goplus check`/`lint` errors/warnings render as squiggles with severity, code, hint. |
| Lint on save | Done | `goplus lint` runs on save and populates the Problems panel. |
| Variable type inlay hints | Done | Inferred variable types render as inlay hints (0.2.0). |
| Strict Go type validation on check | Done | `goplus check` runs `go build` on generated output to surface real type errors (0.2.0). |
| Go/reverse source navigation | Partial | `goplus.navigateToGo` / `goplus.navigateToGp` commands work via source maps, but are **command-palette only** — there is no click/Ctrl+click Definition gesture wired to source-map navigation yet. |
| Format on save | Partial | Works via VSCode `editor.formatOnSave` + the formatting provider. The extension's own `goplus.formatOnSave` setting is currently **not read** (dead setting) and must be implemented or removed. |
| Snippet support | Done | Snippets for `fn`, `struct`, `enum`, `match`, `impl`, `@derive`, common decorators. |
| Hover information | Done | Hover on enum variants, decorators, and `?`/`!` shows contextual docs. |
| Extension marketplace | Partial | Packaged as `.vsix` with README, icon, changelog; marketplace publishing pending a publisher account. |

## Benchmarks & Evidence

Goal: prove GoPlus's value claims with a few reproducible numbers rather than
broad benchmarking.

| Task | Status | Acceptance Criteria |
| --- | --- | --- |
| Pipeline microbench | Done | `benches/pipeline.rs` measures parse/analyze/codegen on a sample. |
| Token-economy benchmark | Planned | Measure LLM tokens for equivalent logic in hand-written Go vs `.gp` across representative patterns. |
| Runtime-parity benchmark | Planned | Show generated Go performs the same as hand-written Go (no runtime layer). |
| Incremental/cache benchmark | Planned | Show the content-hash cache skips unchanged packages on rebuild. |

## Documentation

| Task | Status | Acceptance Criteria |
| --- | --- | --- |
| Single-source status (this file) | Done | README/docs link here instead of restating status. |
| Docs site (mdBook + GitHub Pages) | Planned | Published at `https://trinq2003.github.io/golang-plus/`, built in CI on merge to `main`. |
| Doc-consistency CI check | Planned | CI fails if README restates a status that contradicts this table. |

## Contribution Priorities

Recommended order (highest value / highest risk first):

1. ~~Comment-safe formatter~~ — **done**: the formatter reattaches comments and
   is round-trip/idempotency tested; in-place `fmt` refuses rather than drop a
   comment.
2. **Structure `for`/`switch`/`select` bodies** — so nested `match`/`if` are
   analyzed and desugared, and source maps reach inside them.
3. Expand linter rules on the stabilized AST.
4. Fix method-decorator validation and add richer derives (`Clone`, `Ord`).
5. Docs site + benchmarks to make the project legible and credible to outsiders.

For implementation work, start with:

- Parser/frontend: `src/parser/`, `src/lexer.rs`, `src/ast.rs`
- Semantics: `src/sema/`
- Generated Go: `src/codegen/`
- Project loading, CI-facing behavior, and Go toolchain integration: `src/compiler/`
