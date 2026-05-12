PRAGMA foreign_keys = ON;
PRAGMA journal_mode = MEMORY;
PRAGMA synchronous = OFF;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;

CREATE TABLE IF NOT EXISTS object(
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	class TEXT NOT NULL,
	identity INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS stacktrace(
	id INTEGER PRIMARY KEY AUTOINCREMENT
);

CREATE TABLE IF NOT EXISTS stacktrace_elements(
	id INTEGER NOT NULL,
	frame_idx INTEGER NOT NULL,

	-- ELEMENT
	method TEXT NULL,
	frame_source TEXT NULL,
	line_number INTEGER NULL,

	-- LOCK
	object_id INTEGER NULL,
	PRIMARY KEY (id, frame_idx),
	FOREIGN KEY (id) REFERENCES stacktrace(id) DEFERRABLE INITIALLY DEFERRED,
	FOREIGN KEY (object_id) REFERENCES object(id) DEFERRABLE INITIALLY DEFERRED
);

CREATE INDEX IF NOT EXISTS stacktrace_elements_id_frame_idx ON stacktrace_elements(id, frame_idx);
CREATE INDEX IF NOT EXISTS stacktrace_elements_object_id ON stacktrace_elements(object_id);

CREATE TABLE IF NOT EXISTS stuckthread(
	thread_id INTEGER NOT NULL,
	start INTEGER NOT NULL,
	name TEXT NULL,
	request TEXT NULL,
	active_duration_ms INTEGER NOT NULL,
	active_monitor_start INTEGER NULL,
	active_monitor_end INTEGER NULL,
	stack_id INTEGER NOT NULL,
	FOREIGN KEY (stack_id) REFERENCES stacktrace(id) DEFERRABLE INITIALLY DEFERRED
);

CREATE INDEX IF NOT EXISTS stuckthread_stack_id ON stuckthread(stack_id);

CREATE TABLE IF NOT EXISTS threaddump(
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	snapshot INTEGER NOT NULL,
	triggered_unix_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS thread(
	id INTEGER NOT NULL,
	name TEXT NULL,
	state TEXT NOT NULL,

	-- TIMED_WAITING/WAITING ON
	wait_object_id INTEGER NULL,

	-- BLOCKED/WAITING TO LOCK
	owner_id INTEGER NULL,
	owner_name TEXT NULL,
	lock_object_id INTEGER NULL,

	-- REFERENCE KEYS
	stack_id INTEGER NULL,

	-- THREAD DUMP
	dump_id INTEGER NOT NULL,

	UNIQUE (id, stack_id),

	FOREIGN KEY (dump_id) REFERENCES threaddump(id) DEFERRABLE INITIALLY DEFERRED,
	FOREIGN KEY (stack_id) REFERENCES stacktrace(id) DEFERRABLE INITIALLY DEFERRED,
	FOREIGN KEY (wait_object_id) REFERENCES object(id) DEFERRABLE INITIALLY DEFERRED,
	FOREIGN KEY (lock_object_id) REFERENCES object(id) DEFERRABLE INITIALLY DEFERRED
);

CREATE INDEX IF NOT EXISTS thread_dump_id ON thread(dump_id);
CREATE INDEX IF NOT EXISTS thread_stack_id ON thread(stack_id);
CREATE INDEX IF NOT EXISTS thread_wait_object_id ON thread(wait_object_id);
CREATE INDEX IF NOT EXISTS thread_lock_object_id ON thread(lock_object_id);
