package main

@derive(String)
enum State {
    Idle
    Running
    Paused
    Done
}

fn next(s: State) -> State {
    match s {
        Idle => State::Running
        Running => State::Paused
        Paused => State::Running
        Done => State::Done
    }
}
