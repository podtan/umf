# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-11-12

### Added
- **UDML/URP Support**: Full Universal Data Morphism Language integration
  - Complete UDML specification in `umf.udml.yaml` defining all six domains
  - Compile-time UDML validation using `udml` crate
  - Runtime access to UDML specification via `udml_spec` module
  - New `udml` feature flag for optional UDML/URP functionality
- **UDML Domains Defined**:
  - Information: 9 message entities (InternalMessage, ChatML, streaming, etc.)
  - Access: 4 access rules for message operations
  - Manipulation: 7 mutations for message creation and updates
  - Extract: 6 transforms for format conversion and token counting
  - Movement: 3 routes for message flow (to provider, streaming, internal)
  - Coordination: 2 primitives for orchestration patterns
- **Generated Constants**: Entity IDs, operation IDs, and schema references from UDML spec
- **Build Integration**: `build.rs` validates UDML at compile time

### Changed
- Bumped `udml` dependency to `0.1.0` (build and optional runtime)
- Enhanced module organization with `udml_spec` module

### Documentation
- Added comprehensive UDML specification documenting all message operations
- Schema references for all message types and transformations

## [0.1.0] - 2025-10-30

### Added
- Initial release of Universal Message Format (UMF)
- Core message types:
  - `InternalMessage` - Provider-agnostic message structure
  - `MessageRole` - System, User, Assistant, Tool roles
  - `MessageContent` - Text or structured content blocks
  - `ContentBlock` - Text, ToolUse, ToolResult, Image variants
  - `ImageSource` - Base64 or URL image sources
- OpenAI-compatible tool types:
  - `ToolCall` - Function call structure
  - `FunctionCall` - Function invocation details
  - `Function` - Tool definition
  - `Tool` - Complete tool specification
  - `GenerateResult` - Generation result enum
- Complete test suite with 11 tests covering:
  - Message creation and serialization
  - Content block validation
  - Spec compliance verification
  - Round-trip serialization
- Comprehensive documentation:
  - API documentation with examples
  - README with usage guide
  - Type-level documentation

### Features
- OpenAI-compatible message structure
- Provider-agnostic design for easy conversion
- Full tool calling support
- Type-safe Rust API
- Spec-compliant JSON serialization
- Zero unsafe code
- Minimal dependencies (serde, serde_json)

[Unreleased]: https://github.com/podtan/umf/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/podtan/umf/releases/tag/v0.1.0
