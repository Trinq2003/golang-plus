# Case study: a small REST API in GoPlus

A task-tracker REST service written entirely in `.gp` — enough of a real service
(HTTP layer, service layer, a store, a state machine, JSON) to test the
"readable, debuggable, review-friendly" promise on something past a 30-line
example.

```bash
goplus run  examples/rest-api-demo/main.gp    # runs a self-contained demo, then exits
goplus test examples/rest-api-demo/main.gp    # runs the HTTP handler tests (httptest)
goplus run  examples/rest-api-demo/main.gp -- serve   # starts the server on :8080
```

## What it exercises

| GoPlus feature | Where |
| --- | --- |
| `enum` + exhaustive `match` state machine | `Status`, `canTransition` — add a state and it stops compiling until you handle it |
| Error sugar `?` across layers | `transition` / `runDemo` read top-to-bottom, no `if err != nil` ladders |
| `@log` decorator | wraps `transition` with enter/exit/error logging, zero call-site noise |
| `@derive(JsonMarshal, JsonUnmarshal)` on struct **and** enum | `Task` and `Status` (de)serialize as clean JSON; `Status` as its name |
| `@derive(Clone)` | `loadTask` returns an independent copy, so callers can't mutate the store |
| Standard Go interop | `net/http`, `encoding/json`, `sync`, struct tags — all used directly |

The store is an in-memory map behind `insertTask` / `loadTask` / `storeTask`; a
real database driver slots in at exactly that boundary without touching the
handlers or service.

## Side by side — the `transition` service function

**GoPlus** (what you write and review):

```gp
@log
fn transition(id: int, to: Status) -> Task! {
    task := loadTask(id)?
    if !canTransition(task.State, to) {
        return Task{}, errors.New(fmt.Sprintf("cannot move from %s to %s", task.State, to))
    }
    task.State = to
    storeTask(task)
    return task
}
```

**The Go it stands in for** (what you'd otherwise write and review):

```go
func transition(id int, to Status) (Task, error) {
	fmt.Printf("[log] enter transition\n")
	task, err := loadTask(id)
	if err != nil {
		fmt.Printf("[log] error transition: %v\n", err)
		return Task{}, err
	}
	if !canTransition(task.State, to) {
		e := errors.New(fmt.Sprintf("cannot move from %s to %s", task.State, to))
		fmt.Printf("[log] error transition: %v\n", e)
		return Task{}, e
	}
	task.State = to
	storeTask(task)
	fmt.Printf("[log] exit transition\n")
	return task, nil
}
```

Same behaviour; the GoPlus version is roughly half the lines and keeps the
logging and error plumbing out of the reader's way — while the generated Go
above stays fully transparent and debuggable (this is exactly what
`goplus transpile` emits).

## It earned its keep

Writing this medium-sized service surfaced (and fixed) two real codegen bugs
that the single-file examples never hit:

1. `@derive(JsonMarshal/JsonUnmarshal)` on an **enum** emitted `json.Errorf`,
   which does not exist — it should be `fmt.Errorf`.
2. The JSON derives ignored explicit `` `json:"..."` `` struct tags and mangled
   field names (e.g. `ID` → `i_d`); they now honor the field's tag.

Both are covered by regression tests in `src/codegen/tests.rs`.
