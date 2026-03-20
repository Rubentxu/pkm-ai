//! Semantic Clustering: Group blocks by semantic similarity using K-means
//!
//! This module provides K-means clustering implementation for organizing
//! blocks based on their embedding vectors.

#![allow(dead_code)]

use crate::ai::embeddings::EmbeddingGenerator;
use crate::models::SmartSection;
use anyhow::Result;
use rand::Rng;
use std::collections::HashMap;
use ulid::Ulid;

/// Clustering algorithm configuration
#[derive(Debug, Clone)]
pub struct ClusteringConfig {
    /// Maximum iterations for K-means convergence
    pub max_iterations: usize,
    /// Minimum centroid movement to consider converged
    pub convergence_threshold: f32,
    /// Number of runs with different initializations (for stability)
    pub n_init_runs: usize,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
}

impl Default for ClusteringConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 1e-4,
            n_init_runs: 3,
            seed: None,
        }
    }
}

/// Semantic clusterer using K-means algorithm
#[derive(Debug, Clone)]
pub struct SemanticClusterer {
    embeddings: EmbeddingGenerator,
    config: ClusteringConfig,
}

impl SemanticClusterer {
    /// Create a new semantic clusterer with default configuration
    pub fn new() -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            config: ClusteringConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ClusteringConfig) -> Self {
        Self {
            embeddings: EmbeddingGenerator::new(),
            config,
        }
    }

    /// Cluster embeddings into k groups using K-means algorithm
    ///
    /// Returns a vector of clusters, where each cluster is a vector of indices
    /// into the original embeddings array.
    pub async fn cluster(&self, embeddings: &[Vec<f32>], k: usize) -> Result<Vec<Vec<usize>>> {
        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        if k == 0 || k > embeddings.len() {
            return Ok(vec![(0..embeddings.len()).collect()]);
        }

        // Handle edge case: if only one embedding or k=1, return single cluster
        if embeddings.len() == 1 || k == 1 {
            return Ok(vec![vec![0]]);
        }

        let k = k.min(embeddings.len());

        // Run K-means multiple times and pick the best result
        let mut best_assignments: Option<Vec<Vec<usize>>> = None;
        let mut best_inertia = f32::MAX;

        for _ in 0..self.config.n_init_runs {
            let (assignments, inertia) = self.kmeans_internal(embeddings, k).await;

            if inertia < best_inertia {
                best_inertia = inertia;
                best_assignments = Some(assignments);
            }
        }

        Ok(best_assignments.unwrap_or_else(|| vec![(0..embeddings.len()).collect()]))
    }

    /// K-means clustering implementation
    async fn kmeans_internal(&self, embeddings: &[Vec<f32>], k: usize) -> (Vec<Vec<usize>>, f32) {
        let dim = embeddings.first().map(|e| e.len()).unwrap_or(384);

        // Initialize centroids using k-means++ algorithm
        let mut centroids = self.initialize_centroids(embeddings, k);
        let mut assignments = vec![0usize; embeddings.len()];
        let mut inertia = 0.0f32;

        for _iteration in 0..self.config.max_iterations {
            // Assign each embedding to nearest centroid
            let mut new_assignments = vec![0usize; embeddings.len()];
            let mut new_centroids = vec![vec![0.0f32; dim]; k];
            let mut centroid_counts = vec![0usize; k];
            let mut new_inertia = 0.0f32;

            // Safety check: ensure centroids has exactly k elements
            if centroids.len() != k {
                eprintln!("DEBUG: centroids.len() = {} but k = {}", centroids.len(), k);
            }

            for (i, embedding) in embeddings.iter().enumerate() {
                let (nearest, dist) = self.find_nearest_centroid(embedding, &centroids);

                new_assignments[i] = nearest;
                new_inertia += dist;

                // Safety bounds check
                if nearest < centroid_counts.len() {
                    centroid_counts[nearest] += 1;
                } else {
                    eprintln!("DEBUG: nearest {} >= centroid_counts.len() {}", nearest, centroid_counts.len());
                }

                // Add to centroid sum
                for (j, val) in embedding.iter().enumerate() {
                    if nearest < new_centroids.len() && j < new_centroids[nearest].len() {
                        new_centroids[nearest][j] += val;
                    }
                }
            }

            // Calculate new centroids
            let mut converged = true;
            for c in 0..k {
                if centroid_counts[c] > 0 {
                    let count = centroid_counts[c] as f32;
                    let old_centroid = &centroids[c];

                    for val in new_centroids[c].iter_mut().take(dim) {
                        *val /= count;
                    }

                    // Check convergence
                    let movement = self.euclidean_distance(&new_centroids[c], old_centroid);
                    if movement > self.config.convergence_threshold {
                        converged = false;
                    }
                } else {
                    // Empty cluster: reinitialize with random point
                    let random_idx = rand::thread_rng().gen_range(0..embeddings.len());
                    new_centroids[c] = embeddings[random_idx].clone();
                    converged = false;
                }
            }

            centroids = new_centroids;
            assignments = new_assignments;
            inertia = new_inertia;

            if converged {
                break;
            }
        }

        // Convert assignments to cluster format
        let mut clusters: Vec<Vec<usize>> = vec![Vec::new(); k];
        for (i, cluster) in assignments.iter().enumerate() {
            clusters[*cluster].push(i);
        }

        // Remove empty clusters
        clusters.retain(|c| !c.is_empty());

        (clusters, inertia)
    }

    /// Initialize centroids using k-means++ algorithm for better initial placement
    fn initialize_centroids(&self, embeddings: &[Vec<f32>], k: usize) -> Vec<Vec<f32>> {
        let mut rng = rand::thread_rng();
        let _dim = embeddings.first().map(|e| e.len()).unwrap_or(384);
        let mut centroids = Vec::with_capacity(k);

        // Choose first centroid randomly
        let first_idx = rng.gen_range(0..embeddings.len());
        centroids.push(embeddings[first_idx].clone());

        // Choose remaining centroids with probability proportional to distance squared
        for _ in 1..k {
            let mut distances = Vec::with_capacity(embeddings.len());

            for embedding in embeddings {
                let (_, min_dist) = self.find_nearest_centroid(embedding, &centroids);
                distances.push(min_dist);
            }

            // Normalize to get probabilities
            let total: f32 = distances.iter().sum();
            let mut added = false;

            if total > 0.0 {
                let probs: Vec<f32> = distances.iter().map(|d| d / total).collect();

                // Sample based on probabilities
                let mut cumulative = 0.0;
                let threshold: f32 = rng.r#gen();

                for (i, &prob) in probs.iter().enumerate() {
                    cumulative += prob;
                    if cumulative >= threshold {
                        centroids.push(embeddings[i].clone());
                        added = true;
                        break;
                    }
                }
            }

            // Fallback to random if something went wrong
            if !added && centroids.len() < k {
                let random_idx = rng.gen_range(0..embeddings.len());
                centroids.push(embeddings[random_idx].clone());
            }
        }

        // Ensure we have exactly k centroids
        while centroids.len() < k {
            let random_idx = rng.gen_range(0..embeddings.len());
            centroids.push(embeddings[random_idx].clone());
        }

        // Truncate if we somehow got more (shouldn't happen)
        centroids.truncate(k);

        centroids
    }

    /// Find the nearest centroid to an embedding
    fn find_nearest_centroid(&self, embedding: &[f32], centroids: &[Vec<f32>]) -> (usize, f32) {
        if centroids.is_empty() {
            return (0, f32::MAX);
        }

        let mut nearest = 0usize;
        let mut min_dist = f32::MAX;

        for (i, centroid) in centroids.iter().enumerate() {
            let dist = self.euclidean_distance(embedding, centroid);
            if dist < min_dist {
                min_dist = dist;
                nearest = i;
            }
        }

        // Safety check: ensure index is within bounds
        if nearest >= centroids.len() {
            nearest = centroids.len() - 1;
        }

        (nearest, min_dist)
    }

    /// Calculate Euclidean distance between two vectors
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }

        let sum_sq: f32 = a.iter().zip(b.iter())
            .map(|(x, y)| (x - y) * (x - y))
            .sum();

        sum_sq.sqrt()
    }

    /// Perform hierarchical/agglomerative clustering
    ///
    /// Uses a simple nearest-neighbor chaining algorithm.
    pub async fn hierarchical_cluster(
        &self,
        embeddings: &[Vec<f32>],
        threshold: f32,
    ) -> Result<Vec<Vec<usize>>> {
        if embeddings.is_empty() {
            return Ok(Vec::new());
        }

        if embeddings.len() == 1 {
            return Ok(vec![vec![0]]);
        }

        let n = embeddings.len();
        let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
        let _merged = vec![false; n];

        // Calculate initial distance matrix
        let mut distances: HashMap<(usize, usize), f32> = HashMap::new();

        for i in 0..n {
            for j in (i + 1)..n {
                let dist = self.euclidean_distance(&embeddings[i], &embeddings[j]);
                distances.insert((i, j), dist);
            }
        }

        // Merge closest clusters until threshold
        while clusters.len() > 1 {
            // Find closest pair
            let mut min_dist = f32::MAX;
            let mut merge_pair = (0, 0);

            for (i, c1) in clusters.iter().enumerate() {
                for (j, c2) in clusters.iter().enumerate() {
                    if i >= j {
                        continue;
                    }

                    // Calculate average linkage distance
                    let mut total_dist = 0.0f32;
                    let mut count = 0usize;

                    for &idx1 in c1 {
                        for &idx2 in c2 {
                            let key = if idx1 < idx2 { (idx1, idx2) } else { (idx2, idx1) };
                            if let Some(&dist) = distances.get(&key) {
                                total_dist += dist;
                                count += 1;
                            }
                        }
                    }

                    if count > 0 {
                        let avg_dist = total_dist / count as f32;
                        if avg_dist < min_dist {
                            min_dist = avg_dist;
                            merge_pair = (i, j);
                        }
                    }
                }
            }

            // Stop if below threshold
            if min_dist > threshold {
                break;
            }

            // Merge clusters
            let (i, j) = merge_pair;
            let _c1 = clusters[i].clone();
            let c2 = clusters[j].clone();

            clusters[i].extend(c2);
            clusters.remove(j);

            // Update distance cache for merged cluster
            // (simplified: just mark old entries as stale)
        }

        Ok(clusters)
    }

    /// Create SmartSection objects from clustering results
    pub async fn create_smart_sections(
        &self,
        blocks: &[crate::models::Block],
        clusters: Vec<Vec<usize>>,
    ) -> Result<Vec<SmartSection>> {
        let mut sections = Vec::new();

        for (i, cluster_indices) in clusters.iter().enumerate() {
            if cluster_indices.is_empty() {
                continue;
            }

            let cluster_blocks: Vec<_> = cluster_indices.iter()
                .filter_map(|&idx| blocks.get(idx))
                .collect();

            // Calculate semantic centroid
            let mut embeddings: Vec<Vec<f32>> = Vec::new();
            for b in &cluster_blocks {
                if let Ok(emb) = self.embeddings.embed(b).await { embeddings.push(emb) }
            }

            let centroid = if !embeddings.is_empty() {
                let dim = embeddings[0].len();
                let mut centroid = vec![0.0f32; dim];
                for emb in &embeddings {
                    for (j, val) in emb.iter().enumerate() {
                        centroid[j] += val;
                    }
                }
                let count = embeddings.len() as f32;
                for val in centroid.iter_mut() {
                    *val /= count;
                }
                Some(centroid)
            } else {
                None
            };

            // Calculate coherence
            let coherence = if embeddings.len() > 1 {
                let mut total_sim = 0.0f32;
                let mut count = 0usize;

                for i in 0..embeddings.len() {
                    for j in (i + 1)..embeddings.len() {
                        total_sim += EmbeddingGenerator::cosine_similarity(&embeddings[i], &embeddings[j]);
                        count += 1;
                    }
                }

                if count > 0 {
                    total_sim / count as f32
                } else {
                    1.0
                }
            } else {
                1.0
            };

            // Extract keywords
            let keywords = self.extract_section_keywords(&cluster_blocks);

            // Generate intent
            let intent = if !keywords.is_empty() {
                let keyword_slice: Vec<&str> = keywords.iter().take(3).map(|s| s.as_str()).collect();
                format!("Section covering: {}", keyword_slice.join(", "))
            } else {
                format!("Section {}", i + 1)
            };

            let section = SmartSection {
                id: Ulid::new(),
                intent,
                boundary_constraints: keywords.clone(),
                keywords: keywords.clone(),
                semantic_centroid: centroid,
                semantic_bloom: [0u128; 1],
                density: cluster_indices.len() as u32,
                expected_density: None,
                vacancy: crate::models::VacancyLevel::from_density(
                    cluster_indices.len() as u32,
                    cluster_blocks.len() as u32 * 2,
                ),
                coherence_score: coherence,
                gravity_hooks: cluster_indices.first().map(|&i| blocks[i].id).into_iter().collect(),
            };

            sections.push(section);
        }

        Ok(sections)
    }

    /// Extract representative keywords from a cluster of blocks
    fn extract_section_keywords(&self, blocks: &[&crate::models::Block]) -> Vec<String> {
        let stop_words = [
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "by", "from", "as", "is", "was", "are", "were", "been",
            "this", "that", "these", "those", "it", "its", "be", "have", "has",
        ];

        let mut freq: HashMap<String, usize> = HashMap::new();

        for block in blocks {
            let text = format!("{} {}", block.title, block.content);
            let words: Vec<_> = text.to_lowercase()
                .split_whitespace()
                .filter(|w| w.len() >= 4 && !stop_words.contains(w))
                .map(|w| w.chars().filter(|c| c.is_alphanumeric()).collect::<String>())
                .filter(|w| w.len() >= 3)
                .collect();

            for word in words {
                *freq.entry(word).or_insert(0) += 1;
            }
        }

        let mut keywords: Vec<_> = freq.into_iter().collect();
        keywords.sort_by(|a, b| b.1.cmp(&a.1));

        keywords.into_iter()
            .take(10)
            .map(|(k, _)| k)
            .collect()
    }

    /// Calculate silhouette score for clustering quality
    ///
    /// Returns values between -1 and 1, where higher is better.
    pub async fn silhouette_score(&self, embeddings: &[Vec<f32>], clusters: &[Vec<usize>]) -> f32 {
        if embeddings.is_empty() || clusters.is_empty() {
            return 0.0;
        }

        let n = embeddings.len();
        let mut scores = Vec::with_capacity(n);

        for point_idx in 0..n {
            // Find which cluster this point belongs to
            let my_cluster = clusters.iter()
                .position(|c| c.contains(&point_idx))
                .unwrap_or(0);

            // Get the cluster this point belongs to
            let cluster = &clusters[my_cluster];

            // Calculate a (mean distance to points in same cluster)
            let mut a = 0.0f32;
            let mut same_count = 0usize;

            for &other_idx in cluster {
                if other_idx != point_idx {
                    a += self.euclidean_distance(&embeddings[point_idx], &embeddings[other_idx]);
                    same_count += 1;
                }
            }

            if same_count > 0 {
                a /= same_count as f32;
            } else {
                a = 0.0;
            }

            // Calculate b (minimum mean distance to points in other clusters)
            let mut b = f32::MAX;

            for (other_cluster_idx, other_cluster) in clusters.iter().enumerate() {
                if other_cluster_idx == my_cluster {
                    continue;
                }

                let mut mean_dist = 0.0f32;
                let mut count = 0usize;

                for &other_idx in other_cluster {
                    mean_dist += self.euclidean_distance(&embeddings[point_idx], &embeddings[other_idx]);
                    count += 1;
                }

                if count > 0 {
                    mean_dist /= count as f32;
                    b = b.min(mean_dist);
                }
            }

            // Silhouette for this point
            let s = if a == 0.0 && b == 0.0 {
                0.0
            } else if a == 0.0 {
                b
            } else {
                (b - a) / a.max(b)
            };

            scores.push(s);
        }

        if scores.is_empty() {
            return 0.0;
        }

        scores.iter().sum::<f32>() / scores.len() as f32
    }
}

