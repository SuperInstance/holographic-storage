//! Distributed holographic encoding for redundant storage.
//!
//! This crate implements holographic storage principles where data is encoded
//! as wave interference patterns, distributed across multiple locations, and
//! reconstructable from partial fragments.


// ============================================================================
// interference_pattern module
// ============================================================================

pub mod interference_pattern {
    

    /// A wave pattern representing encoded data.
    #[derive(Debug, Clone)]
    pub struct WavePattern {
        pub amplitudes: Vec<f64>,
        pub phases: Vec<f64>,
        pub frequency: f64,
    }

    impl WavePattern {
        pub fn new(amplitudes: Vec<f64>, phases: Vec<f64>, frequency: f64) -> Self {
            Self { amplitudes, phases, frequency }
        }

        pub fn from_data(data: &[u8]) -> Self {
            let amplitudes: Vec<f64> = data.iter().map(|&b| b as f64 / 255.0).collect();
            let phases: Vec<f64> = data.iter().map(|&b| (b as f64 * std::f64::consts::PI * 2.0) / 256.0).collect();
            Self { amplitudes, phases, frequency: 1.0 }
        }

        pub fn sample(&self, t: f64) -> f64 {
            self.amplitudes.iter().zip(self.phases.iter())
                .map(|(a, p)| a * (self.frequency * t * 2.0 * std::f64::consts::PI + p).cos())
                .sum()
        }

        pub fn length(&self) -> usize {
            self.amplitudes.len()
        }

        pub fn superpose(&self, other: &WavePattern) -> WavePattern {
            let max_len = self.amplitudes.len().max(other.amplitudes.len());
            let mut amps = Vec::with_capacity(max_len);
            let mut phs = Vec::with_capacity(max_len);
            for i in 0..max_len {
                let a1 = self.amplitudes.get(i).copied().unwrap_or(0.0);
                let a2 = other.amplitudes.get(i).copied().unwrap_or(0.0);
                let p1 = self.phases.get(i).copied().unwrap_or(0.0);
                let p2 = other.phases.get(i).copied().unwrap_or(0.0);
                amps.push(a1 + a2);
                phs.push((p1 + p2) / 2.0);
            }
            WavePattern::new(amps, phs, self.frequency)
        }

        pub fn normalize(&mut self) {
            let max_amp = self.amplitudes.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(0.0001);
            for a in &mut self.amplitudes {
                *a /= max_amp;
            }
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            self.amplitudes.iter().zip(self.phases.iter())
                .map(|(a, p)| {
                    
                    (a * 128.0 + p * 20.0) as u8
                })
                .collect()
        }

        pub fn interference_with(&self, other: &WavePattern, alpha: f64) -> WavePattern {
            let max_len = self.amplitudes.len().max(other.amplitudes.len());
            let mut amps = Vec::with_capacity(max_len);
            let mut phs = Vec::with_capacity(max_len);
            for i in 0..max_len {
                let a1 = self.amplitudes.get(i).copied().unwrap_or(0.0);
                let a2 = other.amplitudes.get(i).copied().unwrap_or(0.0);
                let p1 = self.phases.get(i).copied().unwrap_or(0.0);
                let p2 = other.phases.get(i).copied().unwrap_or(0.0);
                amps.push(alpha * a1 + (1.0 - alpha) * a2);
                phs.push(p1 * alpha + p2 * (1.0 - alpha));
            }
            WavePattern::new(amps, phs, self.frequency)
        }

        pub fn energy(&self) -> f64 {
            self.amplitudes.iter().map(|a| a * a).sum()
        }

        pub fn add_noise(&mut self, noise_level: f64) {
            let pseudo_rand = |i: usize| -> f64 {
                let x = ((i as f64 * 12.9898 + 78.233).sin() * 43758.5453).fract();
                2.0 * x - 1.0
            };
            for (i, a) in self.amplitudes.iter_mut().enumerate() {
                *a += noise_level * pseudo_rand(i);
                *a = a.max(0.0);
            }
        }

        pub fn is_empty(&self) -> bool {
            self.amplitudes.is_empty()
        }
    }

    /// Encode data into an interference pattern with a reference wave.
    pub fn encode(data: &[u8], reference_frequency: f64) -> WavePattern {
        let mut pattern = WavePattern::from_data(data);
        pattern.frequency = reference_frequency;
        pattern
    }

    /// Create a carrier wave for holographic encoding.
    pub fn carrier_wave(frequency: f64, length: usize, phase_offset: f64) -> WavePattern {
        let amplitudes = vec![1.0; length];
        let phases: Vec<f64> = (0..length).map(|i| phase_offset + i as f64 * frequency).collect();
        WavePattern::new(amplitudes, phases, frequency)
    }

    /// Compute interference between two wave patterns at all sample points.
    pub fn compute_interference(a: &WavePattern, b: &WavePattern) -> Vec<f64> {
        let max_len = a.amplitudes.len().max(b.amplitudes.len());
        (0..max_len).map(|i| {
            let s1 = a.amplitudes.get(i).copied().unwrap_or(0.0) *
                     (a.phases.get(i).copied().unwrap_or(0.0)).cos();
            let s2 = b.amplitudes.get(i).copied().unwrap_or(0.0) *
                     (b.phases.get(i).copied().unwrap_or(0.0)).cos();
            s1 + s2
        }).collect()
    }

