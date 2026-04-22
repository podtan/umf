# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.6] - 2026-04-22

### Fixed
- Include `reasoning_content` in token counting via `to_chatml_string()`
  - Reasoning/thinking content was excluded from ChatML string serialization,
    causing `count_tokens()` to significantly underreport context size
  - Added reasoning_content wrapped in `<think />` tags so the tiktoken
    counter sees all tokens actually sent to the LLM API

## [0.2.5] - 2026-04-11

### Added
- `count_tokens_for_text()` helper function for arbitrary text tokenization using cl100k_base
  - Useful for measuring tool definition sizes, system prompts, or text outside conversation messages
  - Re-exported from library root

## [0.2.4] - 2026-02-20

### Changed
- Added debug logging for tool call argument accumulation (when `RUST_LOG` contains "debug")

## [0.2.2] - 2026-02-20

### Added
- **Reasoning/Thinking Support**: Full support for thinking models (GLM, Qwen3, DeepSeek)
  - `StreamChunk::Reasoning(String)`: New variant for reasoning content deltas
  - `AccumulatedResponse::reasoning`: New field to store accumulated reasoning
  - `ChatMLMessage::reasoning_content`: Optional field for reasoning content
  - `ChatMLMessage::new_assistant_with_reasoning()`: Constructor for messages with reasoning
  - `ChatMLFormatter::add_assistant_message_with_reasoning()`: Add assistant message with reasoning
- `to_dict()` now includes `reasoning_content` field when present for API calls

### Changed
- `StreamingAccumulator` now handles `StreamChunk::Reasoning` chunks
- Reasoning is accumulated separately from text content

### Fixed
- Thinking model responses now properly capture reasoning content
- Reasoning is included in API requests back to thinking models

## [0.2.1] - 2026-02-17

### Changed
- **GenerateResult::ToolCalls** now includes optional text content
  - Changed from tuple variant `ToolCalls(Vec<ToolCall>)` to struct variant
  - New struct variant: `ToolCalls { calls: Vec<ToolCall>, content: Option<String> }`
  - `content` field captures text that precedes tool calls in streaming responses
  - This fixes assistant text responses being lost when tools are also invoked

### Fixed
- Assistant messages with both text content and tool calls now properly preserve the text
- Checkpoint conversation snapshots now include full assistant responses

## [0.2.0] - 2025-01-27

### Added
- **MCP Module** (`mcp` feature): New module for Model Context Protocol tool support
  - `McpTool`: MCP-native tool representation with name, description, and input schema
  - `McpToolAnnotations`: Tool annotations (title, read_only_hint, destructive_hint, etc.)
  - `McpTool::new()`: Constructor for creating tools with name, description, and parameters
  - `McpTool::from_schema()`: Create tool from raw JSON schema value
  - Builder methods: `with_title()`, `with_read_only_hint()`, `with_destructive_hint()`, 
    `with_idempotent_hint()`, `with_open_world_hint()` for fluent configuration
- **Internal Module** (`internal` feature): Hub model for multi-source tool aggregation
  - `InternalToolDefinition`: Provider-agnostic tool representation
  - Conversion traits between `InternalToolDefinition` and `McpTool`
  - `impl From<McpTool> for InternalToolDefinition`
  - `impl From<InternalToolDefinition> for McpTool`

### Changed
- MCP tools now use proper constructor methods instead of direct struct initialization
- Internal tools can be seamlessly converted to/from MCP format

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
