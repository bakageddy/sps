PRAGMA memory_limit = '1GB';
SET preserve_insertion_order = false;

CREATE TABLE IF NOT EXISTS stuckthread_events (
	tid	UBIGINT NOT NULL,
	start UBIGINT NOT NULL,
	active_duration_ms UINTEGER NOT NULL,
	active_monitor UINTEGER DEFAULT 0,
	begin_event BOOLEAN DEFAULT TRUE NOT NULL,
	name STRING NULL,
	request STRING NULL,
);

CREATE TABLE IF NOT EXISTS stuckthread_stacktraces (
    tid				UBIGINT NOT NULL, 
	stamp			UBIGINT NOT NULL,
    line			UBIGINT NULL,
    frame_idx		UINTEGER NOT NULL,
    method			STRING NULL,
    frame_source	STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS thread_stacktrace (
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
	identity			UBIGINT NULL,
	owner_id			UBIGINT NULL,
	snapshot			UTINYINT NOT NULL,
	owner_name			STRING NULL,
	class				STRING NULL,
	name				STRING NULL,
    state				STRING NOT NULL,
);

CREATE TABLE IF NOT EXISTS stuckquery_pgsql (
	timestamp			UBIGINT NOT NULL,
	pid					UBIGINT NOT NULL,
	query_time_ms		UBIGINT NOT NULL,
	txn_time_ms			UBIGINT NOT NULL,
	last_state_change	UBIGINT NOT NULL,
	client_port			USMALLINT NULL,
	active				BOOLEAN NOT NULL,
	waiting				BOOLEAN NOT NULL,
	client_address		STRING NULL,
	db_name				STRING NULL,
	query				STRING NULL,
	application_name	STRING NULL,
	client_hostname		STRING NULL,
);

CREATE TABLE IF NOT EXISTS stuckquery_mssql (
	timestamp				UBIGINT NOT NULL,
	sessionid				UBIGINT NOT NULL,
	status					STRING NOT NULL,
	txn_id					UBIGINT	NOT NULL,		
	blocked_by				UBIGINT NOT NULL,
	wait_type				STRING NULL,
	wait_resource			STRING NULL,
	wait_time_ms			UBIGINT NOT NULL,
	cpu_time_ms				UBIGINT NOT NULL,
	logical_reads			UBIGINT NOT NULL,
	physical_reads			UBIGINT NOT NULL,
	physical_writes			UBIGINT NOT NULL,
	elapsed_time_ms			UBIGINT NOT NULL,
	statement				STRING NOT NULL,
	command_text			STRING NULL,
	command					STRING NULL,
	login_name				STRING NOT NULL,
	host_name				STRING NOT NULL,
	db_name					STRING NOT NULL,
	program_name			STRING NOT NULL,
	host_process_id			UBIGINT NOT NULL,
	last_request_end_ms		UBIGINT NOT NULL,
	login_time_ms			UBIGINT NOT NULL,
	open_transaction_count	UBIGINT NOT NULL,
);

CREATE TABLE IF NOT EXISTS cpumonitoring (
	tid						UBIGINT NOT NULL,
	timestamp				UBIGINT NOT NULL,
	name					STRING NULL,
	state					STRING NULL,
	cpu						FLOAT NOT NULL,	
);

CREATE TABLE IF NOT EXISTS cpumonitoring_traces (
    tid				UBIGINT NOT NULL, 
	stamp			UBIGINT NOT NULL,
    line			UBIGINT NULL,
    frame_idx		UINTEGER NOT NULL,
    method			STRING NULL,
    frame_source	STRING NOT NULL
);

CREATE TABLE IF NOT EXISTS cpumemstats (
	timestamp				UBIGINT NOT NULL,
	process_id				UBIGINT NOT NULL,
	total_usage				FLOAT NOT NULL,
	usage					FLOAT NOT NULL,
	path					STRING NOT NULL,
	name					STRING NOT NULL,
	is_cpu					BOOLEAN NOT NULL,
);
