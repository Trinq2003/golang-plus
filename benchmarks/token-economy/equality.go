package main

import "fmt"

type Point struct {
	X     int
	Y     int
	Label string
}

func (p Point) Debug() string {
	return fmt.Sprintf("Point{X:%+v, Y:%+v, Label:%+v}", p.X, p.Y, p.Label)
}

func (p Point) Equal(other Point) bool {
	return p.X == other.X && p.Y == other.Y && p.Label == other.Label
}
