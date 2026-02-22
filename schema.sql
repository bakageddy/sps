CREATE TABLE IF NOT EXISTS stuckthread_meta(
	stack_id INTEGER NOT NULL,
	thread_id INTEGER NOT NULL,
	thread_name TEXT NULL,
	api_request TEXT NULL,
	active_duration_ms INTEGER NOT NULL,
	active_monitor_count_start INTEGER NULL,
	active_monitor_count_end INTEGER NULL,
);

CREATE TABLE IF NOT EXISTS stuckthread_stack(
	stack_id INTEGER NOT NULL,
	frame_idx INTEGER NOT NULL,
	method TEXT NOT NULL,
	frame_source TEXT NOT NULL,
	line_number INTEGER NULL,
	PRIMARY KEY (stack_id, frame_idx)
);