    /// Encode multiple data sources into a single holographic pattern.
    pub fn multiplex_encode(sources: &[&[u8]], reference_frequency: f64) -> WavePattern {
        let mut result = WavePattern::new(vec![], vec![], reference_frequency);
        for (idx, data) in sources.iter().enumerate() {
            let mut pattern = WavePattern::from_data(data);
            let angle_offset = idx as f64 * std::f64::consts::PI / sources.len() as f64;
            for p in &mut pattern.phases {
                *p += angle_offset;
            }
            pattern.frequency = reference_frequency;
            result = result.superpose(&pattern);
        }
        result
    }

    /// Demultiplex a holographic pattern to extract specific data.
    pub fn demultiplex(pattern: &WavePattern, index: usize, total: usize, expected_len: usize) -> Vec<f64> {
        let angle_offset = index as f64 * std::f64::consts::PI / total as f64;
        (0..expected_len).map(|i| {
            let amp = pattern.amplitudes.get(i).copied().unwrap_or(0.0);
            let phase = pattern.phases.get(i).copied().unwrap_or(0.0);
            amp * (phase - angle_offset).cos()
        }).collect()
    }
}

// ============================================================================
// reference_beam module
// ============================================================================

pub mod reference_beam {
    use std::collections::HashMap;

    /// A reference beam used to retrieve data from holographic storage.
    #[derive(Debug, Clone)]
    pub struct ReferenceBeam {
        pub frequency: f64,
        pub angle: f64,
        pub phase_offset: f64,
        pub intensity: f64,
    }

    impl ReferenceBeam {
        pub fn new(frequency: f64, angle: f64, phase_offset: f64) -> Self {
            Self { frequency, angle, phase_offset, intensity: 1.0 }
        }

        pub fn with_intensity(mut self, intensity: f64) -> Self {
            self.intensity = intensity;
            self
        }

        pub fn generate_key(&self, data_len: usize) -> Vec<f64> {
            (0..data_len).map(|i| {
                self.intensity * (self.frequency * i as f64 * 2.0 * std::f64::consts::PI / data_len as f64 + self.phase_offset).cos()
            }).collect()
        }

        pub fn matches(&self, other: &ReferenceBeam, tolerance: f64) -> bool {
            (self.frequency - other.frequency).abs() < tolerance &&
            (self.angle - other.angle).abs() < tolerance &&
            (self.phase_offset - other.phase_offset).abs() < tolerance
        }

        pub fn correlation(&self, data: &[f64], offset: usize) -> f64 {
            let key = self.generate_key(data.len());
            let mut sum = 0.0;
            for i in 0..data.len() {
                let ki = key.get((i + offset) % key.len()).copied().unwrap_or(0.0);
                sum += data[i] * ki;
            }
            sum / data.len() as f64
        }

        pub fn rotate(&mut self, delta_angle: f64) {
            self.angle += delta_angle;
            self.angle %= (2.0 * std::f64::consts::PI);
        }

        pub fn shift_phase(&mut self, delta: f64) {
            self.phase_offset += delta;
        }

        /// Generate multiple reference beams at evenly spaced angles.
        pub fn fan(count: usize, base_frequency: f64) -> Vec<ReferenceBeam> {
            (0..count).map(|i| {
                let angle = i as f64 * 2.0 * std::f64::consts::PI / count as f64;
                ReferenceBeam::new(base_frequency, angle, 0.0)
            }).collect()
        }

        pub fn to_hash_key(&self) -> u64 {
            let f_bits = self.frequency.to_bits();
            let a_bits = self.angle.to_bits();
            f_bits.wrapping_add(a_bits).wrapping_mul(0x517cc1b727220a95)
        }
    }

    /// A key store mapping data identifiers to reference beams.
    #[derive(Debug, Clone)]
    pub struct KeyStore {
        keys: HashMap<String, ReferenceBeam>,
    }

    impl KeyStore {
        pub fn new() -> Self {
            Self { keys: HashMap::new() }
        }

        pub fn store(&mut self, id: &str, beam: ReferenceBeam) {
            self.keys.insert(id.to_string(), beam);
        }

        pub fn retrieve(&self, id: &str) -> Option<&ReferenceBeam> {
            self.keys.get(id)
        }

        pub fn remove(&mut self, id: &str) -> Option<ReferenceBeam> {
            self.keys.remove(id)
        }

        pub fn contains(&self, id: &str) -> bool {
            self.keys.contains_key(id)
        }

        pub fn len(&self) -> usize {
            self.keys.len()
        }

        pub fn is_empty(&self) -> bool {
            self.keys.is_empty()
        }

        pub fn list_keys(&self) -> Vec<String> {
            self.keys.keys().cloned().collect()
        }

        pub fn find_matching(&self, beam: &ReferenceBeam, tolerance: f64) -> Vec<String> {
            self.keys.iter()
                .filter(|(_, b)| b.matches(beam, tolerance))
                .map(|(k, _)| k.clone())
                .collect()
        }
    }

    impl Default for KeyStore {
        fn default() -> Self {
            Self::new()
        }
    }
}

// ============================================================================
// reconstruction module
// ============================================================================

