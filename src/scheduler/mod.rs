#[derive(Debug, Clone)]
pub struct AdaptiveScheduler {
    pub max_parallel_chunks: usize,
}

impl AdaptiveScheduler {
    pub fn new(max_parallel_chunks: usize) -> Self {
        Self {
            max_parallel_chunks,
        }
    }

    /// Very simple adaptive policy placeholder; can be expanded with throughput telemetry.
    pub fn choose_parallelism(&self, file_size: Option<u64>) -> usize {
        match file_size {
            Some(size) if size < 8 * 1024 * 1024 => 1,
            Some(size) if size < 64 * 1024 * 1024 => self.max_parallel_chunks.min(4),
            _ => self.max_parallel_chunks,
        }
    }
}
