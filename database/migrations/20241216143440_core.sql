-- Add migration script here

CREATE TABLE IF NOT EXISTS rank (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_name TEXT NOT NULL,
    used_time TIMESTAMP NOT NULL
);