//! Embedding Generator: Generate semantic embeddings for blocks
//!
//! This module provides hash-based pseudo-embeddings for MVP.
//! For production, integrate with sentence-transformers or external embedding services.

use crate::models::Block;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Embedding dimension for the pseudo-embedding scheme
pub const EMBEDDING_DIM: usize = 384;

/// Default similarity threshold for link suggestions
pub const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.5;

/// Embedding generator for semantic similarity
///
/// For MVP, uses hash-based pseudo-embeddings that capture:
/// - Word frequency patterns
/// - Character-level features
/// - Structural characteristics
///
/// Production integration points:
/// - OpenAI `text-embedding-3-small` (384 dimensions)
/// - Local `all-MiniLM-L6-v2` model
/// - SurrealDB native vector search (MTREE index)
#[derive(Debug, Clone)]
pub struct EmbeddingGenerator {
    // Reserved for future external embedding service integration
    _private: (),
}

#[allow(dead_code)]
impl EmbeddingGenerator {
    /// Create a new embedding generator
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Generate embedding for a block
    ///
    /// Uses hash-based pseudo-embeddings that capture text structure.
    /// For production, replace with actual embedding model integration.
    pub async fn embed(&self, block: &Block) -> anyhow::Result<Vec<f32>> {
        let text = self.prepare_text(block);
        Ok(self.hash_to_embedding(&text))
    }

    /// Generate embedding for raw text content
    pub async fn embed_text(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let prepared = self.prepare_text_raw(text);
        Ok(self.hash_to_embedding(&prepared))
    }

    /// Prepare text from block for embedding
    fn prepare_text(&self, block: &Block) -> String {
        let mut parts = Vec::new();

        // Include title for context
        if !block.title.is_empty() && block.title != "Fleeting Note" {
            parts.push(block.title.clone());
        }

        // Include content
        parts.push(block.content.clone());

        // Include tags for semantic context
        if !block.tags.is_empty() {
            parts.push(block.tags.join(" "));
        }

        parts.join(" ")
    }

    /// Prepare raw text for embedding
    fn prepare_text_raw(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .to_lowercase()
    }

    /// Convert hash to fixed-size embedding vector
    ///
    /// Uses multiple hash functions to create a pseudo-embedding
    /// that captures word frequency and character patterns.
    fn hash_to_embedding(&self, text: &str) -> Vec<f32> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let char_counts = self.char_frequency(text);
        let word_counts = self.word_frequency(&words);

        let mut embedding = vec![0.0f32; EMBEDDING_DIM];

        // Component 1: Character frequency distribution (first 128 dims)
        for (i, (_char, count)) in char_counts.iter().take(128).enumerate() {
            embedding[i] = *count as f32;
        }

        // Component 2: Word hash distribution (next 128 dims)
        for (i, (_, count)) in word_counts.iter().take(128).enumerate() {
            embedding[128 + i] = *count as f32;
        }

