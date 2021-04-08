CREATE TABLE IF NOT EXISTS players (
    name    TEXT NOT NULL UNIQUE ON CONFLICT REPLACE,
    rating  INTEGER NOT NULL DEFAULT(1000)
);
