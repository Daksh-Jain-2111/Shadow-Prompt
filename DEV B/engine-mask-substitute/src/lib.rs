//! ShadowPrompt masking + substitution: realistic replacements, in-memory session maps,
//! back-to-front safe masking, and unmasking of model replies.
//!
//! ## Example
//!
//! ```
//! use engine_mask_substitute::{mask_text, unmask_text, Detection, SessionMap};
//!
//! let input = "Rahul's PAN is ABCDE1234F and phone is 9876543210";
//! let detections = vec![
//!     Detection {
//!         entity_type: "NAME".into(),
//!         start: 0,
//!         end: 5,
//!         value: "Rahul".into(),
//!         is_valid: true,
//!     },
//!     Detection {
//!         entity_type: "PAN".into(),
//!         start: 15,
//!         end: 25,
//!         value: "ABCDE1234F".into(),
//!         is_valid: true,
//!     },
//!     Detection {
//!         entity_type: "PHONE".into(),
//!         start: 39,
//!         end: 49,
//!         value: "9876543210".into(),
//!         is_valid: true,
//!     },
//! ];
//!
//! let mut session = SessionMap::new();
//! let masked = mask_text(input, &detections, &mut session);
//! assert!(!masked.contains("ABCDE1234F"));
//! assert!(!masked.contains("9876543210"));
//!
//! let pan_masked = session.get_or_create("PAN", "ABCDE1234F");
//! let name_masked = session.get_or_create("NAME", "Rahul");
//! let model_reply = format!("{name_masked} should verify {pan_masked}");
//! let restored = unmask_text(&model_reply, &session);
//! assert!(restored.contains("Rahul"));
//! assert!(restored.contains("ABCDE1234F"));
//! ```

mod generators;
mod masking;
mod session_map;
mod unmasking;

pub use generators::generate_substitute;
pub use masking::mask_text;
pub use session_map::SessionMap;
pub use unmasking::{unmask_exact_then_fuzzy, unmask_text, FuzzySpan, UnmaskFuzzyResult};

/// Match layout used by `engine-detect-regex` so callers can share detections.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    pub entity_type: String,
    pub start: usize,
    pub end: usize,
    pub value: String,
    pub is_valid: bool,
}