        // Component 3: Structural features (next 64 dims)
        let text_len = text.len() as f32;
        embedding[256] = text_len.min(10000.0) / 10000.0;
        embedding[257] = words.len() as f32 / 100.0;
        embedding[258] = if words.is_empty() { 0.0 } else { text_len / words.len() as f32 } / 100.0;
        embedding[259] = self.unique_word_ratio(&words);
        embedding[260] = self.avg_word_length(&words) / 20.0;
        embedding[261] = self.question_marks(text) as f32 / 10.0;
        embedding[262] = self.exclamation_marks(text) as f32 / 10.0;
        embedding[263] = self.number_count(text) as f32 / 20.0;
        embedding[264] = self.uppercase_ratio(text);
        embedding[265] = self.sentence_count(text) as f32 / 20.0;
        embedding[266] = self.avg_sentence_length(text) / 50.0;
        embedding[267] = self.paragraph_count(text) as f32 / 10.0;
        embedding[268] = self.code_indicator(text);
        embedding[269] = self.link_indicator(text);
        embedding[270] = self.list_indicator(text);
        embedding[271] = self.heading_indicator(text);
        embedding[272] = self.blockquote_indicator(text);
        embedding[273] = self.table_indicator(text);
        embedding[274] = self.image_indicator(text);
        embedding[275] = self.hashtag_count(text) as f32 / 20.0;
        embedding[276] = self.mention_count(text) as f32 / 10.0;
        embedding[277] = self.punctuation_ratio(text);
        embedding[278] = self.digit_ratio(text);
        embedding[279] = self.alpha_ratio(text);
        embedding[280] = self.whitespace_ratio(text);
        embedding[281] = self.special_char_ratio(text);
        embedding[282] = self.capitalized_word_ratio(&words);
        embedding[283] = self.short_word_ratio(&words);
        embedding[284] = self.long_word_ratio(&words);
        embedding[285] = self.unique_char_ratio(text);
        embedding[286] = self.vowel_ratio(text);
        embedding[287] = self.consonant_ratio(text);
        embedding[288] = self.letter_ratio(text);
        embedding[289] = self.avg_word_frequency(&word_counts);
        embedding[290] = self.word_diversity(&words);
        embedding[291] = self.type_token_ratio(&words);
        embedding[292] = self.lexical_density(&words);
        embedding[293] = self.stop_word_ratio(&words);
        embedding[294] = self.noun_indicator(text);
        embedding[295] = self.verb_indicator(text);
        embedding[296] = self.adjective_indicator(text);
        embedding[297] = self.adverb_indicator(text);
        embedding[298] = self.pronoun_indicator(text);
        embedding[299] = self.preposition_indicator(text);
        embedding[300] = self.conjunction_indicator(text);
        embedding[301] = self.interjection_indicator(text);
        embedding[302] = self.determiner_indicator(text);
        embedding[303] = self.starting_article(text);
        embedding[304] = self.starting_question(text);
        embedding[305] = self.starting_exclamation(text);
        embedding[306] = self.ends_with_question(text);
        embedding[307] = self.ends_with_exclamation(text);
        embedding[308] = self.ends_with_period(text);
        embedding[309] = self.contains_number(text);
        embedding[310] = self.contains_date(text);
        embedding[311] = self.contains_time(text);
        embedding[312] = self.contains_url(text);
        embedding[313] = self.contains_email(text);
        embedding[314] = self.contains_phone(text);
        embedding[315] = self.contains_citation(text);
        embedding[316] = self.contains_footnote(text);
        embedding[317] = self.bullet_point_style(text);
        embedding[318] = self.numbered_list_style(text);
        embedding[319] = self.mixed_list_style(text);

        // Component 4: Hash-based distribution (remaining dims)
        // Uses multiple hash seeds to create pseudo-random but deterministic distribution
        for i in 0..64 {
            let seed = (i as u64).wrapping_mul(0x9e3779b97f4a7c15);
            let hash = self.custom_hash(text, seed);
            embedding[320 + i] = (hash % 1000) as f32 / 1000.0;
        }

