# SPS (Stuck Thread Parser & Server)

  High-performance Rust parser for Tomcat `StuckThreadValve` stuck thread logs and thread dumps.
  It extracts thread metadata and stack frames into a structured SQLite database for analysis.

  Includes an embedded Model Context Protocol (MCP) server, allowing AI agents (like Claude Desktop) to directly query, aggregate, and inspect thread dumps.

  ## Architecture

  - **Ingestion Engine:** Zero-copy parsing of massive text dumps directly into SQLite.
  - **Analysis Engine:** Exposes diagnostic endpoints to aggregate thread failures, group by endpoints, and extract 200+
frame stack traces.

  ## Usage

  The `sps` binary provides two main commands currently: `parse` (for ingestion) and `mcp` (for analysis).

  ### 1. Ingesting Data (`parse`)

  Parse a raw JVM thread dump file and load it into a SQLite database.

  ```bash
  # Parse dump into a default/auto-generated database
  sps parse --path /path/to/sps/logs

  # Parse dump into a specific database file
  sps parse --path /path/to/sps/logs --database /path/to/sample1.db
  ```

### 2. Running the MCP Server ( mcp )

Expose the parsed SQLite database to an AI agent via the Model Context Protocol.

Standard I/O Mode (Default for Claude Desktop): Runs the server over  stdin / stdout .

```bash
  sps mcp --stdio --database /path/to/sample1.db
```

HTTP/SSE Mode: Runs a local web server for MCP clients that connect via HTTP Server-Sent Events.

```bash
  sps mcp --database /path/to/sample1.db --bind 127.0.0.1 --port 8080
```


### 3. Web UI ( web )

(Coming Soon) - A dedicated web interface for exploring parsed thread dumps.
