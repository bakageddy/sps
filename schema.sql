CREATE TABLE IF NOT EXISTS stuckthread_meta(
	stack_id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
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
	PRIMARY KEY (stack_id, frame_idx)
);