        // Normalize the embedding
        self.normalize(&embedding)
    }

    /// Custom hash function with seed
    fn custom_hash(&self, text: &str, seed: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(seed);
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate character frequency map
    fn char_frequency(&self, text: &str) -> Vec<(char, usize)> {
        let mut freq = std::collections::HashMap::new();
        for c in text.chars() {
            if c.is_alphanumeric() || c.is_whitespace() {
                *freq.entry(c).or_insert(0) += 1;
            }
        }
        let mut pairs: Vec<_> = freq.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs
    }

    /// Calculate word frequency map
    fn word_frequency(&self, words: &[&str]) -> Vec<(String, usize)> {
        let mut freq = std::collections::HashMap::new();
        for word in words {
            let normalized = word.to_lowercase();
            if normalized.len() > 2 {
                *freq.entry(normalized).or_insert(0) += 1;
            }
        }
        let mut pairs: Vec<_> = freq.into_iter().collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs
    }

    /// Unique word ratio
    fn unique_word_ratio(&self, words: &[&str]) -> f32 {
        if words.is_empty() {
            return 0.0;
        }
        let unique: std::collections::HashSet<_> = words.iter().collect();
        unique.len() as f32 / words.len() as f32
    }

    /// Average word length
    fn avg_word_length(&self, words: &[&str]) -> f32 {
        if words.is_empty() {
            return 0.0;
        }
        let total: usize = words.iter().map(|w| w.len()).sum();
        total as f32 / words.len() as f32
    }

    /// Count question marks
    fn question_marks(&self, text: &str) -> usize {
        text.chars().filter(|c| *c == '?').count()
    }

    /// Count exclamation marks
    fn exclamation_marks(&self, text: &str) -> usize {
        text.chars().filter(|c| *c == '!').count()
    }

    /// Count numbers in text
    fn number_count(&self, text: &str) -> usize {
        text.split_whitespace()
            .filter(|w| w.chars().all(|c| c.is_numeric() || c == '.' || c == ','))
            .count()
    }

    /// Uppercase character ratio
    fn uppercase_ratio(&self, text: &str) -> f32 {
        let total = text.chars().filter(|c| c.is_alphabetic()).count();
        if total == 0 {
            return 0.0;
        }
        let upper = text.chars().filter(|c| c.is_uppercase()).count();
        upper as f32 / total as f32
    }

    /// Sentence count
    fn sentence_count(&self, text: &str) -> usize {
        let mut count = 0;
        let mut prev_was_end = true;
        for c in text.chars() {
            if c == '.' || c == '!' || c == '?' {
                if !prev_was_end {
                    count += 1;
                    prev_was_end = true;
                }
            } else if c.is_alphanumeric() {
                prev_was_end = false;
            }
        }
        if !prev_was_end {
            count += 1;
        }
        count.max(1)
    }

    /// Average sentence length
    fn avg_sentence_length(&self, text: &str) -> f32 {
        let sentences = self.sentence_count(text);
        if sentences == 0 {
            return 0.0;
        }
        text.split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.split_whitespace().count())
            .sum::<usize>() as f32
            / sentences as f32
    }

    /// Paragraph count
    fn paragraph_count(&self, text: &str) -> usize {
        text.split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .count()
            .max(1)
    }

    /// Code indicator (contains code-like patterns)
    fn code_indicator(&self, text: &str) -> f32 {
        let code_patterns = [
            "```", "fn ", "let ", "pub ", "impl ", "struct ", "enum ", "match ",
            "if ", "for ", "while ", "return ", "=>", "->", "::", "use ",
            "const ", "static ", "mut ", "&", "|*", "#[", "*/", "//", "/*",
        ];
        let lower = text.to_lowercase();
        let count = code_patterns.iter().filter(|p| lower.contains(&p.to_lowercase())).count();
        (count as f32 / code_patterns.len() as f32).min(1.0)
    }

    /// Link indicator (contains URL or markdown links)
    fn link_indicator(&self, text: &str) -> f32 {
        let lower = text.to_lowercase();
        let has_url = lower.contains("http://") || lower.contains("https://");
        let has_markdown_link = lower.contains("](") || lower.contains("][");
        let has_wiki_link = lower.contains("[[");
        let count = [has_url, has_markdown_link, has_wiki_link].iter().filter(|&&x| x).count();
        if count > 0 { 1.0 } else { 0.0 }
    }

    /// List indicator
    fn list_indicator(&self, text: &str) -> f32 {
        let has_bullet = text.lines().any(|l| l.trim().starts_with('-') || l.trim().starts_with('*'));
        let has_numbered = text.lines().any(|l| {
            let trimmed = l.trim();
            trimmed.len() > 1 && trimmed.chars().next().unwrap().is_numeric() && trimmed.contains('.')
        });
        if has_bullet || has_numbered { 1.0 } else { 0.0 }
    }

    /// Heading indicator
    fn heading_indicator(&self, text: &str) -> f32 {
        let has_heading = text.lines().any(|l| {
            let trimmed = l.trim();
            (trimmed.starts_with('#') && !trimmed.starts_with("##")) ||
            (!trimmed.is_empty() && trimmed == trimmed.to_uppercase() && trimmed.len() < 50)
        });
        if has_heading { 1.0 } else { 0.0 }
    }

    /// Blockquote indicator
    fn blockquote_indicator(&self, text: &str) -> f32 {
        let has_quote = text.lines().any(|l| l.trim().starts_with('>'));
        if has_quote { 1.0 } else { 0.0 }
    }

    /// Table indicator
    fn table_indicator(&self, text: &str) -> f32 {
        let lines: Vec<_> = text.lines().collect();
        if lines.len() < 2 {
            return 0.0;
        }
        let has_separator = lines.iter().any(|l| l.contains("---") || l.contains("|---|"));
        let has_pipes = lines.iter().filter(|l| l.contains('|')).count() >= 2;
        if has_separator || has_pipes { 1.0 } else { 0.0 }
    }

    /// Image indicator
    fn image_indicator(&self, text: &str) -> f32 {
        let lower = text.to_lowercase();
        let has_image = lower.contains("![") || lower.contains(".png") ||
                        lower.contains(".jpg") || lower.contains(".jpeg") ||
                        lower.contains(".gif") || lower.contains(".svg");
        if has_image { 1.0 } else { 0.0 }
    }

    /// Hashtag count
    fn hashtag_count(&self, text: &str) -> usize {
        text.split_whitespace()
            .filter(|w| w.starts_with('#'))
            .count()
    }

    /// Mention count
    fn mention_count(&self, text: &str) -> usize {
        text.split_whitespace()
            .filter(|w| w.starts_with('@'))
            .count()
    }

    /// Punctuation ratio
    fn punctuation_ratio(&self, text: &str) -> f32 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let punct = text.chars().filter(|c| c.is_ascii_punctuation()).count();
        punct as f32 / total as f32
    }

    /// Digit ratio
    fn digit_ratio(&self, text: &str) -> f32 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let digits = text.chars().filter(|c| c.is_numeric()).count();
        digits as f32 / total as f32
    }

    /// Alpha ratio
    fn alpha_ratio(&self, text: &str) -> f32 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let alpha = text.chars().filter(|c| c.is_alphabetic()).count();
        alpha as f32 / total as f32
    }

    /// Whitespace ratio
    fn whitespace_ratio(&self, text: &str) -> f32 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let ws = text.chars().filter(|c| c.is_whitespace()).count();
        ws as f32 / total as f32
    }

    /// Special character ratio
    fn special_char_ratio(&self, text: &str) -> f32 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let special = text.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
        special as f32 / total as f32
    }

    /// Capitalized word ratio
    fn capitalized_word_ratio(&self, words: &[&str]) -> f32 {
        if words.is_empty() {
            return 0.0;
        }
        let capitalized = words.iter().filter(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.is_uppercase() && chars.any(|c| c.is_lowercase()),
                None => false,
            }
        }).count();
        capitalized as f32 / words.len() as f32
    }

    /// Short word ratio (words <= 3 chars)
    fn short_word_ratio(&self, words: &[&str]) -> f32 {
        if words.is_empty() {
            return 0.0;
        }
        let short = words.iter().filter(|w| w.len() <= 3).count();
        short as f32 / words.len() as f32
    }

    /// Long word ratio (words >= 10 chars)
    fn long_word_ratio(&self, words: &[&str]) -> f32 {
        if words.is_empty() {
            return 0.0;
        }
        let long = words.iter().filter(|w| w.len() >= 10).count();
        long as f32 / words.len() as f32
    }

    /// Unique character ratio
    fn unique_char_ratio(&self, text: &str) -> f32 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let unique: std::collections::HashSet<_> = text.chars().collect();
        unique.len() as f32 / total as f32
    }

    /// Vowel ratio
    fn vowel_ratio(&self, text: &str) -> f32 {
        let vowels = ['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U'];
        let total = text.chars().filter(|c| c.is_alphabetic()).count();
        if total == 0 {
            return 0.0;
        }
        let vowel_count = text.chars().filter(|c| vowels.contains(c)).count();
        vowel_count as f32 / total as f32
    }

    /// Consonant ratio
    fn consonant_ratio(&self, text: &str) -> f32 {
        let vowels = ['a', 'e', 'i', 'o', 'u', 'A', 'E', 'I', 'O', 'U'];
        let total = text.chars().filter(|c| c.is_alphabetic()).count();
        if total == 0 {
            return 0.0;
        }
        let consonant_count = text.chars().filter(|c| c.is_alphabetic() && !vowels.contains(c)).count();
        consonant_count as f32 / total as f32
    }

    /// Letter ratio
    fn letter_ratio(&self, text: &str) -> f32 {
        let total = text.chars().count();
        if total == 0 {
            return 0.0;
        }
        let letters = text.chars().filter(|c| c.is_alphabetic()).count();
        letters as f32 / total as f32
    }

    /// Average word frequency
    fn avg_word_frequency(&self, word_counts: &[(String, usize)]) -> f32 {
        if word_counts.is_empty() {
            return 0.0;
        }
        let total: usize = word_counts.iter().map(|(_, c)| c).sum();
        total as f32 / word_counts.len() as f32 / 100.0
    }

    /// Word diversity (unique words / total words)
    fn word_diversity(&self, words: &[&str]) -> f32 {
        self.unique_word_ratio(words)
    }

    /// Type-token ratio (vocabulary richness)
    fn type_token_ratio(&self, words: &[&str]) -> f32 {
        self.unique_word_ratio(words)
    }

    /// Lexical density (content words / total words)
    fn lexical_density(&self, words: &[&str]) -> f32 {
        if words.is_empty() {
            return 0.0;
        }
        // Approximate content words as words > 5 chars
        let content = words.iter().filter(|w| w.len() > 5).count();
        content as f32 / words.len() as f32
    }

    /// Stop word ratio (common words like "the", "is", "at")
    fn stop_word_ratio(&self, words: &[&str]) -> f32 {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "be", "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "can", "this", "that", "these", "those",
            "i", "you", "he", "she", "it", "we", "they", "what", "which", "who",
            "when", "where", "why", "how", "all", "each", "every", "both", "few",
            "more", "most", "other", "some", "such", "no", "nor", "not", "only",
            "own", "same", "so", "than", "too", "very", "just", "also", "now",
        ];
        if words.is_empty() {
            return 0.0;
        }
        let stop = words.iter()
            .map(|w| w.to_lowercase())
            .filter(|w| stop_words.contains(&w.as_str()))
            .count();
        stop as f32 / words.len() as f32
    }

    /// Noun indicator (heuristic: words starting with uppercase or ending with common noun suffixes)
    fn noun_indicator(&self, text: &str) -> f32 {
        let noun_suffixes = ["tion", "sion", "ness", "ment", "ity", "ance", "ence", "er", "or"];
        let lower = text.to_lowercase();
        let mut score: f32 = 0.0;

        for word in lower.split_whitespace() {
            if word.len() > 3 {
                let first_char = word.chars().next().unwrap();
                if first_char.is_uppercase() && word.len() > 2 && text.contains(word) {
                    score += 0.3;
                }
                for suffix in &noun_suffixes {
                    if word.ends_with(suffix) {
                        score += 0.2;
                        break;
                    }
                }
            }
        }

        score.min(1.0)
    }

    /// Verb indicator (heuristic: common verb patterns)
    fn verb_indicator(&self, text: &str) -> f32 {
        let verb_suffixes = ["ate", "ize", "ify", "en", "ed", "ing"];
        let lower = text.to_lowercase();
        let mut score: f32 = 0.0;

        for word in lower.split_whitespace() {
            for suffix in &verb_suffixes {
                if word.ends_with(suffix) && word.len() > 4 {
                    score += 0.3;
                    break;
                }
            }
        }

        score.min(1.0)
    }

    /// Adjective indicator
    fn adjective_indicator(&self, text: &str) -> f32 {
        let adj_suffixes = ["ous", "ive", "ful", "less", "able", "ible", "al", "ical", "ous"];
        let lower = text.to_lowercase();
        let mut score: f32 = 0.0;

        for word in lower.split_whitespace() {
            for suffix in &adj_suffixes {
                if word.ends_with(suffix) && word.len() > 4 {
                    score += 0.3;
                    break;
                }
            }
        }

        score.min(1.0)
    }

    /// Adverb indicator
    fn adverb_indicator(&self, text: &str) -> f32 {
        let lower = text.to_lowercase();
        let count = lower.split_whitespace()
            .filter(|w| w.ends_with("ly") && w.len() > 3)
            .count();
        (count as f32 / 10.0).min(1.0)
    }

    /// Pronoun indicator
    fn pronoun_indicator(&self, text: &str) -> f32 {
        let pronouns = ["i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
                        "my", "your", "his", "her", "its", "our", "their", "mine", "yours", "hers", "ours", "theirs"];
        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let count = words.iter().filter(|w| pronouns.contains(w)).count();
        (count as f32 / words.len() as f32).min(1.0)
    }

    /// Preposition indicator
    fn preposition_indicator(&self, text: &str) -> f32 {
        let prepositions = ["in", "on", "at", "by", "for", "with", "about", "against", "between",
                           "into", "through", "during", "before", "after", "above", "below", "to",
                           "from", "up", "down", "over", "under", "again", "further", "then", "once"];
        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let count = words.iter().filter(|w| prepositions.contains(w)).count();
        (count as f32 / words.len() as f32).min(1.0)
    }

    /// Conjunction indicator
    fn conjunction_indicator(&self, text: &str) -> f32 {
        let conjunctions = ["and", "but", "or", "nor", "for", "yet", "so", "although", "because",
                           "if", "when", "where", "while", "since", "unless", "until", "before", "after"];
        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let count = words.iter().filter(|w| conjunctions.contains(w)).count();
        (count as f32 / words.len() as f32).min(1.0)
    }

    /// Interjection indicator
    fn interjection_indicator(&self, text: &str) -> f32 {
        let interjections = ["oh", "wow", "hey", "hi", "hello", "bye", "yeah", "yes", "no", "ok",
                            "okay", "hmm", "aha", "alas", "brr", "oops", "ouch", "ugh", "yay", "yikes"];
        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let count = words.iter().filter(|w| interjections.contains(w)).count();
        (count as f32 / words.len() as f32).min(1.0)
    }

    /// Determiner indicator
    fn determiner_indicator(&self, text: &str) -> f32 {
        let determiners = ["a", "an", "the", "this", "that", "these", "those", "my", "your", "his",
                          "her", "its", "our", "their", "some", "any", "no", "every", "each"];
        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }
        let count = words.iter().filter(|w| determiners.contains(w)).count();
        (count as f32 / words.len() as f32).min(1.0)
    }

    /// Starts with article
    fn starting_article(&self, text: &str) -> f32 {
        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        if let Some(first) = words.first() && ["a", "an", "the"].contains(first) {
            return 1.0;
        }
        0.0
    }

    /// Starts with question word
    fn starting_question(&self, text: &str) -> f32 {
        let question_words = ["what", "who", "where", "when", "why", "how", "which", "whom", "whose"];
        let lower = text.to_lowercase();
        let words: Vec<_> = lower.split_whitespace().collect();
        if let Some(first) = words.first() && question_words.contains(first) {
            return 1.0;
        }
        0.0
    }

    /// Starts with exclamation
    fn starting_exclamation(&self, text: &str) -> f32 {
        let trimmed = text.trim();
        if trimmed.starts_with('!') || trimmed.starts_with("wow") || trimmed.starts_with("oh") {
            return 1.0;
        }
        0.0
    }

    /// Ends with question mark
    fn ends_with_question(&self, text: &str) -> f32 {
        let trimmed = text.trim();
        if trimmed.ends_with('?') {
            return 1.0;
        }
        0.0
    }

    /// Ends with exclamation
    fn ends_with_exclamation(&self, text: &str) -> f32 {
        let trimmed = text.trim();
        if trimmed.ends_with('!') {
            return 1.0;
        }
        0.0
    }

    /// Ends with period
    fn ends_with_period(&self, text: &str) -> f32 {
        let trimmed = text.trim();
        if trimmed.ends_with('.') {
            return 1.0;
        }
        0.0
    }

    /// Contains number pattern
    fn contains_number(&self, text: &str) -> f32 {
        if text.split_whitespace().any(|w| w.chars().any(|c| c.is_numeric())) {
            return 1.0;
        }
        0.0
    }

    /// Contains date pattern
    fn contains_date(&self, text: &str) -> f32 {
        let date_patterns = [
            r"\d{4}-\d{2}-\d{2}",  // ISO date
            r"\d{1,2}/\d{1,2}/\d{2,4}",  // US date
            r"\d{1,2}\.\d{1,2}\.\d{2,4}",  // EU date
        ];
        let lower = text.to_lowercase();
        for pattern in &date_patterns {
            if let Ok(re) = regex::Regex::new(pattern) && re.is_match(&lower) {
                return 1.0;
            }
        }
        0.0
    }

    /// Contains time pattern
    fn contains_time(&self, text: &str) -> f32 {
        let time_patterns = [
            r"\d{1,2}:\d{2}",  // 12:30
            r"\d{1,2}:\d{2}:\d{2}",  // 12:30:45
            r"\d{1,2}\s*(am|pm|AM|PM)",  // 12pm
        ];
        let lower = text.to_lowercase();
        for pattern in &time_patterns {
            if let Ok(re) = regex::Regex::new(pattern) && re.is_match(&lower) {
                return 1.0;
            }
        }
        0.0
    }

    /// Contains URL
    fn contains_url(&self, text: &str) -> f32 {
        let url_patterns = [
            r"https?://[^\s]+",
            r"www\.[^\s]+",
        ];
        for pattern in &url_patterns {
            if let Ok(re) = regex::Regex::new(pattern) && re.is_match(text) {
                return 1.0;
            }
        }
        0.0
    }

    /// Contains email
    fn contains_email(&self, text: &str) -> f32 {
        let email_pattern = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";
        if let Ok(re) = regex::Regex::new(email_pattern) && re.is_match(text) {
            return 1.0;
        }
        0.0
    }

    /// Contains phone number
    fn contains_phone(&self, text: &str) -> f32 {
        let phone_patterns = [
            r"\+?\d{1,3}[-.\s]?\(?\d{1,4}\)?[-.\s]?\d{1,4}[-.\s]?\d{1,9}",
            r"\(\d{3}\)\s*\d{3}[-.\s]?\d{4}",
        ];
        for pattern in &phone_patterns {
            if let Ok(re) = regex::Regex::new(pattern) && re.is_match(text) {
                return 1.0;
            }
        }
        0.0
    }

    /// Contains citation (academic style)
    fn contains_citation(&self, text: &str) -> f32 {
        let citation_patterns = [
            r"\([^)]*\d{4}[^)]*\)",  // (Author, 2020)
            r"\[\d+\]",  // [1]
            r"\[\w+\s+et\s+al\.?\]",  // [Smith et al.]
        ];
        for pattern in &citation_patterns {
            if let Ok(re) = regex::Regex::new(pattern) && re.is_match(text) {
                return 1.0;
            }
        }
        0.0
    }

    /// Contains footnote reference
    fn contains_footnote(&self, text: &str) -> f32 {
        if text.contains("^[") || text.contains("^:") || text.contains("^note") {
            return 1.0;
        }
        0.0
    }

    /// Bullet point list style
    fn bullet_point_style(&self, text: &str) -> f32 {
        let lines: Vec<_> = text.lines().collect();
        let bullet_lines = lines.iter()
            .filter(|l| {
                let trimmed = l.trim();
                trimmed.starts_with('-') || trimmed.starts_with('*') || trimmed.starts_with('•')
            })
            .count();
        if lines.is_empty() {
            return 0.0;
        }
        (bullet_lines as f32 / lines.len() as f32).min(1.0)
    }

    /// Numbered list style
    fn numbered_list_style(&self, text: &str) -> f32 {
        let lines: Vec<_> = text.lines().collect();
        let numbered_lines = lines.iter()
            .filter(|l| {
                let trimmed = l.trim();
                trimmed.len() > 1 &&
                trimmed.chars().next().unwrap().is_numeric() &&
                (trimmed.contains('.') || trimmed.contains(')'))
            })
            .count();
        if lines.is_empty() {
            return 0.0;
        }
        (numbered_lines as f32 / lines.len() as f32).min(1.0)
    }

    /// Mixed list style (both bullets and numbers)
    fn mixed_list_style(&self, text: &str) -> f32 {
        let has_bullet = self.bullet_point_style(text) > 0.0;
        let has_numbered = self.numbered_list_style(text) > 0.0;
        if has_bullet && has_numbered {
            1.0
        } else {
            0.0
        }
    }

    /// Normalize embedding vector to unit length
    fn normalize(&self, embedding: &[f32]) -> Vec<f32> {
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm == 0.0 {
            return embedding.to_vec();
        }
        embedding.iter().map(|x| x / norm).collect()
    }

    /// Calculate cosine similarity between two embeddings
    ///
    /// Returns a value between -1.0 (opposite) and 1.0 (identical),
    /// with 0.0 indicating orthogonality.
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    /// Calculate euclidean distance between two embeddings
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return f32::MAX;
        }

        let sum_sq: f32 = a.iter().zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum();

        sum_sq.sqrt()
    }

    /// Find top-k most similar embeddings
    pub fn top_k_similar(query: &[f32], candidates: &[Vec<f32>], k: usize) -> Vec<(usize, f32)> {
        let mut similarities: Vec<(usize, f32)> = candidates.iter()
            .enumerate()
            .map(|(i, emb)| (i, Self::cosine_similarity(query, emb)))
            .collect();

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        similarities.into_iter().take(k).collect()
    }
}

