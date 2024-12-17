-- Add migration script here

CREATE TABLE IF NOT EXISTS rank (
    id INTEGER PRIMARY KEY,
    user_name TEXT NOT NULL,
    used_time INTEGER NOT NULL,
    correct_num INTEGER NOT NULL
);