pub mod reconstruction {
    /// A partially reconstructed hologram.
    #[derive(Debug, Clone)]
    pub struct PartialReconstruction {
        pub data: Vec<Option<f64>>,
        pub confidence: Vec<f64>,
        pub total_samples: usize,
    }

    impl PartialReconstruction {
        pub fn new(size: usize) -> Self {
            Self {
                data: vec![None; size],
                confidence: vec![0.0; size],
                total_samples: 0,
            }
        }

        pub fn update(&mut self, index: usize, value: f64, conf: f64) {
            if index < self.data.len() {
                self.total_samples += 1;
                match self.data[index] {
                    None => {
                        self.data[index] = Some(value);
                        self.confidence[index] = conf;
                    }
                    Some(existing) => {
                        let old_conf = self.confidence[index];
                        let total_conf = old_conf + conf;
                        self.data[index] = Some((existing * old_conf + value * conf) / total_conf);
                        self.confidence[index] = total_conf;
                    }
                }
            }
        }

        pub fn get(&self, index: usize) -> Option<f64> {
            self.data.get(index).and_then(|&v| v)
        }

        pub fn completion_ratio(&self) -> f64 {
            let filled = self.data.iter().filter(|v| v.is_some()).count();
            filled as f64 / self.data.len().max(1) as f64
        }

        pub fn average_confidence(&self) -> f64 {
            let filled: Vec<_> = self.confidence.iter().zip(self.data.iter())
                .filter(|(_, d)| d.is_some())
                .map(|(c, _)| *c)
                .collect();
            if filled.is_empty() { 0.0 } else { filled.iter().sum::<f64>() / filled.len() as f64 }
        }

        pub fn is_complete(&self) -> bool {
            self.data.iter().all(|v| v.is_some())
        }

        pub fn filled_count(&self) -> usize {
            self.data.iter().filter(|v| v.is_some()).count()
        }

        pub fn to_bytes(&self, threshold: f64) -> Vec<u8> {
            self.data.iter().zip(self.confidence.iter())
                .filter_map(|(v, c)| {
                    if *c >= threshold {
                        v.map(|val| (val * 255.0).clamp(0.0, 255.0) as u8)
                    } else {
                        None
                    }
                })
                .collect()
        }

        pub fn merge(&mut self, other: &PartialReconstruction) {
            for i in 0..self.data.len().min(other.data.len()) {
                if let Some(v) = other.data[i] {
                    self.update(i, v, other.confidence[i]);
                }
            }
        }

        pub fn size(&self) -> usize {
            self.data.len()
        }
    }

    /// Reconstruct data from fragments using weighted averaging.
    pub fn reconstruct_from_fragments(fragments: &[&[f64]], fragment_weights: &[f64]) -> Vec<f64> {
        if fragments.is_empty() { return vec![]; }
        let max_len = fragments.iter().map(|f| f.len()).max().unwrap_or(0);
        let mut result = vec![0.0; max_len];
        let mut total_weight = vec![0.0; max_len];

        for (frag, weight) in fragments.iter().zip(fragment_weights.iter()) {
            for (i, &val) in frag.iter().enumerate() {
                result[i] += val * weight;
                total_weight[i] += weight;
            }
        }
        for i in 0..max_len {
            if total_weight[i] > 0.0 {
                result[i] /= total_weight[i];
            }
        }
        result
    }

    /// Estimate quality of reconstruction.
    pub fn estimate_quality(original: &[f64], reconstructed: &[f64]) -> f64 {
        if original.is_empty() || reconstructed.is_empty() { return 0.0; }
        let min_len = original.len().min(reconstructed.len());
        let mut sum_sq = 0.0;
        let mut sum_orig_sq = 0.0;
        for i in 0..min_len {
            let diff = original[i] - reconstructed[i];
            sum_sq += diff * diff;
            sum_orig_sq += original[i] * original[i];
        }
        if sum_orig_sq < 1e-10 { return 1.0; }
        1.0 - (sum_sq / sum_orig_sq).min(1.0)
    }

    /// Fill gaps in reconstructed data using linear interpolation.
    pub fn interpolate_gaps(data: &mut [Option<f64>]) -> usize {
        let len = data.len();
        if len == 0 { return 0; }
        let mut filled = 0;
        for i in 0..len {
            if data[i].is_none() {
                let prev = (0..i).rev().find_map(|j| data[j]);
                let next = (i+1..len).find_map(|j| data[j]);
                match (prev, next) {
                    (Some(p), Some(n)) => {
                        data[i] = Some((p + n) / 2.0);
                        filled += 1;
                    }
                    (Some(p), None) => { data[i] = Some(p); filled += 1; }
                    (None, Some(n)) => { data[i] = Some(n); filled += 1; }
                    _ => {}
                }
            }
        }
        filled
    }
}

// ============================================================================
// redundancy module
// ============================================================================

pub mod redundancy {
    use std::collections::HashMap;

    /// Configuration for redundancy splitting.
    #[derive(Debug, Clone)]
    pub struct RedundancyConfig {
        pub total_shards: usize,
        pub threshold: usize,
    }

    impl RedundancyConfig {
        pub fn new(total: usize, threshold: usize) -> Self {
            Self { total_shards: total, threshold }
        }

        pub fn can_recover(&self, available: usize) -> bool {
            available >= self.threshold
        }

        pub fn redundancy_ratio(&self) -> f64 {
            self.total_shards as f64 / self.threshold as f64
        }
    }

