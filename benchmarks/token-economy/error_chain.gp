package main

import "strings"

fn readStep(s: string) -> string! {
    if len(s) == 0 {
        return error("empty input")
    }
    return strings.TrimSpace(s)
}

fn pipeline(input: string) -> string! {
    a := readStep(input)?
    b := readStep(a)?
    c := readStep(b)?
    return strings.ToUpper(c)
}
