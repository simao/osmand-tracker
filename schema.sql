-- PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
id TEXT NOT NULL PRIMARY KEY,
name TEXT NOT NULL,
pass TEXT NOT NULL,
ts TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tracking_points (
user_id TEXT NOT NULL,
lat REAL NOT NULL,
lon REAL NOT NULL,
altitude REAL NOT NULL,
speed REAL NOT NULL,
hdop REAL NULL,
bearing TEXT NULL,
received_at TEXT NOT NULL,
ts TEXT NOT NULL
-- FOREIGN KEY(user_id) REFERENCES users(id)
)
;

CREATE INDEX tracking_user_idx ON tracking_points(user_id);

CREATE INDEX tracking_ts_idx ON tracking_points(user_id, ts);
