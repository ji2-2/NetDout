#[derive(Clone, Debug)]
pub struct AppConfig {
    pub api_bind_addr: String,
    pub database_path: String,
    pub worker_threads: usize,
    pub chunk_size: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_bind_addr: "127.0.0.1:8472".to_string(),
            database_path: "netdout.db".to_string(),
            worker_threads: 8,
            chunk_size: 2 * 1024 * 1024,
        }
    }
}
