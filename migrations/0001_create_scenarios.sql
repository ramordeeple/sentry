CREATE TABLE scenarios (
    date TEXT PRIMARY KEY NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE rates (
    scenario_date TEXT NOT NULL REFERENCES scenarios(date),
    id TEXT NOT NULL,
    num_code TEXT NOT NULL,
    char_code TEXT NOT NULL,
    nominal BIGINT NOT NULL CHECK (nominal > 0),
    name TEXT NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (scenario_date, char_code)
);
