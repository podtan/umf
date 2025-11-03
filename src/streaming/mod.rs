//! Streaming response accumulation for LLM providers.
//!
//! This module provides utilities for accumulating streaming responses from LLM providers,
//! handling both text deltas and tool call deltas with sparse index support.

mod accumulator;
mod types;

pub use accumulator::StreamingAccumulator;
pub use types::{StreamChunk, AccumulatedResponse};

#[cfg(test)]
mod tests;
