package main

type State int

const (
	Idle State = iota
	Running
	Paused
	Done
)

func (s State) String() string {
	switch s {
	case Idle:
		return "Idle"
	case Running:
		return "Running"
	case Paused:
		return "Paused"
	case Done:
		return "Done"
	default:
		return "State(?)"
	}
}

// next must be kept exhaustive by hand — Go's compiler does not check it.
func next(s State) State {
	switch s {
	case Idle:
		return Running
	case Running:
		return Paused
	case Paused:
		return Running
	case Done:
		return Done
	default:
		panic("unreachable")
	}
}