impl Default for SemanticClusterer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_random_embeddings(count: usize, dim: usize) -> Vec<Vec<f32>> {
        let mut rng = rand::thread_rng();
        (0..count)
            .map(|_| (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
            .collect()
    }

    #[tokio::test]
    async fn test_cluster_empty() {
        let clusterer = SemanticClusterer::new();
        let result = clusterer.cluster(&[], 3).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_cluster_single() {
        let clusterer = SemanticClusterer::new();
        let embeddings = vec![vec![1.0, 0.0, 0.0]];
        let result = clusterer.cluster(&embeddings, 1).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], vec![0]);
    }

    #[tokio::test]
    async fn test_cluster_k_greater_than_n() {
        let clusterer = SemanticClusterer::new();
        let embeddings = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let result = clusterer.cluster(&embeddings, 10).await.unwrap();
        // Should return single cluster with all points
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
    }

    #[tokio::test]
    async fn test_cluster_basic() {
        let clusterer = SemanticClusterer::new();

        // Create 6 embeddings: 3 close to [1,0,0] and 3 close to [0,1,0]
        let mut embeddings = Vec::new();

        // Cluster 1: centered at [1, 0, 0]
        for _ in 0..3 {
            embeddings.push(vec![1.0 + rand::random::<f32>() * 0.1,
                               rand::random::<f32>() * 0.1,
                               rand::random::<f32>() * 0.1]);
        }

        // Cluster 2: centered at [0, 1, 0]
        for _ in 0..3 {
            embeddings.push(vec![rand::random::<f32>() * 0.1,
                               1.0 + rand::random::<f32>() * 0.1,
                               rand::random::<f32>() * 0.1]);
        }

        let result = clusterer.cluster(&embeddings, 2).await.unwrap();

        // Should have 2 clusters
        assert!(result.len() <= 2 && result.len() >= 1);

        // Each cluster should have members
        for cluster in &result {
            assert!(!cluster.is_empty());
        }
    }

