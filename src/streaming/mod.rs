//! Streaming response accumulation for LLM providers.
//!
//! This module provides utilities for accumulating streaming responses from LLM providers,
//! handling both text deltas and tool call deltas with sparse index support.

mod accumulator;
mod types;

pub(crate) use accumulator::StreamingAccumulator;
pub(crate) use types::{StreamChunk, AccumulatedResponse};

#[cfg(test)]
mod tests;
