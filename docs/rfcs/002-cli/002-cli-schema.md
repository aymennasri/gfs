# RFC: Schema Extraction (`gfs schema`)

## Status

**Implemented** - 2026-02-28

## Summary

Add `gfs schema` command and `extract_schema` MCP tool to extract and inspect database schema metadata in structured JSON format. This enables programmatic understanding of database structure without manual SQL queries.

## Motivation

- **Schema Introspection**: Fundamental capability for database tooling and automation
- **MCP Integration**: Allow AI agents to understand database structure for intelligent query generation
- **Documentation**: Generate schema documentation from live databases
- **Schema Comparison**: Enable schema diff and validation workflows
- **Metadata Model**: The rich `DatasourceMetadata` model exists but wasn't being populated from live databases

## Design

### Architecture

Following the established use case pattern with these components:

1. **Domain Layer**: `ExtractSchemaUseCase` orchestrates schema extraction
2. **Port Extension**: `schema_extraction_queries()` method on `DatabaseProvider` trait
3. **Adapters**: PostgreSQL and MySQL providers implement database-specific schema queries
4. **Applications**: CLI command (`gfs schema`) and MCP tool (`extract_schema`)

### Query Strategy

Uses **direct SQL queries** against system catalogs:
- PostgreSQL: Query `pg_catalog` and `information_schema`
- MySQL: Query `information_schema` tables

**Rationale**: Read-only queries don't require sidecar containers. Uses existing query infrastructure to execute SQL and capture structured JSON output.

### Implementation

#### 1. Database Provider Trait

Added `schema_extraction_queries()` method to `DatabaseProvider` trait:

```rust
fn schema_extraction_queries(&self) -> HashMap<String, String> {
    HashMap::new() // Default: provider doesn't support schema extraction
}
```

**Query keys**:
- `"version"`: Database version string
- `"schemas"`: List of schemas/namespaces (JSON array)
- `"tables"`: List of tables with metadata (JSON array)
- `"columns"`: List of columns with full metadata (JSON array)

#### 2. Provider Implementations

**PostgreSQL** (`postgresql.rs`):
- Queries `pg_namespace` for schemas
- Queries `pg_class` + `pg_stat_user_tables` for tables
- Queries `information_schema.columns` for column metadata
- Returns JSON arrays using `json_agg(row_to_json(...))`

**MySQL** (`mysql.rs`):
- Queries `information_schema.SCHEMATA` for schemas
- Queries `information_schema.TABLES` for tables
- Queries `information_schema.COLUMNS` for columns
- Returns JSON arrays using `JSON_ARRAYAGG(JSON_OBJECT(...))`

#### 3. Use Case

`ExtractSchemaUseCase` (`extract_schema_usecase.rs`):
1. Loads repo config to get provider and container name
2. Resolves provider from registry
3. Gets connection info via `Compute::get_task_connection_info`
4. Retrieves schema queries from provider
5. Executes each query using `query_client_command`
6. Parses JSON results into `DatasourceMetadata`
7. Returns `SchemaOutput` with populated metadata

#### 4. CLI Command

`gfs schema` (`cmd_schema.rs`):

```bash
gfs schema [--path <dir>] [--output <file>] [--compact]
```

**Flags**:
- `--path`: Repository root (default: current directory)
- `--output`: Write to file instead of stdout
- `--compact`: Output compact JSON (default: pretty-printed)

#### 5. MCP Tool

`extract_schema` tool in MCP server:

```json
{
  "name": "extract_schema",
  "description": "Extract database schema metadata from the running database instance...",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": { "type": "string", "description": "repo root path" }
    }
  }
}
```

Returns complete `DatasourceMetadata` as JSON.

## MVP Scope (Phase 1)

**Implemented**:
- ✅ Database version and driver info
- ✅ Schemas (namespaces)
- ✅ Tables (with basic metadata: size, comments)
- ✅ Columns (full metadata: types, constraints, defaults, nullability)
- ✅ Primary keys (placeholder in table metadata)
- ✅ Foreign key relationships (placeholder in table metadata)

**Not yet populated** (Phase 2+):
- Views, indexes, functions, triggers
- PostgreSQL-specific: materialized views, policies, extensions, custom types
- Roles and privileges
- Publications and replication metadata

## Output Format

Example output:

