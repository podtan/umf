# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2025-12-10

### Added
- **Events Module**: New `events` module for conversation tracking and storage
  - `MessageEvent`: Message events with metadata (timestamp, sequence, token count, model info)
  - `ToolCallEvent`: Tool call events with execution status and MCP context
  - `ToolResultEvent`: Tool result events with duration and error tracking
  - `EventEnvelope`: Type-erased wrapper for any event type
  - `Event` trait: Common interface for all event types
  - `EventType` enum: Message, ToolCall, ToolResult, SystemSignal, Error
  - `ToolCall`: Simple tool call representation for events
  - `ToolResult`: Tool execution result with success/error variants
  - `McpContext`: MCP server context for tool calls
  - `ModelInfo`: Model information for assistant messages
  - `ToolCallStatus`: Pending, Executing, Completed, Failed, Cancelled
- JSONL serialization support via `EventEnvelope::to_json_line()` and `from_json_line()`
- 12 new tests for the events module

### Notes
- Events module is always included (no feature flag required)
- Designed for append-only event logging in `events.jsonl`
- Works with ABK v0.1.38+ split-file checkpoint format

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
