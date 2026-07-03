package main

import (
	"errors"
	"strings"
)

func readStep(s string) (string, error) {
	if len(s) == 0 {
		return "", errors.New("empty input")
	}
	return strings.TrimSpace(s), nil
}

func pipeline(input string) (string, error) {
	a, err := readStep(input)
	if err != nil {
		return "", err
	}
	b, err := readStep(a)
	if err != nil {
		return "", err
	}
	c, err := readStep(b)
	if err != nil {
		return "", err
	}
	return strings.ToUpper(c), nil
}