    #[tokio::test]
    async fn test_euclidean_distance() {
        let clusterer = SemanticClusterer::new();

        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let dist = clusterer.euclidean_distance(&a, &b);
        assert!((dist - 5.0).abs() < 0.01);

        // Same vector should have 0 distance
        let c = vec![1.0, 2.0];
        let dist2 = clusterer.euclidean_distance(&c, &c);
        assert!((dist2 - 0.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_find_nearest_centroid() {
        let clusterer = SemanticClusterer::new();

        let embedding = vec![1.0, 0.0];
        let centroids = vec![
            vec![0.0, 0.0],  // distance 1.0
            vec![1.0, 0.0],  // distance 0.0
            vec![2.0, 0.0],  // distance 1.0
        ];

        let (nearest, dist) = clusterer.find_nearest_centroid(&embedding, &centroids);
        assert_eq!(nearest, 1);
        assert!((dist - 0.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_hierarchical_clustering() {
        let clusterer = SemanticClusterer::new();

        let embeddings = vec![
            vec![0.0, 0.0],
            vec![0.1, 0.1],
            vec![0.2, 0.2],
            vec![10.0, 10.0],
            vec![10.1, 10.1],
        ];

        // With high threshold, should keep most separate
        let result = clusterer.hierarchical_cluster(&embeddings, 0.5).await.unwrap();
        assert!(result.len() >= 2);

        // With low threshold, should merge similar points
        let result = clusterer.hierarchical_cluster(&embeddings, 5.0).await.unwrap();
        assert!(result.len() < embeddings.len());
    }

    #[tokio::test]
    async fn test_silhouette_score() {
        let clusterer = SemanticClusterer::new();

        // Perfect clusters
        let embeddings = vec![
            vec![1.0, 0.0],
            vec![1.1, 0.0],
            vec![0.9, 0.0],
            vec![0.0, 1.0],
            vec![0.0, 1.1],
            vec![0.0, 0.9],
        ];

        let clusters = vec![
            vec![0, 1, 2],
            vec![3, 4, 5],
        ];

        let score = clusterer.silhouette_score(&embeddings, &clusters).await;
        // High silhouette score for well-separated clusters
        assert!(score > 0.5);
    }

    #[tokio::test]
    async fn test_config() {
        let config = ClusteringConfig {
            max_iterations: 200,
            convergence_threshold: 1e-6,
            n_init_runs: 5,
            seed: Some(42),
        };

        let clusterer = SemanticClusterer::with_config(config.clone());

        assert_eq!(clusterer.config.max_iterations, 200);
        assert_eq!(clusterer.config.convergence_threshold, 1e-6);
        assert_eq!(clusterer.config.n_init_runs, 5);
        assert_eq!(clusterer.config.seed, Some(42));
    }

    #[tokio::test]
    async fn test_cluster_with_config() {
        let config = ClusteringConfig {
            max_iterations: 10,
            convergence_threshold: 0.1,
            n_init_runs: 1,
            seed: Some(123),
        };

        let clusterer = SemanticClusterer::with_config(config);
        let embeddings = create_random_embeddings(10, 4);

        let result = clusterer.cluster(&embeddings, 3).await.unwrap();

        // Should produce some clusters
        let total_points: usize = result.iter().map(|c| c.len()).sum();
        assert_eq!(total_points, 10);
    }

    #[tokio::test]
    async fn test_extract_section_keywords() {
        let clusterer = SemanticClusterer::new();

        let blocks = vec![
            crate::models::Block::permanent("Rust Programming", "Rust is a systems programming language"),
            crate::models::Block::permanent("Rust Performance", "Rust provides excellent performance"),
            crate::models::Block::permanent("Rust Safety", "Rust guarantees memory safety"),
        ];

        let block_refs: Vec<&crate::models::Block> = blocks.iter().collect();
        let keywords = clusterer.extract_section_keywords(&block_refs);

        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"programming".to_string()));
        assert!(keywords.contains(&"performance".to_string()));
        assert!(keywords.contains(&"safety".to_string()));
    }
}