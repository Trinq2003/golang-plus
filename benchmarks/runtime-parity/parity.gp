package main

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

fn weight(s: Sign) -> int {
    match s {
        Neg => -1
        Zero => 0
        Pos => 1
    }
}

fn scoreAll(xs: []int) -> int {
    total := 0
    for _, x := range xs {
        total = total + weight(classify(x))
    }
    return total
}
