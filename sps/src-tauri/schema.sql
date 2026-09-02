-- DUCKDB
USE main;

SET
  preserve_insertion_order = false;

CREATE TABLE IF NOT EXISTS main.cpumonitoring (
  tid UBIGINT NOT NULL,
  timestamp UBIGINT NOT NULL,
  cpu FLOAT NOT NULL,
  state STRING NOT NULL,
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