```json
{
  "version": "PostgreSQL 16.1",
  "driver": "postgres",
  "schemas": [
    {
      "id": 2200,
      "name": "public",
      "owner": "postgres"
    }
  ],
  "tables": [
    {
      "id": 16384,
      "schema": "public",
      "name": "users",
      "rls_enabled": false,
      "rls_forced": false,
      "bytes": 8192,
      "size": "8192 bytes",
      "live_rows_estimate": 0,
      "dead_rows_estimate": 0,
      "comment": null,
      "primary_keys": [],
      "relationships": []
    }
  ],
  "columns": [
    {
      "id": "public.users.id",
      "table_id": 16384,
      "schema": "public",
      "table": "users",
      "name": "id",
      "ordinal_position": 1,
      "data_type": "integer",
      "format": "int4",
      "is_identity": false,
      "identity_generation": null,
      "is_generated": false,
      "is_nullable": false,
      "is_updatable": true,
      "is_unique": true,
      "check": null,
      "default_value": null,
      "enums": [],
      "comment": null
    }
  ]
}
```

## Usage Examples

### CLI

```bash
# Extract schema and print to stdout
gfs schema

# Extract schema and save to file
gfs schema --output schema.json

# Compact output
gfs schema --compact

# Specify repo path
gfs schema --path /path/to/repo
```

### MCP

```python
# Using MCP client
result = await mcp_client.call_tool("extract_schema", {
    "path": "/path/to/repo"
})
schema = json.loads(result.content[0].text)
print(f"Found {len(schema['tables'])} tables")
```

## Testing Strategy

### Manual Testing

1. **PostgreSQL**:
   ```bash
   gfs init --database-provider postgres --database-version 16
   gfs compute start
   gfs query "CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT);"
   gfs schema
   ```

2. **MySQL**:
   ```bash
   gfs init --database-provider mysql --database-version 8.0
   gfs compute start
   gfs query "CREATE TABLE users (id INT PRIMARY KEY AUTO_INCREMENT, name VARCHAR(255));"
   gfs schema
   ```

### Integration Tests

Integration tests can be added in `integration_tests/src/test_schema_extraction.rs`:
- Test PostgreSQL schema extraction with sample tables
- Test MySQL schema extraction with sample tables
- Test CLI command end-to-end
- Test MCP tool
- Test error cases (no database, not running, etc.)

## Future Enhancements

### Phase 2 Features

- **Views and Materialized Views**: Extract view definitions and dependencies
- **Indexes**: Full index metadata with attributes (unique, partial, etc.)
- **Functions**: Function signatures, arguments, return types
- **Triggers**: Trigger definitions and event handlers
- **Custom Types**: Enums, composites, domains
- **RLS Policies**: Row-level security policies
- **Roles and Privileges**: Permission metadata
- **Extensions**: PostGIS, pgcrypto, etc.

### Additional Improvements

- **Filtering**: `--schema <name>` to extract specific schemas only
- **Include/Exclude Patterns**: Selective extraction
- **Parallel Queries**: Better performance for large databases
- **Caching**: TTL-based caching for repeated queries
- **Schema Diff**: Compare two schema extractions
- **Schema Validation**: Validate against expected structure

## Files Changed

**Created**:
- `crates/domain/src/usecases/repository/extract_schema_usecase.rs`
- `crates/applications/cli/src/commands/cmd_schema.rs`
- `docs/rfcs/002-cli/002-cli-schema.md`

**Modified**:
- `crates/domain/src/ports/database_provider.rs` (added `schema_extraction_queries()`)
- `crates/domain/src/usecases/repository/mod.rs` (export module)
- `crates/adapters/compute-docker/src/containers/postgresql.rs` (implement queries)
- `crates/adapters/compute-docker/src/containers/mysql.rs` (implement queries)
- `crates/applications/cli/src/commands/mod.rs` (export cmd_schema)
- `crates/applications/cli/src/main.rs` (wire up Schema subcommand)
- `crates/applications/mcp/src/tools.rs` (add extract_schema tool)

## References

- Plan: [Schema Extraction Implementation Plan](https://github.com/Guepard-Corp/gfs/issues/XXX)
- Related: [Export RFC](./002-cli-export.md), [Query RFC](./002-cli-query.md)
- Model: `crates/domain/src/model/datasource/metadata.rs`
