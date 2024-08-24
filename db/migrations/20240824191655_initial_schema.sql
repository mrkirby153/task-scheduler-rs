-- migrate:up
CREATE TABLE tasks (
    id bytea PRIMARY KEY, 
    topic varchar(255) NOT NULL,
    run_at timestamp NOT NULL,
    payload bytea NOT NULL
);

-- migrate:down

DROP TABLE tasks;
