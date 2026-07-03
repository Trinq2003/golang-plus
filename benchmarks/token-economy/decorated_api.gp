package main

import "fmt"

@log
@retry(3, 10)
fn fetch(id: int) -> string! {
    return fmt.Sprintf("item-%d", id)
}
