# UMF - Universal Message Format

A provider-agnostic message representation for LLM (Large Language Model) interactions.

## Overview

UMF provides a standardized message format that can be converted to any LLM provider format (OpenAI, Anthropic, Google Gemini, Cohere, etc.). It follows OpenAI's JSON structure as the foundation while supporting flexible conversion to provider-specific formats.

## Features

- **OpenAI-Compatible Base**: Follows OpenAI's message structure
- **Provider-Agnostic**: Easily convert to any LLM provider format
- **Tool Calling Support**: Full support for function/tool calling
- **Type-Safe**: Strongly typed Rust API with comprehensive validation
- **Spec-Compliant**: Matches the Universal Message Format specification exactly

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
umf = "0.1.0"
```

### Basic Example

```rust
use umf::{InternalMessage, MessageRole, ContentBlock};

// Create a simple text message
let msg = InternalMessage::user("Hello, world!");

// Create a message with tool calls
let msg = InternalMessage::assistant_with_tools(
    "Let me search for that",
    vec![ContentBlock::tool_use(
        "call_123",
        "search",
        serde_json::json!({"query": "rust programming"}),
    )],
);

// Create a tool result message
let msg = InternalMessage::tool_result(
    "call_123",
    "search",
    "Found 5 results about rust programming",
);
```

### Message Structure

```rust
pub struct InternalMessage {
    pub role: MessageRole,              // system, user, assistant, tool
    pub content: MessageContent,        // Text or Blocks
    pub metadata: HashMap<String, String>,
    pub tool_call_id: Option<String>,   // For tool messages
    pub name: Option<String>,           // For tool messages
}
```

### Content Types

**Simple Text:**
```rust
let msg = InternalMessage::user("Hello");
```

**Structured Content Blocks:**
```rust
use umf::ContentBlock;

let blocks = vec![
    ContentBlock::text("I'll help you with that"),
    ContentBlock::tool_use("call_123", "search", serde_json::json!({"q": "test"})),
];

let msg = InternalMessage {
    role: MessageRole::Assistant,
    content: MessageContent::Blocks(blocks),
    // ...
};
```

### Tool Calling

**Tool Use:**
```rust
let tool_call = ContentBlock::tool_use(
    "call_abc123",
    "search",
    serde_json::json!({"query": "weather"}),
);
```

**Tool Result:**
```rust
let msg = InternalMessage::tool_result(
    "call_abc123",
    "search",
    "It's sunny, 72°F",
);
```

## JSON Serialization

UMF types serialize to JSON exactly as specified:

**Text Block:**
```json
{
  "type": "text",
  "text": "Hello world"
}
```

**Tool Use Block:**
```json
{
  "type": "tool_use",
  "id": "call_123",
  "name": "search",
  "input": {"query": "weather"}
}
```

**Tool Result Block:**
```json
{
  "type": "tool_result",
  "tool_use_id": "call_123",
  "content": "Result text"
}
```

## Provider Conversion

UMF can be converted to any provider format:

### OpenAI
- Keep `role`, `content`, `tool_calls` as-is
- Remove internal metadata fields

### Anthropic
- Extract system messages to separate parameter
- Convert `tool_calls` to content blocks
- Convert `role: "tool"` to `role: "user"` with `tool_result` blocks

### Google Gemini / Cohere
- Follow similar conversion patterns based on provider API

## Spec Compliance

UMF follows the Universal Message Format specification:
- Lowercase `type` field (`"text"`, `"tool_use"`, `"tool_result"`)
- Flattened block fields (no nested `data` objects)
- OpenAI-compatible structure as baseline
- Full round-trip serialization support

## Testing

Run tests:
```bash
cargo test -p umf
```

All tests include:
- Message creation and serialization
- Content block validation
- Spec compliance verification
- Round-trip serialization tests

## License

MIT OR Apache-2.0

## Related

- [Simpaticoder](https://github.com/podtan/simpaticoder) - Terminal-first software engineering agent
- Universal Message Format Specification - See `specs/universal-message-format/`
