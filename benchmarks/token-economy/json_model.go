package main

import "encoding/json"

type User struct {
	ID    int
	Name  string
	Email string
	Tags  []string
}

func (u User) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		ID    int      `json:"id"`
		Name  string   `json:"name"`
		Email string   `json:"email"`
		Tags  []string `json:"tags"`
	}{u.ID, u.Name, u.Email, u.Tags})
}

func (u *User) UnmarshalJSON(data []byte) error {
	var raw struct {
		ID    int      `json:"id"`
		Name  string   `json:"name"`
		Email string   `json:"email"`
		Tags  []string `json:"tags"`
	}
	if err := json.Unmarshal(data, &raw); err != nil {
		return err
	}
	u.ID = raw.ID
	u.Name = raw.Name
	u.Email = raw.Email
	u.Tags = raw.Tags
	return nil
}

func (u User) Clone() User {
	out := u
	out.Tags = append([]string(nil), u.Tags...)
	return out
}
