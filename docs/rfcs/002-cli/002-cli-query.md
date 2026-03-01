# RFC 002-CLI-QUERY: Database Query Command

## Status
Proposed

## Context
Users need a way to interact with their GFS-managed databases directly from the CLI. Currently, users would need to manually determine connection details and use native database clients separately.

## Problem Statement
There is no unified way to:
1. Execute ad-hoc SQL queries against GFS-managed databases
2. Open interactive database shells for exploration and debugging
3. Query databases in a database-agnostic manner through the GFS CLI

## Proposed Solution

### Command Syntax
```bash
# Execute a SQL query
gfs query "SELECT * FROM users WHERE id = 1"

# Open interactive database terminal
gfs query
```

### Design Principles
1. **Database Agnostic**: The implementation should support any database system through a generic interface
2. **Native Client Pass-through**: Leverage native database clients (psql, mysql, etc.) for full feature support
3. **Simple UX**: Hide connection complexity from users - automatically detect and use connection details
4. **Interactive Mode**: When no query is provided, open an interactive terminal session

### Implementation Architecture

#### Core Components

1. **Query Command Handler** (`cmd_query.rs`)
   - Parse command arguments (query string, flags)
   - Determine target database from GFS state
   - Delegate to appropriate database executor

2. **Database Executor Interface** (in `ports/database_provider.rs`)
   ```rust
   pub trait DatabaseQueryExecutor {
       /// Execute a SQL query and return results
       fn execute_query(&self, query: &str) -> Result<QueryResult>;

       /// Open interactive terminal session
       fn open_interactive_terminal(&self) -> Result<()>;

       /// Get native CLI command for this database
       fn get_native_cli_command(&self, connection_info: &ConnectionInfo) -> Command;
   }
   ```

3. **Database-Specific Executors**
   - PostgreSQL executor (uses `psql`)
   - MySQL executor (uses `mysql`)
   - Future: SQLite, MongoDB, etc.

#### Execution Flow

```
gfs query [SQL]
    ↓
Parse arguments
    ↓
Load GFS state → Get active database
    ↓
Build connection info
    ↓
SQL provided?
    ├─ Yes → Execute query via native client
    └─ No  → Open interactive terminal
    ↓
Return results / open terminal
```

### Technical Details

#### Connection Information Handling
- Read database connection details from GFS state
- Support both local containers and remote databases
- Pass credentials securely to native clients (via environment variables or connection strings)

#### Native Client Integration
- **PostgreSQL**: Use `psql` with connection string
  ```bash
  psql "postgresql://user:pass@host:port/dbname" -c "SELECT ..."
  ```

- **MySQL**: Use `mysql` with connection parameters
  ```bash
  mysql -h host -P port -u user -p password -D dbname -e "SELECT ..."
  ```

#### Interactive Mode
- When no query is provided, spawn native client without `-c` or `-e` flags
- Inherit current terminal I/O
- Exit code propagation from native client

### Error Handling
- No active database → Error: "No database found. Initialize with `gfs init`"
- Native client not found → Error: "Database client 'psql' not found. Install PostgreSQL client tools."
- Connection failure → Display connection error from native client
- SQL syntax errors → Pass through native client errors

### Security Considerations
- Never log connection strings with passwords
- Use environment variables for credentials when possible
- Warn if executing queries that modify data in production environments

## Alternatives Considered

### Alternative 1: Built-in SQL Parser
**Pros**: No external dependencies, consistent behavior
**Cons**: Massive implementation effort, limited feature support
**Decision**: Rejected - native clients provide better features and compatibility

### Alternative 2: Direct Database Driver Integration
**Pros**: Programmatic result handling, no CLI dependencies
**Cons**: Complex, less features than native clients, harder to maintain
**Decision**: Rejected for v1 - may reconsider for programmatic use cases

## Testing Strategy
- Unit tests for command parsing
- Integration tests with test containers
- Test both query execution and interactive modes
- Test error cases (missing client, bad credentials, etc.)

## Future Enhancements
- Query result formatting (JSON, CSV, table)
- Query history and favorites
- Multi-database query support (query federation)
- Read-only mode for production databases
- Query result export to files

## Open Questions
1. Should we support multiple simultaneous database connections?
2. Should we add query shortcuts/aliases?
3. Should we provide a built-in query builder for common operations?

## Implementation Checklist
- [x] Add `query_client_command` method to `DatabaseProvider` trait
- [x] Implement PostgreSQL query executor
- [x] Implement MySQL query executor
- [x] Add `cmd_query.rs` command handler
- [x] Add CLI command definition and argument parsing
- [ ] Add integration tests
- [ ] Update documentation