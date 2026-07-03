package main

import "testing"

// Hand-written Go equivalent of parity.gp, to benchmark side by side against the
// GoPlus-generated code in the same package (scoreAll/weight/classify).

type SignH int

const (
	NegH SignH = iota
	ZeroH
	PosH
)

func classifyH(n int) SignH {
	if n < 0 {
		return NegH
	}
	if n == 0 {
		return ZeroH
	}
	return PosH
}

func weightH(s SignH) int {
	switch s {
	case NegH:
		return -1
	case ZeroH:
		return 0
	case PosH:
		return 1
	default:
		return 0
	}
}

func scoreAllH(xs []int) int {
	total := 0
	for _, x := range xs {
		total += weightH(classifyH(x))
	}
	return total
}

var benchInput = func() []int {
	xs := make([]int, 100000)
	for i := range xs {
		xs[i] = i%7 - 3
	}
	return xs
}()

var sink int

// TestParity guards that both implementations agree before we benchmark them.
func TestParity(t *testing.T) {
	if got, want := scoreAll(benchInput), scoreAllH(benchInput); got != want {
		t.Fatalf("generated=%d hand=%d", got, want)
	}
}

func BenchmarkGoPlusGenerated(b *testing.B) {
	for i := 0; i < b.N; i++ {
		sink = scoreAll(benchInput)
	}
}

func BenchmarkHandWrittenGo(b *testing.B) {
	for i := 0; i < b.N; i++ {
		sink = scoreAllH(benchInput)
	}
}
