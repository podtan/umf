//! MCP (Model Context Protocol) type definitions.
//!
//! This module provides types matching the MCP specification with
//! conversions to/from UMF internal format.
//!
//! ## Tool Types
//!
//! - [`McpTool`] - Tool definition with input schema
//! - [`McpToolCall`] - Request to invoke a tool
//! - [`McpToolResult`] - Result from tool invocation
//! - [`McpContent`] - Content types (text, image, resource)
//!
//! ## Conversion
//!
//! MCP types implement [`ToInternal`] and [`FromInternal`] traits for
//! seamless conversion to/from UMF's internal hub format.
//!
//! ```rust,ignore
//! use umf::{McpTool, ToInternal, InternalTool};
//!
//! let mcp_tool: McpTool = /* from server */;
//! let internal: InternalTool = mcp_tool.to_internal();
//! ```

mod convert;
mod tool;

pub use tool::{
    McpContent, McpInputSchema, McpTool, McpToolAnnotations, McpToolCall, McpToolResult,
};
