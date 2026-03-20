//! Smart Sections: Sections with semantic awareness
//!
//! Smart Sections have:
//! - Intent (what this section accomplishes)
//! - Boundary constraints (what belongs and what doesn't)
//! - Semantic centroid (for similarity)
//! - Density/vacancy tracking

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Vacancy level for a section
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VacancyLevel {
    /// No gaps, section is complete
    Full,
    /// Minor gaps, section is nearly complete
    NearlyFull,
    /// Some gaps, section needs work
    Partial,
    /// Significant gaps, section is incomplete
    Sparse,
    /// Almost empty, section is a stub
    Empty,
}

impl VacancyLevel {
    pub fn from_density(density: u32, max_expected: u32) -> Self {
        let ratio = density as f32 / max_expected as f32;
        match ratio {
            r if r >= 0.9 => Self::Full,
            r if r >= 0.7 => Self::NearlyFull,
            r if r >= 0.4 => Self::Partial,
            r if r >= 0.1 => Self::Sparse,
            _ => Self::Empty,
        }
    }
}

/// A Smart Section with semantic awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSection {
    /// Section ID (ULID of the structure block)
    pub id: ulid::Ulid,

    /// Human-readable intent
    /// Example: "Explain the actor lifecycle in Nexus-WASM"
    pub intent: String,

    /// Boundary constraints (what belongs here)
    /// Example: ["actor lifecycle", "startup", "shutdown", "recovery"]
    pub boundary_constraints: Vec<String>,

    /// Keywords extracted from blocks in section
    pub keywords: Vec<String>,

    /// Semantic centroid (average of block embeddings)
    /// Used for similarity search and gravity hooks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_centroid: Option<Vec<f32>>,

    /// Semantic bloom filter for O(1) similarity approximation
    /// Uses a single u128 for fast intersection calculations
    #[serde(default)]
    pub semantic_bloom: [u128; 1],

    /// Number of blocks in section
    pub density: u32,

    /// Expected number of blocks (from outline)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_density: Option<u32>,

    /// Vacancy level
    pub vacancy: VacancyLevel,

    /// Coherence score (0.0-1.0)
    /// How well blocks fit together semantically
    pub coherence_score: f32,

    /// Gravity hooks (blocks that attract related content)
    #[serde(default)]
    pub gravity_hooks: Vec<ulid::Ulid>,
}

#[allow(dead_code)]
impl SmartSection {
    /// Create a new smart section
    pub fn new(intent: impl Into<String>) -> Self {
        Self {
            id: ulid::Ulid::new(),
            intent: intent.into(),
            boundary_constraints: Vec::new(),
            keywords: Vec::new(),
            semantic_centroid: None,
            semantic_bloom: [0u128; 1],
            density: 0,
            expected_density: None,
            vacancy: VacancyLevel::Empty,
            coherence_score: 0.0,
            gravity_hooks: Vec::new(),
        }
    }

    /// Add a boundary constraint
    pub fn with_boundary(mut self, constraint: impl Into<String>) -> Self {
        self.boundary_constraints.push(constraint.into());
        self
    }

    /// Set expected density
    pub fn with_expected_density(mut self, expected: u32) -> Self {
        self.expected_density = Some(expected);
        self.update_vacancy();
        self
    }

    /// Add a block to the section
    pub fn add_block(&mut self) {
        self.density += 1;
        self.update_vacancy();
    }

    /// Update vacancy level based on density
    fn update_vacancy(&mut self) {
        if let Some(expected) = self.expected_density {
            self.vacancy = VacancyLevel::from_density(self.density, expected);
        } else {
            // If no expected density, assume nearly full if density > 0
            self.vacancy = if self.density > 0 {
                VacancyLevel::NearlyFull
            } else {
                VacancyLevel::Empty
            };
        }
    }

    /// Check if a topic matches boundary constraints
    pub fn matches_boundary(&self, topic: &str) -> bool {
        self.boundary_constraints
            .iter()
            .any(|constraint| topic.to_lowercase().contains(&constraint.to_lowercase()))
    }

    /// Calculate cosine similarity with another centroid
    pub fn similarity_to(&self, other_centroid: &[f32]) -> Option<f32> {
        let centroid = self.semantic_centroid.as_ref()?;
        if centroid.len() != other_centroid.len() {
            return None;
        }

        let dot: f32 = centroid.iter()
            .zip(other_centroid.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm_a: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = other_centroid.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return None;
        }

        Some(dot / (norm_a * norm_b))
    }

    /// Add a gravity hook
    pub fn add_gravity_hook(&mut self, block_id: ulid::Ulid) {
        if !self.gravity_hooks.contains(&block_id) {
            self.gravity_hooks.push(block_id);
        }
    }

    /// Update semantic bloom from a keyword
    ///
    /// Uses a simple hash-based bloom filter where each keyword
    /// sets bits based on its hash.
    pub fn add_keyword_to_bloom(&mut self, keyword: &str) {
        let hash = self.hash_keyword(keyword);
        self.semantic_bloom[0] |= hash;
    }

    /// Update semantic bloom from multiple keywords
    pub fn update_bloom_from_keywords(&mut self, keywords: &[String]) {
        for keyword in keywords {
            self.add_keyword_to_bloom(keyword);
        }
    }