    /// A shard of data distributed across storage locations.
    #[derive(Debug, Clone)]
    pub struct Shard {
        pub index: usize,
        pub data: Vec<u8>,
        pub checksum: u32,
    }

    impl Shard {
        pub fn new(index: usize, data: Vec<u8>) -> Self {
            let checksum = Self::compute_checksum(&data);
            Self { index, data, checksum }
        }

        pub fn verify(&self) -> bool {
            Self::compute_checksum(&self.data) == self.checksum
        }

        fn compute_checksum(data: &[u8]) -> u32 {
            data.iter().fold(0u32, |acc, &b| acc.wrapping_add(b as u32))
        }

        pub fn size(&self) -> usize {
            self.data.len()
        }

        pub fn is_empty(&self) -> bool {
            self.data.is_empty()
        }
    }

    /// Split data into N shards using simple XOR-based redundancy.
    pub fn split_data(data: &[u8], config: &RedundancyConfig) -> Vec<Shard> {
        let n = config.total_shards;
        let k = config.threshold;
        let chunk_size = data.len().div_ceil(k);

        // Create k data chunks
        let data_chunks: Vec<Vec<u8>> = (0..k).map(|i| {
            let start = i * chunk_size;
            let end = (start + chunk_size).min(data.len());
            let mut chunk = data[start..end].to_vec();
            chunk.resize(chunk_size, 0u8);
            chunk
        }).collect();

        let mut shards: Vec<Shard> = data_chunks.into_iter().enumerate()
            .map(|(i, chunk)| Shard::new(i, chunk))
            .collect();

        // Create parity shards
        for parity_idx in 0..(n - k) {
            let mut parity = vec![0u8; chunk_size];
            for shard in &shards[..k] {
                for (j, p) in parity.iter_mut().enumerate() {
                    if j < shard.data.len() {
                        *p ^= shard.data[j];
                    }
                }
            }
            // Vary parity by shifting indices
            if parity_idx > 0 {
                let shift = parity_idx % parity.len().max(1);
                parity.rotate_right(shift);
            }
            shards.push(Shard::new(k + parity_idx, parity));
        }

        shards
    }

    /// Reconstruct data from available shards.
    pub fn reconstruct_data(shards: &[Shard], config: &RedundancyConfig, original_len: usize) -> Option<Vec<u8>> {
        if shards.len() < config.threshold { return None; }

        // If we have all data shards, just concatenate
        let data_shard_count = shards.iter().filter(|s| s.index < config.threshold).count();
        if data_shard_count >= config.threshold {
            let mut data_shards: Vec<&Shard> = shards.iter()
                .filter(|s| s.index < config.threshold)
                .collect();
            data_shards.sort_by_key(|s| s.index);

            let mut result = Vec::new();
            for (i, shard) in data_shards.iter().enumerate() {
                let is_last = i == data_shards.len() - 1;
                let end = if is_last {
                    shard.data.len().min(original_len.saturating_sub(result.len()))
                } else {
                    shard.data.len()
                };
                result.extend_from_slice(&shard.data[..end]);
            }
            result.truncate(original_len);
            return Some(result);
        }

        // Simple parity recovery: if we're missing at most 1 data shard and have parity
        let missing: Vec<usize> = (0..config.threshold)
            .filter(|i| !shards.iter().any(|s| s.index == *i))
            .collect();

        if missing.len() == 1 && shards.len() >= config.threshold {
            let missing_idx = missing[0];
            let chunk_size = shards[0].data.len();

            // XOR all available data shards + parity to recover missing
            let mut recovered = vec![0u8; chunk_size];
            for shard in shards {
                for (j, r) in recovered.iter_mut().enumerate() {
                    if j < shard.data.len() {
                        *r ^= shard.data[j];
                    }
                }
            }

            let mut result = vec![0u8; original_len];
            let mut offset = 0;
            for i in 0..config.threshold {
                let shard_data = if i == missing_idx {
                    &recovered
                } else if let Some(s) = shards.iter().find(|s| s.index == i) {
                    &s.data
                } else {
                    continue;
                };
                let copy_len = shard_data.len().min(original_len - offset);
                result[offset..offset + copy_len].copy_from_slice(&shard_data[..copy_len]);
                offset += copy_len;
            }
            return Some(result);
        }

        None
    }

    /// Distribute shards across named locations.
    pub fn distribute_shards(shards: Vec<Shard>, locations: &[&str]) -> HashMap<String, Vec<Shard>> {
        let mut map: HashMap<String, Vec<Shard>> = HashMap::new();
        for (i, shard) in shards.into_iter().enumerate() {
            let loc = locations[i % locations.len()];
            map.entry(loc.to_string()).or_default().push(shard);
        }
        map
    }

    /// Collect shards from multiple locations.
    pub fn collect_shards(locations: &HashMap<String, Vec<Shard>>) -> Vec<Shard> {
        let mut all: Vec<Shard> = locations.values().flat_map(|v| v.iter().cloned()).collect();
        all.sort_by_key(|s| s.index);
        all.dedup_by_key(|s| s.index);
        all
    }

    /// Verify all shards have valid checksums.
    pub fn verify_shards(shards: &[Shard]) -> bool {
        shards.iter().all(|s| s.verify())
    }
}

