//! Rule execution engine and violation aggregation

pub mod aggregator;
pub mod executor;
pub mod file_walker;

pub use aggregator::{AggregationResult, RuleRegionStatus, ViolationAggregator};
pub use executor::{ExecutionEngine, ExecutionResult};
