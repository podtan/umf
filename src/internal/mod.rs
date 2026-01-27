//! Internal format module - the hub for protocol conversion.
//!
//! This module provides protocol-agnostic types that serve as the
//! canonical representation for all message and tool formats.
//!
//! ## Hub Model Architecture
//!
//! The internal format acts as a hub in a hub-and-spoke architecture:
//!
//! ```text
//!                    ┌─────────────┐
//!                    │   OpenAI    │
//!                    │   Format    │
//!                    └──────┬──────┘
//!                           │
//!     ┌─────────────┐       │       ┌─────────────┐
//!     │  Anthropic  ├───────┼───────┤     MCP     │
//!     │   Format    │       │       │   Format    │
//!     └─────────────┘       │       └─────────────┘
//!                           ▼
//!                    ╔═════════════╗
//!                    ║  Internal   ║
//!                    ║   Format    ║
//!                    ╚═════════════╝
//!                           ▲
//!     ┌─────────────┐       │       ┌─────────────┐
//!     │   Google    ├───────┼───────┤    A2A      │
//!     │   Format    │       │       │   Format    │
//!     └─────────────┘       │       └─────────────┘
//! ```
//!
//! ## Usage
//!
//! Convert from protocol-specific to internal format:
//!
//! ```rust,ignore
//! use umf::{McpTool, ToInternal, InternalTool};
//!
//! let mcp_tool: McpTool = /* from MCP server */;
//! let internal: InternalTool = mcp_tool.to_internal();
//! ```
//!
//! Convert from internal to protocol-specific format:
//!
//! ```rust,ignore
//! use umf::{InternalTool, FromInternal, McpTool};
//!
//! let internal: InternalTool = /* from registry */;
//! let mcp_tool: McpTool = McpTool::from_internal(internal);
//! ```

mod tool;
mod traits;

pub use tool::{InternalTool, InternalToolCall, InternalToolResult};
pub use traits::{ConversionError, FromInternal, ToInternal, TryFromInternal, TryToInternal};