// ============================================================================
// fragment module
// ============================================================================

pub mod fragment {
    use std::collections::HashMap;

    /// A fragment of a hologram stored at a specific location.
    #[derive(Debug, Clone)]
    pub struct Fragment {
        pub id: u64,
        pub data: Vec<f64>,
        pub source_location: String,
        pub metadata: HashMap<String, String>,
    }

    impl Fragment {
        pub fn new(id: u64, data: Vec<f64>, source: &str) -> Self {
            Self {
                id,
                data,
                source_location: source.to_string(),
                metadata: HashMap::new(),
            }
        }

        pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
            self.metadata.insert(key.to_string(), value.to_string());
            self
        }

        pub fn size(&self) -> usize {
            self.data.len()
        }

        pub fn is_empty(&self) -> bool {
            self.data.is_empty()
        }

        pub fn energy(&self) -> f64 {
            self.data.iter().map(|v| v * v).sum()
        }

        pub fn normalize(&mut self) {
            let max_val = self.data.iter().cloned().fold(0.0_f64, f64::max);
            if max_val > 0.0 {
                for v in &mut self.data {
                    *v /= max_val;
                }
            }
        }

        pub fn crop(&self, start: usize, end: usize) -> Fragment {
            let cropped: Vec<f64> = self.data.iter()
                .skip(start)
                .take(end.saturating_sub(start))
                .cloned()
                .collect();
            Fragment::new(self.id, cropped, &self.source_location)
        }

        pub fn merge(&self, other: &Fragment) -> Fragment {
            let mut merged = self.data.clone();
            merged.extend_from_slice(&other.data);
            Fragment::new(self.id, merged, &self.source_location)
        }

        pub fn correlation(&self, other: &Fragment) -> f64 {
            let min_len = self.data.len().min(other.data.len());
            if min_len == 0 { return 0.0; }
            let dot: f64 = (0..min_len).map(|i| self.data[i] * other.data[i]).sum();
            let norm_a = self.energy().sqrt();
            let norm_b = other.energy().sqrt();
            if norm_a < 1e-10 || norm_b < 1e-10 { return 0.0; }
            dot / (norm_a * norm_b)
        }

        pub fn to_bytes(&self) -> Vec<u8> {
            self.data.iter().map(|&v| (v * 255.0).clamp(0.0, 255.0) as u8).collect()
        }

        pub fn from_bytes(id: u64, bytes: &[u8], source: &str) -> Self {
            let data: Vec<f64> = bytes.iter().map(|&b| b as f64 / 255.0).collect();
            Fragment::new(id, data, source)
        }
    }

    /// A collection of fragments with lookup capabilities.
    #[derive(Debug, Clone)]
    pub struct FragmentStore {
        fragments: HashMap<u64, Fragment>,
    }

    impl FragmentStore {
        pub fn new() -> Self {
            Self { fragments: HashMap::new() }
        }

        pub fn add(&mut self, fragment: Fragment) {
            self.fragments.insert(fragment.id, fragment);
        }

        pub fn get(&self, id: u64) -> Option<&Fragment> {
            self.fragments.get(&id)
        }

        pub fn remove(&mut self, id: u64) -> Option<Fragment> {
            self.fragments.remove(&id)
        }

        pub fn contains(&self, id: u64) -> bool {
            self.fragments.contains_key(&id)
        }

        pub fn len(&self) -> usize {
            self.fragments.len()
        }

        pub fn is_empty(&self) -> bool {
            self.fragments.is_empty()
        }

        pub fn total_size(&self) -> usize {
            self.fragments.values().map(|f| f.size()).sum()
        }

        pub fn find_by_source(&self, source: &str) -> Vec<&Fragment> {
            self.fragments.values().filter(|f| f.source_location == source).collect()
        }

        pub fn find_high_energy(&self, threshold: f64) -> Vec<&Fragment> {
            self.fragments.values().filter(|f| f.energy() > threshold).collect()
        }

        pub fn all_ids(&self) -> Vec<u64> {
            self.fragments.keys().copied().collect()
        }
    }

    impl Default for FragmentStore {
        fn default() -> Self {
            Self::new()
        }
    }
}

// Re-exports
pub use interference_pattern::{WavePattern, encode, carrier_wave, compute_interference, multiplex_encode, demultiplex};
pub use reference_beam::{ReferenceBeam, KeyStore};
pub use reconstruction::{PartialReconstruction, reconstruct_from_fragments, estimate_quality, interpolate_gaps};
pub use redundancy::{RedundancyConfig, Shard, split_data, reconstruct_data, distribute_shards, collect_shards, verify_shards};
pub use fragment::{Fragment, FragmentStore};

#[cfg(test)]
mod tests {
    use super::*;

    // ---- interference_pattern tests (15) ----

    #[test]
    fn test_wave_pattern_new() {
        let wp = interference_pattern::WavePattern::new(vec![1.0, 0.5], vec![0.0, 1.0], 2.0);
        assert_eq!(wp.amplitudes, vec![1.0, 0.5]);
        assert_eq!(wp.phases, vec![0.0, 1.0]);
        assert_eq!(wp.frequency, 2.0);
    }

    #[test]
    fn test_wave_pattern_from_data() {
        let data = vec![128u8, 255];
        let wp = interference_pattern::WavePattern::from_data(&data);
        assert_eq!(wp.amplitudes.len(), 2);
        assert!((wp.amplitudes[0] - 128.0/255.0).abs() < 0.01);
    }

