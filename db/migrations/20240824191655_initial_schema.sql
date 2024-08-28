-- migrate:up
CREATE TABLE tasks (
    id bytea PRIMARY KEY, 
    exchange varchar(255),
    routing_key varchar(255) NOT NULL,
    run_at timestamptz NOT NULL,
    payload bytea NOT NULL
);

-- migrate:down

DROP TABLE tasks;
