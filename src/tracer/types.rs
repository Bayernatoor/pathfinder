//!
//! Core data types for UTXO tracing Engine
//!
//! This module defines the structues returned by the tracer and used internally
//! to representg traced transactions graphs, terminal endpoints and stats.
use serde::Serialize;
use std::collections::HashMap;

use bitcoin::{Address, Amount, OutPoint, Transaction, Txid};

/// Complete result of a UTXO trace operation
///
/// Contains the tx graph, terminal endpoints where tracing stopped
/// and stats about the trace.
///
/// # Fields
///
/// * `roots` - The starting outpoint of the trace
/// * `graph` - HashMap of all TXs discovered during the trace by Txid
/// * `terminals` - Endpoints where the trace terminated (unspent, exchange, etc...)
/// * `stats` - Statistics of the trace
#[derive(Debug, Clone, Serialize)]
pub struct TraceResult {
    pub root: OutPoint,
    pub graph: HashMap<Txid, TransactionNode>,
    pub terminals: Vec<Terminal>,
    pub stats: TraceStats,
}

/// A Transaction node in a traced graph
///
/// Represents a single tx discovered sduring tracing, along with metadata about its position in
/// the trace.
///
/// # Fields
///
/// * `txid` - Transaction ID
/// * `transaction` - full bitcoin::Transaction data
/// * `depth` - depth in the trace tree (0 is the root tx)
#[derive(Debug, Clone, Serialize)]
pub struct TransactionNode {
    pub txid: Txid,
    pub transaction: Transaction,
    pub depth: usize,
}

/// an output within a traced tx
///
/// Represents a singel output and whether or how it was spent in the trace
///
/// # Fields
///
/// * `outpoint` - The output reference (txid::vout)
/// * `value` - Amount in sats
/// * `address` - Address if available
/// * `spent_by` - Transaction that spent this output (if available)
/// * `terminal_reason` - Why the trace terminated at this output (if appliable)
#[derive(Debug, Clone, Serialize)]
pub struct Output {
    pub outpoint: OutPoint,
    pub value: Amount,
    pub address: Option<Address>,
    pub spent_by: Option<Txid>,
    pub terminal_reason: Option<TerminalReason>,
}

/// A terminal endpoint where tracing stopped.
///
/// Represents a final output in the trace that matched a termination condition
///
/// # Fields
///
/// * `outpoint` - The terminal output refernce
/// * `reason` - Why the trace stopped here
/// * `value` - Amount in sats
#[derive(Debug, Clone, Serialize)]
pub struct Terminal {
    pub outpoint: OutPoint,
    pub reason: TerminalReason,
    pub value: Amount,
}

/// Reason why a trace terminated at a particular output
///
/// Indicates the condition that caused the tracer to stop following a path.
#[derive(Debug, Clone, Serialize)]
pub enum TerminalReason {
    Unspent,
    MaxDepthReached,
    BelowMinValue,
    Exchange(String),
    Mixer(String),
    Sanctioned(String),
    Other(String),
    DataUnavailable,
}

/// Summary statistics about a completed trace.
///
///  # Fields
///
/// * `total_transactions` - Number of unique TXs in the graph
/// * `total_value_traced` - Sum of all output values discovered
/// * `max_depth_reached` - Max depth reached in the trace
/// * `terminal_count` - Number of terminal endpoints found
#[derive(Debug, Clone, Serialize)]
pub struct TraceStats {
    pub total_transactions: usize,
    pub total_value_traced: Amount,
    pub max_depth_reached: usize,
    pub terminal_count: usize,
}