impl Default for EmbeddingGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_block() -> Block {
        Block::permanent("Rust Programming", "Rust is a systems programming language focused on safety and performance.")
            .with_tag("rust")
            .with_tag("programming")
    }

    #[tokio::test]
    async fn test_embed_block() {
        let generator = EmbeddingGenerator::new();
        let block = create_test_block();

        let embedding = generator.embed(&block).await.unwrap();

        assert_eq!(embedding.len(), EMBEDDING_DIM);
        // Check that embedding is normalized
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_embed_text() {
        let generator = EmbeddingGenerator::new();

        let embedding1 = generator.embed_text("Hello world").await.unwrap();
        let embedding2 = generator.embed_text("Hello world").await.unwrap();
        let embedding3 = generator.embed_text("Goodbye world").await.unwrap();

        assert_eq!(embedding1.len(), EMBEDDING_DIM);
        // Same text should produce same embedding
        assert!((embedding1[0] - embedding2[0]).abs() < 0.001);
        // Different text should produce different embedding
        assert!((embedding1[0] - embedding3[0]).abs() > 0.001);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((EmbeddingGenerator::cosine_similarity(&a, &b) - 1.0).abs() < 0.01);

        let c = vec![0.0, 1.0, 0.0];
        assert!((EmbeddingGenerator::cosine_similarity(&a, &c) - 0.0).abs() < 0.01);

        let d = vec![0.5, 0.5, 0.0];
        let expected = 0.707;
        assert!((EmbeddingGenerator::cosine_similarity(&a, &d) - expected).abs() < 0.01);

        // Orthogonal vectors
        let e = vec![0.0, 0.0, 1.0];
        assert!((EmbeddingGenerator::cosine_similarity(&a, &e) - 0.0).abs() < 0.01);

        // Opposite vectors
        let f = vec![-1.0, 0.0, 0.0];
        assert!((EmbeddingGenerator::cosine_similarity(&a, &f) - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0, 0.0];
        let b = vec![3.0, 4.0, 0.0];
        assert!((EmbeddingGenerator::euclidean_distance(&a, &b) - 5.0).abs() < 0.01);

        let c = vec![1.0, 1.0, 1.0];
        let d = vec![1.0, 1.0, 1.0];
        assert!((EmbeddingGenerator::euclidean_distance(&c, &d) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_top_k_similar() {
        let query = vec![1.0, 0.0, 0.0];
        let candidates = vec![
            vec![1.0, 0.0, 0.0],  // identical
            vec![0.0, 1.0, 0.0],  // orthogonal
            vec![0.9, 0.1, 0.0],  // very similar
            vec![0.5, 0.5, 0.0],  // moderate
            vec![0.1, 0.9, 0.0],  // different
        ];

        let top3 = EmbeddingGenerator::top_k_similar(&query, &candidates, 3);

        assert_eq!(top3.len(), 3);
        assert_eq!(top3[0].0, 0); // identical (index 0)
        assert_eq!(top3[1].0, 2); // very similar (index 2)
        assert_eq!(top3[2].0, 3); // moderate (index 3)
    }

    #[test]
    fn test_embedding_properties() {
        let generator = EmbeddingGenerator::new();

        // Same content should produce same embedding
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "The quick brown fox jumps over the lazy dog";

        let rt = tokio::runtime::Runtime::new().unwrap();
        let emb1 = rt.block_on(generator.embed_text(text1)).unwrap();
        let emb2 = rt.block_on(generator.embed_text(text2)).unwrap();

        for (a, b) in emb1.iter().zip(emb2.iter()) {
            assert!((a - b).abs() < 0.001);
        }
    }

    #[test]
    fn test_different_content_different_embedding() {
        let generator = EmbeddingGenerator::new();

        // Use texts that differ in length and structure
        let text1 = "aaa bbb ccc ddd eee fff ggg hhh iii jjj";
        let text2 = "zzz yyy xxx www vvv uuu ttt sss rrr qqq";

        let rt = tokio::runtime::Runtime::new().unwrap();
        let emb1 = rt.block_on(generator.embed_text(text1)).unwrap();
        let emb2 = rt.block_on(generator.embed_text(text2)).unwrap();

        let similarity = EmbeddingGenerator::cosine_similarity(&emb1, &emb2);

        // Embeddings should be different (not exactly 1.0)
        // Note: due to hash-based nature, similarity may still be high
        assert!(similarity < 1.0, "Identical embeddings should have similarity 1.0");
        assert_ne!(emb1, emb2, "Different texts should produce different embeddings");
    }

    #[test]
    fn test_structural_features() {
        let generator = EmbeddingGenerator::new();

        // Test code indicator - use text that retains code patterns after filtering
        let code_text = "fn main let var";  // Simple text with code-like words
        let rt = tokio::runtime::Runtime::new().unwrap();
        let emb_code = rt.block_on(generator.embed_text(code_text)).unwrap();

        // Code indicator should be positive for code-like text
        assert!(emb_code[268] > 0.0, "Code indicator should be positive for code-like text");

        // Test that different structural features detect different patterns
        let short_text = "a b c d e f g h i j k l m n o p";
        let emb_short = rt.block_on(generator.embed_text(short_text)).unwrap();

        // Word counts should differ
        assert!(emb_code[257] != emb_short[257], "Word counts should differ for different texts");
    }
}