    #[test]
    fn test_wave_pattern_sample() {
        let wp = interference_pattern::WavePattern::new(vec![1.0], vec![0.0], 1.0);
        let s = wp.sample(0.0);
        assert!((s - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_wave_pattern_length() {
        let wp = interference_pattern::WavePattern::new(vec![1.0, 2.0, 3.0], vec![0.0; 3], 1.0);
        assert_eq!(wp.length(), 3);
    }

    #[test]
    fn test_wave_pattern_superpose() {
        let a = interference_pattern::WavePattern::new(vec![1.0, 2.0], vec![0.0; 2], 1.0);
        let b = interference_pattern::WavePattern::new(vec![3.0], vec![1.0], 1.0);
        let s = a.superpose(&b);
        assert_eq!(s.amplitudes, vec![4.0, 2.0]);
    }

    #[test]
    fn test_wave_pattern_normalize() {
        let mut wp = interference_pattern::WavePattern::new(vec![2.0, 4.0], vec![0.0; 2], 1.0);
        wp.normalize();
        assert!((wp.amplitudes[0] - 0.5).abs() < 0.01);
        assert!((wp.amplitudes[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_wave_pattern_to_bytes() {
        let wp = interference_pattern::WavePattern::new(vec![0.5], vec![0.5], 1.0);
        let bytes = wp.to_bytes();
        assert_eq!(bytes.len(), 1);
    }

    #[test]
    fn test_encode() {
        let data = vec![1u8, 2, 3, 4];
        let pattern = encode(&data, 2.0);
        assert_eq!(pattern.amplitudes.len(), 4);
        assert_eq!(pattern.frequency, 2.0);
    }

    #[test]
    fn test_carrier_wave() {
        let cw = carrier_wave(1.0, 5, 0.0);
        assert_eq!(cw.amplitudes.len(), 5);
        assert_eq!(cw.phases.len(), 5);
    }

    #[test]
    fn test_compute_interference() {
        let a = interference_pattern::WavePattern::new(vec![1.0], vec![0.0], 1.0);
        let b = interference_pattern::WavePattern::new(vec![1.0], vec![0.0], 1.0);
        let interference = compute_interference(&a, &b);
        assert_eq!(interference.len(), 1);
        assert!(interference[0] > 0.0);
    }

    #[test]
    fn test_interference_with_alpha() {
        let a = interference_pattern::WavePattern::new(vec![1.0], vec![0.0], 1.0);
        let b = interference_pattern::WavePattern::new(vec![0.0], vec![0.0], 1.0);
        let mixed = a.interference_with(&b, 0.5);
        assert!((mixed.amplitudes[0] - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_energy() {
        let wp = interference_pattern::WavePattern::new(vec![3.0, 4.0], vec![0.0; 2], 1.0);
        assert!((wp.energy() - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_add_noise() {
        let mut wp = interference_pattern::WavePattern::new(vec![0.5, 0.5], vec![0.0; 2], 1.0);
        wp.add_noise(0.1);
        // Values should change but remain non-negative
        for a in &wp.amplitudes {
            assert!(*a >= 0.0);
        }
    }

    #[test]
    fn test_multiplex_encode() {
        let sources: Vec<&[u8]> = vec![&[1, 2, 3], &[4, 5, 6]];
        let pattern = multiplex_encode(&sources, 1.0);
        assert_eq!(pattern.amplitudes.len(), 3);
    }

    #[test]
    fn test_demultiplex() {
        let sources: Vec<&[u8]> = vec![&[1, 2], &[3, 4]];
        let pattern = multiplex_encode(&sources, 1.0);
        let demux = demultiplex(&pattern, 0, 2, 2);
        assert_eq!(demux.len(), 2);
    }

    #[test]
    fn test_wave_pattern_is_empty() {
        let wp = interference_pattern::WavePattern::new(vec![], vec![], 1.0);
        assert!(wp.is_empty());
    }

    // ---- reference_beam tests (12) ----

    #[test]
    fn test_reference_beam_new() {
        let rb = reference_beam::ReferenceBeam::new(1.0, 0.5, 0.0);
        assert_eq!(rb.frequency, 1.0);
        assert_eq!(rb.angle, 0.5);
        assert_eq!(rb.intensity, 1.0);
    }

    #[test]
    fn test_reference_beam_with_intensity() {
        let rb = reference_beam::ReferenceBeam::new(1.0, 0.0, 0.0).with_intensity(2.5);
        assert!((rb.intensity - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_generate_key() {
        let rb = reference_beam::ReferenceBeam::new(1.0, 0.0, 0.0);
        let key = rb.generate_key(10);
        assert_eq!(key.len(), 10);
    }

    #[test]
    fn test_matches_exact() {
        let a = reference_beam::ReferenceBeam::new(1.0, 0.5, 0.3);
        let b = reference_beam::ReferenceBeam::new(1.0, 0.5, 0.3);
        assert!(a.matches(&b, 0.01));
    }

    #[test]
    fn test_matches_no_match() {
        let a = reference_beam::ReferenceBeam::new(1.0, 0.5, 0.3);
        let b = reference_beam::ReferenceBeam::new(5.0, 0.5, 0.3);
        assert!(!a.matches(&b, 0.01));
    }

    #[test]
    fn test_correlation() {
        let rb = reference_beam::ReferenceBeam::new(1.0, 0.0, 0.0);
        let data = vec![1.0, 0.5, 0.3];
        let corr = rb.correlation(&data, 0);
        assert!(!corr.is_nan());
    }

    #[test]
    fn test_rotate() {
        let mut rb = reference_beam::ReferenceBeam::new(1.0, 1.0, 0.0);
        rb.rotate(1.0);
        assert!((rb.angle - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_shift_phase() {
        let mut rb = reference_beam::ReferenceBeam::new(1.0, 0.0, 1.0);
        rb.shift_phase(0.5);
        assert!((rb.phase_offset - 1.5).abs() < 0.01);
    }

    #[test]
    fn test_fan() {
        let beams = reference_beam::ReferenceBeam::fan(4, 1.0);
        assert_eq!(beams.len(), 4);
        for beam in &beams {
            assert!((beam.frequency - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_key_store_basic() {
        let mut ks = reference_beam::KeyStore::new();
        let rb = reference_beam::ReferenceBeam::new(1.0, 0.0, 0.0);
        ks.store("test", rb);
        assert!(ks.contains("test"));
        assert!(!ks.contains("other"));
        assert_eq!(ks.len(), 1);
    }

    #[test]
    fn test_key_store_retrieve() {
        let mut ks = reference_beam::KeyStore::new();
        ks.store("a", reference_beam::ReferenceBeam::new(1.0, 0.0, 0.0));
        ks.store("b", reference_beam::ReferenceBeam::new(2.0, 0.0, 0.0));
        let retrieved = ks.retrieve("a").unwrap();
        assert!((retrieved.frequency - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_key_store_find_matching() {
        let mut ks = reference_beam::KeyStore::new();
        ks.store("x", reference_beam::ReferenceBeam::new(1.0, 0.5, 0.3));
        let query = reference_beam::ReferenceBeam::new(1.0, 0.5, 0.3);
        let matches = ks.find_matching(&query, 0.1);
        assert_eq!(matches, vec!["x"]);
    }

    // ---- reconstruction tests (10) ----

    #[test]
    fn test_partial_reconstruction_new() {
        let pr = reconstruction::PartialReconstruction::new(10);
        assert_eq!(pr.size(), 10);
        assert!(!pr.is_complete());
        assert_eq!(pr.filled_count(), 0);
    }

    #[test]
    fn test_partial_reconstruction_update() {
        let mut pr = reconstruction::PartialReconstruction::new(5);
        pr.update(0, 1.0, 0.9);
        assert_eq!(pr.get(0), Some(1.0));
        assert_eq!(pr.filled_count(), 1);
    }

    #[test]
    fn test_partial_reconstruction_weighted_update() {
        let mut pr = reconstruction::PartialReconstruction::new(3);
        pr.update(0, 1.0, 1.0);
        pr.update(0, 3.0, 1.0);
        // weighted average: (1.0*1.0 + 3.0*1.0) / (1.0+1.0) = 2.0
        assert!((pr.get(0).unwrap() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_completion_ratio() {
        let mut pr = reconstruction::PartialReconstruction::new(4);
        pr.update(0, 1.0, 0.5);
        pr.update(2, 0.5, 0.5);
        assert!((pr.completion_ratio() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_is_complete() {
        let mut pr = reconstruction::PartialReconstruction::new(2);
        pr.update(0, 1.0, 1.0);
        pr.update(1, 2.0, 1.0);
        assert!(pr.is_complete());
    }

    #[test]
    fn test_reconstruct_from_fragments() {
        let f1: Vec<f64> = vec![1.0, 2.0];
        let f2: Vec<f64> = vec![3.0, 4.0];
        let result = reconstruction::reconstruct_from_fragments(&[&f1, &f2], &[0.5, 0.5]);
        assert!((result[0] - 2.0).abs() < 0.01);
        assert!((result[1] - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_estimate_quality_perfect() {
        let orig = vec![1.0, 2.0, 3.0];
        let recon = vec![1.0, 2.0, 3.0];
        let q = reconstruction::estimate_quality(&orig, &recon);
        assert!((q - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_estimate_quality_poor() {
        let orig = vec![1.0, 2.0, 3.0];
        let recon = vec![10.0, 20.0, 30.0];
        let q = reconstruction::estimate_quality(&orig, &recon);
        assert!(q < 0.5);
    }

    #[test]
    fn test_interpolate_gaps() {
        let mut data: Vec<Option<f64>> = vec![Some(1.0), None, Some(3.0)];
        let filled = reconstruction::interpolate_gaps(&mut data);
        assert_eq!(filled, 1);
        assert!((data[1].unwrap() - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_partial_reconstruction_merge() {
        let mut pr1 = reconstruction::PartialReconstruction::new(3);
        pr1.update(0, 1.0, 0.5);
        let mut pr2 = reconstruction::PartialReconstruction::new(3);
        pr2.update(1, 2.0, 0.8);
        pr2.update(2, 3.0, 0.9);
        pr1.merge(&pr2);
        assert_eq!(pr1.filled_count(), 3);
    }

    // ---- redundancy tests (8) ----

    #[test]
    fn test_redundancy_config() {
        let config = redundancy::RedundancyConfig::new(5, 3);
        assert!(config.can_recover(3));
        assert!(!config.can_recover(2));
        assert!((config.redundancy_ratio() - 5.0/3.0).abs() < 0.01);
    }

    #[test]
    fn test_shard_new() {
        let shard = redundancy::Shard::new(0, vec![1, 2, 3]);
        assert_eq!(shard.index, 0);
        assert_eq!(shard.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_shard_verify_valid() {
        let shard = redundancy::Shard::new(0, vec![1, 2, 3]);
        assert!(shard.verify());
    }

    #[test]
    fn test_shard_verify_corrupted() {
        let mut shard = redundancy::Shard::new(0, vec![1, 2, 3]);
        shard.data[0] = 99;
        assert!(!shard.verify());
    }

    #[test]
    fn test_split_and_reconstruct_all_shards() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let config = redundancy::RedundancyConfig::new(4, 2);
        let shards = redundancy::split_data(&data, &config);
        assert_eq!(shards.len(), 4);
        let result = redundancy::reconstruct_data(&shards, &config, data.len());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_split_and_reconstruct_partial() {
        let data = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
        let config = redundancy::RedundancyConfig::new(4, 2);
        let shards = redundancy::split_data(&data, &config);
        // Use only threshold shards
        let partial: Vec<redundancy::Shard> = shards.into_iter().take(2).collect();
        let result = redundancy::reconstruct_data(&partial, &config, data.len());
        assert_eq!(result.unwrap(), data);
    }

    #[test]
    fn test_distribute_and_collect_shards() {
        let config = redundancy::RedundancyConfig::new(4, 2);
        let shards = redundancy::split_data(&[1u8, 2, 3], &config);
        let distributed = redundancy::distribute_shards(shards, &["loc_a", "loc_b"]);
        assert_eq!(distributed.len(), 2);
        let collected = redundancy::collect_shards(&distributed);
        assert_eq!(collected.len(), 4);
    }

    #[test]
    fn test_verify_shards() {
        let config = redundancy::RedundancyConfig::new(3, 2);
        let shards = redundancy::split_data(&[1u8, 2, 3, 4], &config);
        assert!(redundancy::verify_shards(&shards));
    }

    // ---- fragment tests (8) ----

    #[test]
    fn test_fragment_new() {
        let f = fragment::Fragment::new(1, vec![0.5, 0.6], "source_a");
        assert_eq!(f.id, 1);
        assert_eq!(f.source_location, "source_a");
    }

    #[test]
    fn test_fragment_with_metadata() {
        let f = fragment::Fragment::new(1, vec![0.5], "src")
            .with_metadata("key", "value");
        assert_eq!(f.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_fragment_energy() {
        let f = fragment::Fragment::new(1, vec![3.0, 4.0], "src");
        assert!((f.energy() - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_fragment_normalize() {
        let mut f = fragment::Fragment::new(1, vec![3.0, 6.0], "src");
        f.normalize();
        assert!((f.data[0] - 0.5).abs() < 0.01);
        assert!((f.data[1] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_fragment_crop() {
        let f = fragment::Fragment::new(1, vec![1.0, 2.0, 3.0, 4.0], "src");
        let cropped = f.crop(1, 3);
        assert_eq!(cropped.data, vec![2.0, 3.0]);
    }

    #[test]
    fn test_fragment_merge() {
        let a = fragment::Fragment::new(1, vec![1.0], "src");
        let b = fragment::Fragment::new(1, vec![2.0], "src");
        let merged = a.merge(&b);
        assert_eq!(merged.data, vec![1.0, 2.0]);
    }

    #[test]
    fn test_fragment_correlation_identical() {
        let f = fragment::Fragment::new(1, vec![1.0, 2.0, 3.0], "src");
        assert!((f.correlation(&f) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_fragment_store() {
        let mut store = fragment::FragmentStore::new();
        store.add(fragment::Fragment::new(1, vec![0.5], "a"));
        store.add(fragment::Fragment::new(2, vec![0.8], "b"));
        assert_eq!(store.len(), 2);
        assert!(store.contains(1));
        assert_eq!(store.get(1).unwrap().source_location, "a");
    }

    #[test]
    fn test_fragment_store_find_by_source() {
        let mut store = fragment::FragmentStore::new();
        store.add(fragment::Fragment::new(1, vec![0.5], "alpha"));
        store.add(fragment::Fragment::new(2, vec![0.8], "beta"));
        store.add(fragment::Fragment::new(3, vec![0.3], "alpha"));
        let found = store.find_by_source("alpha");
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_fragment_to_from_bytes() {
        let f = fragment::Fragment::new(1, vec![0.5, 1.0], "src");
        let bytes = f.to_bytes();
        let restored = fragment::Fragment::from_bytes(1, &bytes, "src");
        assert_eq!(restored.data.len(), 2);
    }

    #[test]
    fn test_fragment_store_high_energy() {
        let mut store = fragment::FragmentStore::new();
        store.add(fragment::Fragment::new(1, vec![0.1], "s"));
        store.add(fragment::Fragment::new(2, vec![5.0, 5.0], "s"));
        let high = store.find_high_energy(10.0);
        assert_eq!(high.len(), 1);
        assert_eq!(high[0].id, 2);
    }
}
