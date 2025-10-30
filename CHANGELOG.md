# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
