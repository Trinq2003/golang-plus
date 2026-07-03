# GoPlus

`goplus` is a surface language for Go, implemented as a Rust transpiler
(`*.gp -> *.go`). It keeps the full Go ecosystem — toolchain, runtime, packages —
while improving developer ergonomics: cleaner error flow, first-class enums and
exhaustive `match`, and safe compile-time decorators.

`goplus` **does not replace Go.** It is a productivity layer on top of Go that
produces readable, debuggable, review-friendly generated Go code, so you never
lose sight of what actually runs.

## Why it exists

- **Less boilerplate to write and review.** The same behaviour takes
  [~40–80% fewer tokens](benchmarks.md) in `.gp` than in hand-written Go.
- **No runtime cost.** The sugar desugars to plain Go — benchmarked at
  [zero allocations and zero overhead](benchmarks.md) versus hand-written Go.
- **The compiler checks more.** `match` on an enum is exhaustiveness-checked;
  `?` is only allowed where an error can be returned; decorators are validated
  against the functions they wrap.

## A taste

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

The status of every feature lives in one place — see
[Roadmap & status](roadmap.md).
