PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS stuckthread_meta(
	stack_id INTEGER PRIMARY KEY AUTOINCREMENT,
	thread_id INTEGER NOT NULL,
	start INTEGER NOT NULL,
	active_duration_ms INTEGER NOT NULL,
	active_monitor_count_start INTEGER NULL,
	active_monitor_count_end INTEGER NULL,
	thread_name TEXT NULL,
	api_request TEXT NULL
);

CREATE TABLE IF NOT EXISTS stuckthread_stack(
	stack_id INTEGER NOT NULL,
	frame_idx INTEGER NOT NULL,
	line_number INTEGER NULL,
	method TEXT NOT NULL,
	frame_source TEXT NOT NULL,
	PRIMARY KEY (stack_id, frame_idx),
	FOREIGN KEY (stack_id) REFERENCES stuckthread_meta(stack_id) DEFERRABLE INITIALLY DEFERRED
);

-- TODO: Better naming!
CREATE TABLE IF NOT EXISTS threaddump_group(
	group_id INTEGER PRIMARY KEY AUTOINCREMENT
);

CREATE TABLE IF NOT EXISTS threaddump(
	group_id INTEGER NOT NULL,
	snapshot_id INTEGER NOT NULL,
	triggered_unix_ms INTEGER NOT NULL,
	FOREIGN KEY (group_id) REFERENCES threaddump_group(group_id) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE IF NOT EXISTS threaddump_threads(
	thread_stack_id INTEGER PRIMARY KEY AUTOINCREMENT,
	thread_id INTEGER NOT NULL,
	thread_name TEXT NULL,
	state TEXT NOT NULL,

	-- TIMED_WAITING/WAITING ON
	wait_object_id INTEGER NULL,

	-- BLOCKED/WAITING TO LOCK
	owner_id INTEGER NULL,
	owner_name TEXT NULL,
	lock_object_id INTEGER NULL,

	UNIQUE (thread_id, thread_stack_id),

	FOREIGN KEY (thread_stack_id) REFERENCES threaddump_stack(thread_stack_id) DEFERRABLE INITIALLY DEFERRED,
	FOREIGN KEY (wait_object_id) REFERENCES threaddump_objects(object_id) DEFERRABLE INITIALLY DEFERRED,
	FOREIGN KEY (lock_object_id) REFERENCES threaddump_objects(object_id) DEFERRABLE INITIALLY DEFERRED
);

CREATE TABLE IF NOT EXISTS threaddump_objects(
	object_id INTEGER PRIMARY KEY AUTOINCREMENT,
	class TEXT NOT NULL,
	identity INTEGER UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS threaddump_stack(
	thread_stack_id INTEGER NOT NULL,
	frame_idx INTEGER NOT NULL,
	-- ELEMENT
	method TEXT NOT NULL,
	frame_source TEXT NOT NULL,
	line_number INTEGER NULL,

	-- LOCK
	object_id INTEGER NULL,
	PRIMARY KEY (thread_stack_id, frame_idx),
	FOREIGN KEY (object_id) REFERENCES threaddump_objects(object_id) DEFERRABLE INITIALLY DEFERRED
);