    /// Calculate bloom intersection score with another bloom filter.
    ///
    /// Returns a value between 0.0 and 1.0 representing Jaccard similarity.
    pub fn bloom_intersection(&self, other: &[u128; 1]) -> f32 {
        let intersection = self.semantic_bloom[0] & other[0];
        let union = self.semantic_bloom[0] | other[0];

        if union == 0 {
            return 0.0;
        }

        let intersect_bits = intersection.count_ones() as f32;
        let union_bits = union.count_ones() as f32;

        intersect_bits / union_bits
    }

    /// Hash a keyword to a bloom filter bit position.
    ///
    /// Uses a simple but effective hash that distributes bits
    /// across the 128-bit space.
    fn hash_keyword(&self, keyword: &str) -> u128 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        keyword.hash(&mut hasher);
        let hash = hasher.finish();

        // Spread the hash across 128 bits using multiple hash rounds
        let mut bloom = 0u128;
        for i in 0..4 {
            let mut hasher = DefaultHasher::new();
            let xors = hash ^ ((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            xors.hash(&mut hasher);
            let round_hash = hasher.finish();
            // Use lower 7 bits of hash to get value 0-127
            let bit_pos = (round_hash & 0x7F) as u32;
            bloom |= 1u128 << bit_pos;
        }

        bloom
    }
}

/// Calculate bloom intersection between two bloom filters.
pub fn calculate_bloom_intersection(a: &[u128; 1], b: &[u128; 1]) -> f32 {
    let intersection = a[0] & b[0];
    let union = a[0] | b[0];

    if union == 0 {
        return 0.0;
    }

    let intersect_bits = intersection.count_ones() as f32;
    let union_bits = union.count_ones() as f32;

    intersect_bits / union_bits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_section_creation() {
        let section = SmartSection::new("Explain actor lifecycle")
            .with_boundary("actor")
            .with_boundary("lifecycle")
            .with_expected_density(10);

        assert_eq!(section.intent, "Explain actor lifecycle");
        assert_eq!(section.boundary_constraints, vec!["actor", "lifecycle"]);
        assert_eq!(section.expected_density, Some(10));
    }

    #[test]
    fn test_vacancy_levels() {
        assert_eq!(VacancyLevel::from_density(9, 10), VacancyLevel::Full);
        assert_eq!(VacancyLevel::from_density(7, 10), VacancyLevel::NearlyFull);
        assert_eq!(VacancyLevel::from_density(4, 10), VacancyLevel::Partial);
        assert_eq!(VacancyLevel::from_density(1, 10), VacancyLevel::Sparse);
        assert_eq!(VacancyLevel::from_density(0, 10), VacancyLevel::Empty);
    }

    #[test]
    fn test_boundary_matching() {
        let section = SmartSection::new("Test")
            .with_boundary("actor model");

        assert!(section.matches_boundary("the actor model is great"));
        assert!(section.matches_boundary("Actor Model patterns"));
        assert!(!section.matches_boundary("unrelated topic"));
    }

    #[test]
    fn test_bloom_initialization() {
        let section = SmartSection::new("Test");

        // Initially bloom should be all zeros
        assert_eq!(section.semantic_bloom[0], 0u128);
    }

    #[test]
    fn test_add_keyword_to_bloom() {
        let mut section = SmartSection::new("Test");

        section.add_keyword_to_bloom("rust");
        section.add_keyword_to_bloom("programming");

        // Bloom should have bits set
        assert!(section.semantic_bloom[0] != 0);
    }

    #[test]
    fn test_bloom_intersection() {
        let mut section1 = SmartSection::new("Test");
        let mut section2 = SmartSection::new("Test");

        // Add same keywords
        section1.add_keyword_to_bloom("rust");
        section2.add_keyword_to_bloom("rust");

        // Same bloom should have intersection of 1.0
        let score = section1.bloom_intersection(&section2.semantic_bloom);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_bloom_intersection_different_keywords() {
        let mut section1 = SmartSection::new("Test");
        let mut section2 = SmartSection::new("Test");

        // Add different keywords
        section1.add_keyword_to_bloom("rust");
        section2.add_keyword_to_bloom("python");

        // Different blooms should have intersection less than 1.0
        let score = section1.bloom_intersection(&section2.semantic_bloom);
        assert!(score < 1.0);
    }

    #[test]
    fn test_bloom_intersection_empty() {
        let section1 = SmartSection::new("Test");
        let section2 = SmartSection::new("Test");

        // Two empty blooms should have 0 intersection
        let score = section1.bloom_intersection(&section2.semantic_bloom);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_update_bloom_from_keywords() {
        let mut section = SmartSection::new("Test");

        let keywords = vec!["rust".to_string(), "programming".to_string(), "systems".to_string()];
        section.update_bloom_from_keywords(&keywords);

        // Bloom should have bits set from all keywords
        assert!(section.semantic_bloom[0] != 0);
    }

    #[test]
    fn test_calculate_bloom_intersection_function() {
        let a: [u128; 1] = [0b1010u128];
        let b: [u128; 1] = [0b0110u128];

        let score = calculate_bloom_intersection(&a, &b);
        assert!(score > 0.0);
        assert!(score < 1.0);
    }
}
