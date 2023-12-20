-- Times are second-precision UNIX timestamps
CREATE TABLE IF NOT EXISTS server_configurations (
    id           INTEGER NOT NULL PRIMARY KEY,
    post_channel INTEGER
);

CREATE TABLE IF NOT EXISTS users (
    id              INTEGER NOT NULL                PRIMARY KEY,
    doing_something BOOLEAN NOT NULL DEFAULT false,

    prompt_frequency INTEGER          DEFAULT null, -- in hours
    prompts_followed INTEGER NOT NULL DEFAULT 0,
    
    message_frequency      INTEGER          DEFAULT null, -- in hours
    stupid_things_messages BOOLEAN NOT NULL DEFAULT true,
    mean_messages          BOOLEAN NOT NULL DEFAULT true,
    extreme_messages       BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS prompt_responses (
    id       INTEGER NOT NULL PRIMARY KEY,
    user_id  INTEGER NOT NULL,
    time     INTEGER NOT NULL,
    accuracy REAL    NOT NULL
);

CREATE TABLE IF NOT EXISTS cum_times (
    id          INTEGER NOT NULL PRIMARY KEY,
    user_id     INTEGER NOT NULL,
    started_at  INTEGER NOT NULL,
    ended_at    INTEGER,
    is_complete BOOLEAN NOT NULL,
    what        TEXT    NOT NULL CHECK(what IN ('gooning', 'prejac')),
    description TEXT    NOT NULL DEFAULT ''
);