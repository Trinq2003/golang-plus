package main

import (
    "encoding/json"
    "errors"
    "fmt"
    "net/http"
    "os"
    "strconv"
    "strings"
    "sync"
)

// --- Domain -----------------------------------------------------------------

// Status is a task's lifecycle state. @derive(String) gives it a readable name;
// the JSON derives (de)serialize it as that name, so the wire format is stable.
@derive(String, JsonMarshal, JsonUnmarshal)
enum Status {
    Todo
    InProgress
    Done
    Cancelled
}

// The allowed transitions are a state machine written with `match`, so the
// compiler forces every state to be handled: add a Status variant and this
// stops compiling until you decide its transitions.
fn canTransition(from: Status, to: Status) -> bool {
    match from {
        Todo => to == Status::InProgress || to == Status::Cancelled
        InProgress => to == Status::Done || to == Status::Cancelled
        Done => false
        Cancelled => false
    }
}

fn parseStatus(name: string) -> Status! {
    switch name {
    case "Todo":
        return Status::Todo
    case "InProgress":
        return Status::InProgress
    case "Done":
        return Status::Done
    case "Cancelled":
        return Status::Cancelled
    default:
        return Status::Todo, errors.New("unknown status: " + name)
    }
}

@derive(JsonMarshal, JsonUnmarshal, Clone)
struct Task {
    ID: int `json:"id"`
    Title: string `json:"title"`
    State: Status `json:"state"`
}

// --- Store (in-memory; a real DB slots in behind these functions) -----------

var mu sync.Mutex
var tasks = map[int]Task{}
var nextID = 1

fn insertTask(title: string) -> Task {
    mu.Lock()
    defer mu.Unlock()
    id := nextID
    nextID += 1
    task := Task{ID: id, Title: title, State: Status::Todo}
    tasks[id] = task
    return task
}

fn loadTask(id: int) -> Task! {
    mu.Lock()
    defer mu.Unlock()
    task, ok := tasks[id]
    if !ok {
        return Task{}, errors.New("task not found")
    }
    return task.Clone()
}

fn storeTask(task: Task) {
    mu.Lock()
    defer mu.Unlock()
    tasks[task.ID] = task
}

// --- Service ----------------------------------------------------------------

// transition validates the state machine, then persists. The error flow uses
// `?`, so the happy path reads top to bottom with no `if err != nil` ladders.
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

// --- HTTP handlers ----------------------------------------------------------

fn writeJSON(w: http.ResponseWriter, code: int, body: any) {
    w.Header().Set("Content-Type", "application/json")
    w.WriteHeader(code)
    json.NewEncoder(w).Encode(body)
}

fn createHandler(w: http.ResponseWriter, r: *http.Request) {
    var req Task
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        writeJSON(w, 400, map[string]string{"error": "invalid body"})
        return
    }
    writeJSON(w, 201, insertTask(req.Title))
}

fn getHandler(w: http.ResponseWriter, r: *http.Request, id: int) {
    task, err := loadTask(id)
    if err != nil {
        writeJSON(w, 404, map[string]string{"error": err.Error()})
        return
    }
    writeJSON(w, 200, task)
}

fn transitionHandler(w: http.ResponseWriter, r: *http.Request, id: int) {
    var body map[string]string
    json.NewDecoder(r.Body).Decode(&body)
    to, err := parseStatus(body["to"])
    if err != nil {
        writeJSON(w, 400, map[string]string{"error": err.Error()})
        return
    }
    task, err := transition(id, to)
    if err != nil {
        writeJSON(w, 409, map[string]string{"error": err.Error()})
        return
    }
    writeJSON(w, 200, task)
}

// tasksByID routes /tasks/{id} and /tasks/{id}/transition.
fn tasksByID(w: http.ResponseWriter, r: *http.Request) {
    rest := strings.TrimPrefix(r.URL.Path, "/tasks/")
    parts := strings.Split(rest, "/")
    id, err := strconv.Atoi(parts[0])
    if err != nil {
        writeJSON(w, 400, map[string]string{"error": "bad id"})
        return
    }
    if len(parts) > 1 && parts[1] == "transition" {
        transitionHandler(w, r, id)
        return
    }
    getHandler(w, r, id)
}

fn serve(addr: string) -> ! {
    http.HandleFunc("/tasks", createHandler)
    http.HandleFunc("/tasks/", tasksByID)
    fmt.Println("listening on", addr)
    return http.ListenAndServe(addr, nil)
}

// --- Demo (runs on `goplus run`, then exits so it is CI-safe) ---------------

fn runDemo() -> ! {
    a := insertTask("write docs")
    b := insertTask("ship release")
    fmt.Println("created:", a.State, b.State)

    moved := transition(a.ID, Status::InProgress)?
    done := transition(moved.ID, Status::Done)?
    fmt.Println("moved:", done.State)

    _, rejected := transition(b.ID, Status::Done)
    if rejected != nil {
        fmt.Println("rejected:", rejected)
    }

    payload := json.Marshal(done)?
    fmt.Println("json:", string(payload))
    return
}

fn main() -> ! {
    if len(os.Args) > 1 && os.Args[1] == "serve" {
        return serve(":8080")
    }
    return runDemo()
}
