-- Times are second-precision UNIX timestamps
CREATE TABLE IF NOT EXISTS server_configurations (
    id           INTEGER NOT NULL PRIMARY KEY,
    post_channel INTEGER
);

CREATE TABLE IF NOT EXISTS users (
    id              INTEGER NOT NULL PRIMARY KEY,
    doing_something BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS cum_times (
    id          INTEGER NOT NULL PRIMARY KEY,
    user_id     INTEGER NOT NULL,
    started_at  INTEGER NOT NULL,
    ended_at    INTEGER,
    is_complete BOOLEAN NOT NULL,
    what        TEXT    NOT NULL CHECK(what IN ('gooning', 'prejac'))
);