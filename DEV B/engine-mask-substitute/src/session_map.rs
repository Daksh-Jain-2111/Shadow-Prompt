//! In-memory forward and reverse maps. Never persisted.

use crate::generators::{generate_substitute, generate_substitute_date, normalize_entity_type};
use rand::Rng;
use std::collections::HashMap;

fn composite_key(entity_type: &str, original: &str) -> String {
    let t = normalize_entity_type(entity_type);
    format!("{}\x1f{}", t.as_ref(), original)
}

pub struct SessionMap {
    pub forward: HashMap<String, String>,
    pub reverse: HashMap<String, String>,
    /// Normalized entity type for each substitute token (fuzzy restore must stay within type).
    pub substitute_entity: HashMap<String, String>,
    /// One shift applied to every DATE/DATES value in this session.
    date_offset_days: Option<i32>,
}

impl SessionMap {
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            reverse: HashMap::new(),
            substitute_entity: HashMap::new(),
            date_offset_days: None,
        }
    }

    /// Return an existing substitute or create one and register both directions.
    pub fn get_or_create(&mut self, entity_type: &str, original: &str) -> String {
        let key = composite_key(entity_type, original);
        if let Some(existing) = self.forward.get(&key) {
            return existing.clone();
        }

        let norm = normalize_entity_type(entity_type);
        let substitute = if norm.as_ref() == "DATE" {
            let offset = self
                .date_offset_days
                .get_or_insert_with(|| rand::thread_rng().gen_range(-380..380));
            generate_substitute_date(original, *offset)
        } else {
            generate_substitute(norm.as_ref(), original)
        };

        self.insert_mapping(entity_type, original, &substitute);
        substitute
    }

    /// Explicitly record a mapping (overwrites prior forward key; reverse uses latest wins).
    pub fn insert_mapping(&mut self, entity_type: &str, original: &str, substitute: &str) {
        let key = composite_key(entity_type, original);
        let norm = normalize_entity_type(entity_type).into_owned();
        self.forward.insert(key, substitute.to_string());
        self.reverse.insert(substitute.to_string(), original.to_string());
        self.substitute_entity
            .insert(substitute.to_string(), norm);
    }

    /// Zeroize in-memory maps (STM). Forward keys and values are cleared with zeroization.
    pub fn secure_wipe(&mut self) {
        use zeroize::Zeroize;
        for (mut k, mut v) in self.forward.drain() {
            k.zeroize();
            v.zeroize();
        }
        for (mut k, mut v) in self.reverse.drain() {
            k.zeroize();
            v.zeroize();
        }
        for (mut k, mut v) in self.substitute_entity.drain() {
            k.zeroize();
            v.zeroize();
        }
        self.date_offset_days = None;
    }

    /// Resolve a masked fragment back to its original secret.
    pub fn reverse_lookup(&self, substitute: &str) -> Option<String> {
        self.reverse.get(substitute).cloned()
    }
}

impl Default for SessionMap {
    fn default() -> Self {
        Self::new()
    }
}
