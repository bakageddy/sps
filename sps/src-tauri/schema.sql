-- DUCKDB
USE main;

SET
  preserve_insertion_order = false;

CREATE TYPE main.cpumonitoring_thread_state AS ENUM (
  'RUNNABLE',
  'NEW',
  'BLOCKED',
  'WAITING',
  'TIMED_WAITING',
  'TERMINATED'
);

CREATE TYPE main.stuckquery_pgsql_state AS ENUM (
	'active',
	'idle in transaction'
);

CREATE TYPE main.stuckquery_mssql_status AS ENUM (
	'runnable',
	'running',
	'rollback',
	'sleeping',
	'background',
	'suspended'
);

CREATE TABLE IF NOT EXISTS main.cpumonitoring (
  tid UBIGINT NOT NULL,
  timestamp UBIGINT NOT NULL,
  cpu FLOAT NOT NULL,
  state main.cpumonitoring_thread_state NOT NULL,
  name STRING NULL,
);

CREATE TABLE IF NOT EXISTS main.cpumonitoring_stacktraces (
  tid UBIGINT NOT NULL,
  timestamp UBIGINT NOT NULL,
  idx UBIGINT NOT NULL,
  method STRING NOT NULL,
  source STRING NOT NULL,
);

CREATE TABLE IF NOT EXISTS main.windows_cpu_stats (
  timestamp UBIGINT NOT NULL,
  total FLOAT NOT NULL,
  path STRING NULL,
  cpu FLOAT NOT NULL,
  pid UBIGINT NOT NULL,
  name STRING NOT NULL,
);

CREATE TABLE IF NOT EXISTS main.windows_memory_stats (
  timestamp UBIGINT NOT NULL,
  total FLOAT NOT NULL,
  path STRING NULL,
  mem FLOAT NOT NULL,
  pid UBIGINT NOT NULL,
  name STRING NOT NULL,
);

CREATE TABLE IF NOT EXISTS main.linux_stats (
  timestamp UBIGINT NOT NULL,
  total_cpu FLOAT NOT NULL,
  total_mem FLOAT NOT NULL,
  user STRING NOT NULL,
  name STRING NOT NULL,
  pid UBIGINT NOT NULL,
  cpu FLOAT NOT NULL,
  mem FLOAT NOT NULL,
  path STRING NOT NULL,
);

CREATE TABLE IF NOT EXISTS main.stuckthread (
  timestamp UBIGINT NOT NULL,
  tid UBIGINT NOT NULL,
  duration UBIGINT NOT NULL,
  name STRING NOT NULL,
  request STRING NULL,
  active UBIGINT NULL,
);

CREATE TABLE IF NOT EXISTS main.stuckthread_traces (
  timestamp UBIGINT NOT NULL,
  tid UBIGINT NOT NULL,
  idx UBIGINT NOT NULL,
  method STRING NOT NULL,
  source STRING NOT NULL,
);

CREATE TABLE IF NOT EXISTS main.stuckquery_pgsql (
	timestamp UBIGINT NOT NULL,
	pid UBIGINT NOT NULL,
	query_time UBIGINT NULL,
	txn_time UBIGINT NULL,
	db_name STRING NOT NULL,
	state main.stuckquery_pgsql_state NOT NULL,
	waiting BOOLEAN NOT NULL,
	query STRING NOT NULL,
	state_change UBIGINT NOT NULL,
	application_name STRING NULL,
	client_addr UINTEGER NULL,
	client_host STRING NULL,
	client_port USMALLINT NULL,
);

CREATE TABLE IF NOT EXISTS main.stuckquery_mssql (
	timestamp UBIGINT NOT NULL,
	session_id UBIGINT NOT NULL,
	status main.stuckquery_mssql_status NOT NULL,
	txn_id UBIGINT NOT NULL,
	blocked_by UBIGINT NOT NULL,
	wait_type STRING NULL,
	wait_resource STRING NULL,
	wait_time_ms UBIGINT NOT NULL,
	cpu_time_ms UBIGINT NOT NULL,
	logical_reads UBIGINT NOT NULL,
	reads UBIGINT NOT NULL,
	writes UBIGINT NOT NULL,
	elapsed UBIGINT NOT NULL,
	statement STRING NOT NULL,
	command_text STRING NOT NULL,
	command STRING NOT NULL,
	login STRING NOT NULL,
	host STRING NOT NULL,
	db STRING NOT NULL,
	program STRING NOT NULL,
	host_process UBIGINT NOT NULL,
	last_request_end UBIGINT NOT NULL,
	login_time UBIGINT NOT NULL,
	open_txn UBIGINT NOT NULL,
);


CREATE TABLE IF NOT EXISTS main.stuckquery_mssql_blocking (
	timestamp UBIGINT NOT NULL,
	head_blocker UBIGINT NOT NULL,
	session_id UBIGINT NOT NULL,
	txn_id UBIGINT NOT NULL,
	blocking_session_id UBIGINT NOT NULL,
	wait_type STRING NULL,
	wait_duration UBIGINT NOT NULL,
	wait_resource STRING NULL,
	statement_start_offset BIGINT NOT NULL,
	statement_end_offset BIGINT NOT NULL,
	plan_handle STRING NOT NULL,
	sql_handle STRING NOT NULL,
	most_recent_sql_handle STRING NOT NULL,
	level UBIGINT NOT NULL,
	blocker_query_or_most_recent_query STRING NOT NULL,
);
