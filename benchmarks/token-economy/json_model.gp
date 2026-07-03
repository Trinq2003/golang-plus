package main

@derive(JsonMarshal, JsonUnmarshal, Clone)
struct User {
    ID:    int      `json:"id"`
    Name:  string   `json:"name"`
    Email: string   `json:"email"`
    Tags:  []string `json:"tags"`
}
