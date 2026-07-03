package main

import (
	"fmt"
	"time"
)

func fetchInner(id int) (string, error) {
	return fmt.Sprintf("item-%d", id), nil
}

func fetchRetry(id int) (string, error) {
	var lastErr error
	for attempt := 0; attempt < 3; attempt++ {
		result, err := fetchInner(id)
		if err == nil {
			return result, nil
		}
		lastErr = err
		time.Sleep(time.Duration(10) * time.Millisecond)
	}
	return "", lastErr
}

func fetch(id int) (string, error) {
	fmt.Printf("[log] enter fetch\n")
	result, err := fetchRetry(id)
	if err != nil {
		fmt.Printf("[log] error fetch: %v\n", err)
		return "", err
	}
	fmt.Printf("[log] exit fetch\n")
	return result, nil
}
