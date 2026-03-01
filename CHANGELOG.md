# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Issue templates for bug reports, feature requests, and enhancements
- Pull request template
- Dependabot configuration for automated dependency updates

## [0.1.4] - 2026-03-01

### Added
- CLI commands: `export`, `import`, `query`, `schema`
- Schema extraction and schema diff functionality
- Import/export repository usecases
- RFCs for export, import, query, and schema commands
- Qwery agent for database query assistance
- GFS showcase video and website updates
- Skills for GFS CLI and MCP usage
- E2E tests for schema extraction and import

### Changed
- Improved CLI output formatting
- Enhanced compute Docker adapters for MySQL and PostgreSQL

## [0.1.3] - 2026-02-25

### Fixed
- CI pipeline to install correct Rust target

## [0.1.2] - 2026-02-25

### Added
- Binary release workflow
- Release artifacts for multiple platforms

### Changed
- Improved CI/CD pipeline

## [0.1.1] - 2026-02-25

### Fixed
- Release pipeline configuration
- Code formatting issues
- CLI storage adapter
- Build issues
- Test suite fixes

### Added
- Default file storage implementation

## [0.1.0] - 2026-02-25

### Added
- Initial project structure with hexagonal architecture
- Core domain layer with ports and adapters pattern
- CLI application with basic commands (`init`, `commit`, `log`, `status`)
- MCP (Model Context Protocol) server support
- Storage adapters:
  - APFS (Apple File System) support for macOS
  - Generic file-based storage adapter
- Compute adapters:
  - Docker integration for database containers
- Database providers:
  - PostgreSQL support
  - MySQL support
  - ClickHouse support
- Configuration management system
- Telemetry and logging infrastructure
- Integration test suite
- CI/CD with GitHub Actions
- Documentation:
  - RFC documents for architecture and design decisions
  - README with basic usage instructions
  - Contributing guidelines
  - Code of Conduct
  - Security policy

[unreleased]: https://github.com/Guepard-Corp/gfs/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/Guepard-Corp/gfs/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/Guepard-Corp/gfs/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/Guepard-Corp/gfs/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Guepard-Corp/gfs/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Guepard-Corp/gfs/releases/tag/v0.1.0
