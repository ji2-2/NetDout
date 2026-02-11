use crate::{
    config::AppConfig,
    db::{ChunkState, ResumeStore},
    models::{ChunkPlan, DownloadInfo, DownloadRequest, DownloadStatus},
    network::HttpClient,
    scheduler::AdaptiveScheduler,
};
use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use reqwest::header::RANGE;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    fs::{self, File},
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::RwLock,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct DownloadEngine {
    state: Arc<RwLock<HashMap<String, DownloadInfo>>>,
    client: HttpClient,
    scheduler: AdaptiveScheduler,
    config: AppConfig,
    db: Arc<ResumeStore>,
}

impl DownloadEngine {
    pub fn new(config: AppConfig, db: Arc<ResumeStore>) -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            client: HttpClient::new(),
            scheduler: AdaptiveScheduler::new(config.worker_threads),
            config,
            db,
        }
    }

    pub async fn enqueue(&self, url: String, output: String) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let info = DownloadInfo {
            id: id.clone(),
            url: url.clone(),
            output: PathBuf::from(output.clone()),
            total_bytes: None,
            downloaded_bytes: 0,
            status: DownloadStatus::Queued,
        };
        self.state.write().await.insert(id.clone(), info);

        let this = self.clone();
        tokio::spawn(async move {
            if let Err(err) = this
                .run_download(
                    id.clone(),
                    DownloadRequest {
                        url,
                        output: output.into(),
                    },
                )
                .await
            {
                let mut guard = this.state.write().await;
                if let Some(entry) = guard.get_mut(&id) {
                    entry.status = DownloadStatus::Failed(err.to_string());
                }
            }
        });

        Ok(id)
    }

    pub async fn status(&self, id: &str) -> Option<DownloadInfo> {
        self.state.read().await.get(id).cloned()
    }

    async fn run_download(&self, id: String, request: DownloadRequest) -> Result<()> {
        self.update_status(&id, DownloadStatus::Running).await;

        let metadata = self.client.probe(&request.url).await?;
        self.set_total_bytes(&id, metadata.content_length).await;

        if metadata.range_supported && metadata.content_length.is_some() {
            self.download_chunked(&id, &request, metadata.content_length.unwrap())
                .await?;
        } else {
            self.download_single(&id, &request).await?;
        }

        self.update_status(&id, DownloadStatus::Completed).await;
        Ok(())
    }

    async fn download_single(&self, id: &str, request: &DownloadRequest) -> Result<()> {
        let bytes = self
            .client
            .client
            .get(&request.url)
            .send()
            .await?
            .bytes()
            .await?;
        fs::write(&request.output, &bytes).await?;
        self.update_progress(id, bytes.len() as u64).await;
        Ok(())
    }

    async fn download_chunked(
        &self,
        id: &str,
        request: &DownloadRequest,
        total: u64,
    ) -> Result<()> {
        let chunks = build_chunk_plan(total, self.config.chunk_size);
        let parallelism = self.scheduler.choose_parallelism(Some(total));
        let part_dir = chunk_dir_for(&request.output);
        fs::create_dir_all(&part_dir).await?;

        let resume_map = self
            .db
            .load_chunk_state(id)?
            .into_iter()
            .map(|s| (s.chunk_index, s))
            .collect::<HashMap<_, _>>();

        stream::iter(chunks.clone())
            .map(|chunk| {
                let url = request.url.clone();
                let id = id.to_string();
                let part_dir = part_dir.clone();
                let client = self.client.client.clone();
                let db = self.db.clone();
                let state = self.state.clone();
                let already = resume_map.get(&chunk.index).cloned();

                async move {
                    let part_path = part_dir.join(format!("chunk-{}.part", chunk.index));
                    let mut downloaded = already.as_ref().map(|s| s.downloaded).unwrap_or(0);
                    if already.as_ref().map(|s| s.complete).unwrap_or(false) {
                        return Ok::<(), anyhow::Error>(());
                    }

                    let mut file = open_chunk_file(&part_path).await?;
                    file.seek(std::io::SeekFrom::Start(downloaded)).await?;

                    let range_start = chunk.start + downloaded;
                    if range_start > chunk.end_inclusive {
                        return Ok(());
                    }

                    let range = format!("bytes={}-{}", range_start, chunk.end_inclusive);
                    let mut stream = client
                        .get(&url)
                        .header(RANGE, range)
                        .send()
                        .await?
                        .bytes_stream();

                    while let Some(next) = stream.next().await {
                        let bytes = next?;
                        file.write_all(&bytes).await?;
                        downloaded += bytes.len() as u64;

                        db.save_chunk_state(&ChunkState {
                            download_id: id.clone(),
                            chunk_index: chunk.index,
                            downloaded,
                            complete: false,
                        })?;

                        let mut guard = state.write().await;
                        if let Some(info) = guard.get_mut(&id) {
                            info.downloaded_bytes += bytes.len() as u64;
                        }
                    }

                    db.save_chunk_state(&ChunkState {
                        download_id: id,
                        chunk_index: chunk.index,
                        downloaded,
                        complete: true,
                    })?;
                    Ok(())
                }
            })
            .buffer_unordered(parallelism)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>>>()?;

        merge_chunks(&part_dir, &request.output, chunks.len()).await?;
        Ok(())
    }

    async fn update_status(&self, id: &str, status: DownloadStatus) {
        if let Some(entry) = self.state.write().await.get_mut(id) {
            entry.status = status;
        }
    }

    async fn set_total_bytes(&self, id: &str, total: Option<u64>) {
        if let Some(entry) = self.state.write().await.get_mut(id) {
            entry.total_bytes = total;
        }
    }

    async fn update_progress(&self, id: &str, downloaded: u64) {
        if let Some(entry) = self.state.write().await.get_mut(id) {
            entry.downloaded_bytes = downloaded;
        }
    }
}

pub fn build_chunk_plan(total: u64, chunk_size: u64) -> Vec<ChunkPlan> {
    let mut plans = Vec::new();
    let mut start = 0;
    let mut index = 0;

    while start < total {
        let end = (start + chunk_size).min(total) - 1;
        plans.push(ChunkPlan {
            index,
            start,
            end_inclusive: end,
        });
        start = end + 1;
        index += 1;
    }

    plans
}

fn chunk_dir_for(output: &Path) -> PathBuf {
    let filename = output
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("download.bin");
    output
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".{}.chunks", filename))
}

async fn open_chunk_file(path: &Path) -> Result<File> {
    Ok(tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .read(true)
        .open(path)
        .await
        .with_context(|| format!("open chunk file: {}", path.display()))?)
}

pub async fn merge_chunks(part_dir: &Path, output: &Path, count: usize) -> Result<()> {
    let tmp = output.with_extension("download_tmp");
    let mut target = File::create(&tmp).await?;

    for idx in 0..count {
        let part_path = part_dir.join(format!("chunk-{}.part", idx));
        let bytes = fs::read(&part_path).await?;
        target.write_all(&bytes).await?;
    }

    target.flush().await?;
    fs::rename(tmp, output).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_plan_covers_exact_size() {
        let chunks = build_chunk_plan(10, 4);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].start, 0);
        assert_eq!(chunks[2].end_inclusive, 9);
    }
}
