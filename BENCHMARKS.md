# GoPlus Benchmarks

Three focused benchmarks back the project's core claims. They are deliberately
small and reproducible rather than exhaustive — enough to show the direction of
the numbers, not to publish a paper. Reproduce them with
[`benchmarks/run.sh`](benchmarks/run.sh); inputs live under [`benchmarks/`](benchmarks/).

Measured on Linux (the project's CI target), Go 1.26, Rust 1.96.

## A. Token economy — an agent writes far less code

The pitch: an LLM (or a human) writes the same behaviour in fewer tokens with
GoPlus, which means fewer chances to get boilerplate wrong and lower cost per
change. Each row is the **same behaviour** written twice: once in `.gp`, once in
plain hand-written Go. Token counts use the real GPT‑4-family tokenizer
(`tiktoken`, `cl100k_base`).

| Pattern | GoPlus `.gp` | Hand-written Go | Token reduction |
| --- | ---: | ---: | ---: |
| Error-propagation chain (`?` vs `if err != nil`) | 81 | 130 | **‑38 %** |
| Enum state machine + `String()` (`@derive`/`match` vs `iota`/`switch`) | 69 | 151 | **‑54 %** |
| Decorated API call (`@log`/`@retry` vs hand-rolled loop) | 40 | 188 | **‑79 %** |
| **Total** | **190** | **469** | **‑59 %** |

The win grows with the amount of boilerplate replaced: error plumbing saves a
third, exhaustive enums roughly half, and decorators (which stand in for whole
retry/logging wrappers) nearly 80 %. GoPlus also **checks** what it generates —
the `match` is exhaustiveness-checked, which the hand-written `switch` is not.

## B. Runtime parity — no overhead over hand-written Go

The pitch: GoPlus is a transpiler, not a runtime, so its sugar (`?`, `enum`,
`match`) must compile to Go that runs exactly as fast as code you would write by
hand. The same algorithm (classify + weight over 100 000 ints) is benchmarked as
GoPlus-generated Go vs a hand-written Go equivalent **in the same package**
(`go test -bench`, `-count=3`); a `TestParity` first asserts both produce
identical output.

| Implementation | ns/op (3 runs) | allocs/op |
| --- | ---: | ---: |
| GoPlus-generated Go | 124 030 · 127 226 · 125 932 | 0 |
| Hand-written Go | 128 724 · 128 744 · 118 633 | 0 |

The two are within run-to-run noise of each other, both with **zero
allocations** — GoPlus's `enum`/`match` desugars to a plain integer `switch`,
adding nothing at runtime. (Decorators like `@memoize`/`@retry` add exactly the
cost of the cache/loop they stand for — the same code you would otherwise write
by hand.)

## C. Transpile speed — fast, and cached

The pitch: GoPlus must not slow down a build/CI loop. The frontend is
microsecond-fast; codegen dominates but is still low-millisecond for a sample
program (`cargo bench --bench pipeline`).

| Stage | Time |
| --- | ---: |
| Parse | ~3.0 µs |
| Analyze (semantic) | ~6.1 µs |
| Codegen (Go text) | ~1.4 ms |

On top of that, a **content-hash package cache** (`.goplus-package-cache.json`,
written into the output dir) lets incremental builds skip packages whose sources
have not changed, so a warm rebuild does no redundant work.

## Caveats

- Small, illustrative inputs on a single machine; treat the magnitudes as
  indicative, not guaranteed.
- Token counts use `cl100k_base` as a representative modern LLM tokenizer; exact
  counts vary by model.
- Benchmark B isolates language *sugar* (no decorators) to measure overhead
  fairly; decorators intentionally add the work they describe.
