# UMF Internal Format

This document describes the Universal Message Format (UMF) used for provider-agnostic message representation in LLM interactions. UMF provides a standardized way to represent messages, content blocks, and tool calls that can be converted to any LLM provider format.

## Goals

- Document the Universal Message Format structures and types
- Provide examples of message creation and tool calling
- Show how UMF enables provider-agnostic LLM interactions

## Core Concepts

UMF is designed to be:
- **OpenAI-Compatible Base**: Follows OpenAI's JSON structure as the foundation
- **Provider-Agnostic**: Can be converted to any LLM provider format (Anthropic, Google Gemini, Cohere, etc.)
- **Metadata Support**: Includes optional metadata for internal tracking
- **Tool Calling Support**: Full support for function/tool calling across providers

## InternalMessage Structure

The core message structure used internally by UMF components:

```json
{
  "role": "user",
  "content": "Hello, world!",
  "metadata": {},
  "tool_call_id": null,
  "name": null
}
```

### Fields

- `role` (string): Message role - one of "system", "user", "assistant", "tool"
- `content` (MessageContent): The message content (text or structured blocks)
- `metadata` (object, optional): Provider-specific metadata for internal tracking
- `tool_call_id` (string, optional): ID for tool result messages
- `name` (string, optional): Tool name for tool result messages

## MessageContent Types

UMF supports two types of message content:

### Text Content

Simple text messages:

```json
{
  "role": "user",
  "content": "Hello, world!"
}
```

### Blocks Content

Structured content with multiple blocks for complex messages:

```json
{
  "role": "assistant",
  "content": [
    {
      "type": "text",
      "text": "Let me search for that information."
    },
    {
      "type": "tool_use",
      "id": "call_123",
      "name": "search",
      "input": {"query": "rust programming"}
    }
  ]
}
```

## ContentBlock Types

### Text Block

```json
{
  "type": "text",
  "text": "Hello, world!"
}
```

### Image Block

```json
{
  "type": "image",
  "source": {
    "type": "url",
    "url": "https://example.com/image.png"
  }
}
```

Or with base64 data:

```json
{
  "type": "image",
  "source": {
    "type": "base64",
    "media_type": "image/png",
    "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
  }
}
```

### Tool Use Block

```json
{
  "type": "tool_use",
  "id": "call_123",
  "name": "search",
  "input": {
    "query": "rust programming",
    "limit": 10
  }
}
```

### Tool Result Block

```json
{
  "type": "tool_result",
  "tool_use_id": "call_123",
  "content": "Found 5 results..."
}
```

## Tool Calling

UMF supports tool calling with the following structures:

### Tool Definition

```json
{
  "type": "function",
  "function": {
    "name": "search",
    "description": "Search for information",
    "parameters": {
      "type": "object",
      "properties": {
        "query": {
          "type": "string",
          "description": "Search query"
        },
        "limit": {
          "type": "integer",
          "description": "Maximum results"
        }
      },
      "required": ["query"]
    }
  }
}
```

### Tool Call

```json
{
  "id": "call_123",
  "type": "function",
  "function": {
    "name": "search",
    "arguments": "{\"query\": \"rust\", \"limit\": 5}"
  }
}
```

## Message Role Types

- `system`: System-level instructions and context
- `user`: User input messages
- `assistant`: AI assistant responses (may include tool calls)
- `tool`: Results from tool execution

## Usage Examples

### Simple Text Conversation

```rust
use umf::{InternalMessage, MessageRole};

let system_msg = InternalMessage::system("You are a helpful assistant.");
let user_msg = InternalMessage::user("What is Rust?");
let assistant_msg = InternalMessage::assistant("Rust is a systems programming language.");
```

### Tool Calling

```rust
use umf::{InternalMessage, ContentBlock};
use serde_json::json;

let tool_call = ContentBlock::tool_use("call_123", "search", json!({"query": "rust"}));
let assistant_msg = InternalMessage::assistant_with_tools("Searching...", vec![tool_call]);

let tool_result = InternalMessage::tool_result("call_123", "search", "Found results...");
```

## Provider Conversion

UMF messages can be converted to provider-specific formats using formatters:

- `ChatMLFormatter`: For OpenAI ChatML format
- `AnthropicFormatter`: For Anthropic Claude format
- `GeminiFormatter`: For Google Gemini format

Each formatter handles the conversion of UMF structures to the provider's expected JSON format.

## Versioning

- Current version: 1.0
- Schema changes will increment the version number
- Backwards compatibility is maintained where possible

## Implementation Notes

- All messages are serialized using serde with JSON
- Content blocks use tagged enums for type safety
- Tool calls follow OpenAI's function calling specification as the base
- Metadata fields are optional and may be ignored by formatters