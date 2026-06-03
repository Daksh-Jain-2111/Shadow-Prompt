mod adapter;
mod entity;
mod merge;
mod pipeline;
mod rules;

pub use adapter::{MockNER, NERAdapter};
pub use entity::{DetectionSource, EntitySpan};
pub use merge::merge_spans;
pub use pipeline::{mask_pipeline, mask_pipeline_extended};
pub use rules::UserRule;

