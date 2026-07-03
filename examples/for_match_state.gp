package main

import "fmt"

// Demonstrates structured `for` parsing: the loop body is a real block, so the
// `match` nested inside it is exhaustiveness-checked and desugared to a Go
// switch (it is no longer opaque raw text).
@derive(String)
enum Sign {
    Neg
    Zero
    Pos
}

fn classify(n: int) -> Sign {
    if n < 0 {
        return Sign::Neg
    }
    if n == 0 {
        return Sign::Zero
    }
    return Sign::Pos
}

fn main() {
    // 3-clause header stays raw; the body is structured.
    for i := -1; i <= 1; i += 1 {
        s := classify(i)
        match s {
            Sign::Neg => fmt.Println(i, "negative")
            Sign::Zero => fmt.Println(i, "zero")
            Sign::Pos => fmt.Println(i, "positive")
        }
    }
}
