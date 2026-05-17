CREATE TYPE IF NOT EXISTS ThreadState AS ENUM (
    'NEW',
    'RUNNABLE',
    'BLOCKED',
    'WAITING',
    'TIMED_WAITING',
    'TERMINATED'
);

CREATE TABLE IF NOT EXISTS stuckthread (
    tid                   UBIGINT  NOT NULL,
    start                 UBIGINT  NOT NULL,           -- unix millis
    active_duration_ms    UBIGINT  NOT NULL,
    active_monitor_start  UINTEGER DEFAULT 0,
    active_monitor_end    UINTEGER DEFAULT 0,
    name                  STRING   NULL,
    request               STRING   NULL
);

-- for thread related stacktraces: tid, triggered_unix_ms
-- for stuckthread related stacktraces: tid, start
CREATE TABLE IF NOT EXISTS stacktrace (
    tid				UBIGINT NOT NULL, 
	stamp			UBIGINT NOT NULL,

    line			UBIGINT NULL,
    identity		UBIGINT NULL,
    frame_idx		UINTEGER NOT NULL,
    class			STRING NULL,
    method			STRING NULL,
    frame_source	STRING NULL
);

CREATE TABLE IF NOT EXISTS thread (
	tid					UBIGINT NOT NULL,
	triggered_unix_ms	UBIGINT NOT NULL,	
	identity			UBIGINT NOT NULL,

	owner_id			UBIGINT NULL,
	owner_name			STRING NULL,
	class				STRING NOT NULL,
	name				STRING NULL,
    state				ThreadState NOT NULL,
);
