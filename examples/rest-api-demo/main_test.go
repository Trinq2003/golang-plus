package main

import (
	"net/http/httptest"
	"strings"
	"testing"
)

func reset() {
	tasks = map[int]Task{}
	nextID = 1
}

func do(method, path, body string) *httptest.ResponseRecorder {
	req := httptest.NewRequest(method, path, strings.NewReader(body))
	rec := httptest.NewRecorder()
	if path == "/tasks" {
		createHandler(rec, req)
	} else {
		tasksByID(rec, req)
	}
	return rec
}

func TestCreateGetTransition(t *testing.T) {
	reset()

	r := do("POST", "/tasks", `{"title":"hello"}`)
	if r.Code != 201 || !strings.Contains(r.Body.String(), `"state":"Todo"`) {
		t.Fatalf("create: %d %s", r.Code, r.Body.String())
	}

	r = do("GET", "/tasks/1", "")
	if r.Code != 200 || !strings.Contains(r.Body.String(), `"id":1`) {
		t.Fatalf("get: %d %s", r.Code, r.Body.String())
	}

	// State machine rejects Todo -> Done.
	r = do("POST", "/tasks/1/transition", `{"to":"Done"}`)
	if r.Code != 409 {
		t.Fatalf("expected 409, got %d %s", r.Code, r.Body.String())
	}

	// Todo -> InProgress is allowed.
	r = do("POST", "/tasks/1/transition", `{"to":"InProgress"}`)
	if r.Code != 200 || !strings.Contains(r.Body.String(), `"state":"InProgress"`) {
		t.Fatalf("transition: %d %s", r.Code, r.Body.String())
	}
}
