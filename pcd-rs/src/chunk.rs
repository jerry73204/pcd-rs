//! Chunked streaming API for memory-efficient point cloud processing.
//!
//! This module provides APIs for processing large PCD files in chunks,
//! enabling parallel processing and bounded memory usage.

use crate::{
    error::{Error, Result},
    metas::PcdMeta,
    reader::Reader,
    record::PcdDeserialize,
    DynRecord,
};
use std::{
    io::BufRead,
    marker::PhantomData,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

/// Progress information for chunk processing
#[derive(Debug, Clone)]
pub struct ChunkProgress {
    /// Current chunk index (0-based)
    pub current_chunk: usize,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Points processed so far
    pub points_processed: usize,
    /// Total points in file
    pub total_points: usize,
    /// Percentage complete (0.0 to 100.0)
    pub percentage: f32,
}

/// Callback for progress updates
pub type ProgressCallback = Box<dyn Fn(ChunkProgress) + Send + Sync>;

/// Configuration for chunked reading
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Number of points per chunk
    pub chunk_size: usize,
    /// Enable parallel processing of chunks
    pub parallel: bool,
    /// Maximum number of threads for parallel processing (None = use all cores)
    pub max_threads: Option<usize>,
    /// Buffer size for pre-loading chunks (for parallel processing)
    pub buffer_chunks: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 10_000,
            parallel: false,
            max_threads: None,
            buffer_chunks: 2,
        }
    }
}

/// A chunk of point cloud data
pub struct Chunk<Record> {
    /// Chunk index (0-based)
    pub index: usize,
    /// Points in this chunk
    pub points: Vec<Record>,
    /// Starting point index in the original file
    pub start_index: usize,
    /// Whether this is the last chunk
    pub is_last: bool,
}

/// Chunked reader for point cloud data
pub struct ChunkedReader<Record, R>
where
    R: BufRead,
{
    reader: Reader<Record, R>,
    config: ChunkConfig,
    points_read: AtomicUsize,
    current_chunk: AtomicUsize,
    total_chunks: usize,
    cancelled: Arc<AtomicBool>,
    progress_callback: Arc<Mutex<Option<ProgressCallback>>>,
    _phantom: PhantomData<Record>,
}

impl<Record, R> ChunkedReader<Record, R>
where
    R: BufRead,
    Record: PcdDeserialize,
{
    /// Create a new chunked reader from an existing reader
    pub fn new(reader: Reader<Record, R>, config: ChunkConfig) -> Self {
        let total_points = reader.meta().num_points as usize;
        let total_chunks = total_points.div_ceil(config.chunk_size);

        Self {
            reader,
            config,
            points_read: AtomicUsize::new(0),
            current_chunk: AtomicUsize::new(0),
            total_chunks,
            cancelled: Arc::new(AtomicBool::new(false)),
            progress_callback: Arc::new(Mutex::new(None)),
            _phantom: PhantomData,
        }
    }

    /// Get metadata
    pub fn meta(&self) -> &PcdMeta {
        self.reader.meta()
    }

    /// Set progress callback
    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(ChunkProgress) + Send + Sync + 'static,
    {
        *self.progress_callback.lock().unwrap() = Some(Box::new(callback));
    }

    /// Cancel processing
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// Check if processing was cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// Report progress
    fn report_progress(&self) {
        let current_chunk = self.current_chunk.load(Ordering::Relaxed);
        let points_processed = self.points_read.load(Ordering::Relaxed);
        let total_points = self.reader.meta().num_points as usize;

        let progress = ChunkProgress {
            current_chunk,
            total_chunks: self.total_chunks,
            points_processed,
            total_points,
            percentage: (points_processed as f32 / total_points as f32) * 100.0,
        };

        if let Some(ref callback) = *self.progress_callback.lock().unwrap() {
            callback(progress);
        }
    }

    /// Read the next chunk
    pub fn read_chunk(&mut self) -> Result<Option<Chunk<Record>>> {
        if self.is_cancelled() {
            return Ok(None);
        }

        let chunk_index = self.current_chunk.load(Ordering::Relaxed);
        if chunk_index >= self.total_chunks {
            return Ok(None);
        }

        let start_index = self.points_read.load(Ordering::Relaxed);
        let mut points = Vec::with_capacity(self.config.chunk_size);

        for _ in 0..self.config.chunk_size {
            if self.is_cancelled() {
                break;
            }

            match self.reader.next() {
                Some(Ok(point)) => points.push(point),
                Some(Err(e)) => return Err(e),
                None => break,
            }
        }

        if points.is_empty() {
            return Ok(None);
        }

        let points_in_chunk = points.len();
        self.points_read
            .fetch_add(points_in_chunk, Ordering::Relaxed);
        self.current_chunk.fetch_add(1, Ordering::Relaxed);

        let is_last = chunk_index == self.total_chunks - 1;

        self.report_progress();

        Ok(Some(Chunk {
            index: chunk_index,
            points,
            start_index,
            is_last,
        }))
    }

    /// Process all chunks with a function
    pub fn for_each_chunk<F>(&mut self, mut processor: F) -> Result<()>
    where
        F: FnMut(Chunk<Record>) -> Result<()>,
    {
        while let Some(chunk) = self.read_chunk()? {
            processor(chunk)?;
        }
        Ok(())
    }

    /// Process all chunks and collect results
    pub fn map_chunks<F, T>(&mut self, mut mapper: F) -> Result<Vec<T>>
    where
        F: FnMut(Chunk<Record>) -> Result<T>,
    {
        let mut results = Vec::with_capacity(self.total_chunks);

        while let Some(chunk) = self.read_chunk()? {
            results.push(mapper(chunk)?);
        }

        Ok(results)
    }
}

/// Parallel chunked reader using rayon
#[cfg(feature = "parallel")]
pub struct ParallelChunkedReader<Record> {
    meta: PcdMeta,
    path: std::path::PathBuf,
    config: ChunkConfig,
    cancelled: Arc<AtomicBool>,
    progress: Arc<AtomicUsize>,
    _phantom: PhantomData<Record>,
}

#[cfg(feature = "parallel")]
impl<Record> ParallelChunkedReader<Record>
where
    Record: PcdDeserialize + Send + Sync,
{
    /// Create a new parallel chunked reader
    pub fn open<P: AsRef<Path>>(path: P, config: ChunkConfig) -> Result<Self> {
        use std::{fs::File, io::BufReader};

        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let mut buf_reader = BufReader::new(file);

        // Read metadata
        let meta = crate::utils::load_meta(&mut buf_reader, &mut 0)?;

        Ok(Self {
            meta,
            path,
            config,
            cancelled: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(AtomicUsize::new(0)),
            _phantom: PhantomData,
        })
    }

    /// Process chunks in parallel
    pub fn par_for_each_chunk<F>(&self, processor: F) -> Result<()>
    where
        F: Fn(Chunk<Record>) -> Result<()> + Send + Sync,
    {
        use rayon::prelude::*;

        let total_points = self.meta.num_points as usize;
        let total_chunks = total_points.div_ceil(self.config.chunk_size);

        // Set thread pool size if specified
        if let Some(max_threads) = self.config.max_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
                .map_err(|e| {
                    Error::IoError(std::io::Error::other(format!(
                        "Failed to create thread pool: {}",
                        e
                    )))
                })?;
        }

        // Process chunks in parallel
        (0..total_chunks)
            .into_par_iter()
            .try_for_each(|chunk_index| {
                if self.cancelled.load(Ordering::Relaxed) {
                    return Ok(());
                }

                // Each thread opens its own reader
                let file = std::fs::File::open(&self.path)?;
                let buf_reader = std::io::BufReader::new(file);
                let mut reader = Reader::<Record, _>::from_reader(buf_reader)?;

                // Skip to the correct chunk
                let start_index = chunk_index * self.config.chunk_size;
                for _ in 0..start_index {
                    if reader.next().is_none() {
                        return Ok(());
                    }
                }

                // Read the chunk
                let mut points = Vec::with_capacity(self.config.chunk_size);
                for _ in 0..self.config.chunk_size {
                    match reader.next() {
                        Some(Ok(point)) => points.push(point),
                        Some(Err(e)) => return Err(e),
                        None => break,
                    }
                }

                if !points.is_empty() {
                    let chunk = Chunk {
                        index: chunk_index,
                        points,
                        start_index,
                        is_last: chunk_index == total_chunks - 1,
                    };

                    processor(chunk)?;

                    // Update progress
                    self.progress.fetch_add(1, Ordering::Relaxed);
                }

                Ok(())
            })
    }

    /// Cancel parallel processing
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

/// Dynamic chunked reader for untyped records
pub type DynChunkedReader<R> = ChunkedReader<DynRecord, R>;

/// Builder for creating chunked readers with configuration
pub struct ChunkedReaderBuilder {
    config: ChunkConfig,
}

impl ChunkedReaderBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: ChunkConfig::default(),
        }
    }

    /// Set chunk size (number of points per chunk)
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.config.chunk_size = size;
        self
    }

    /// Enable parallel processing
    pub fn parallel(mut self, enable: bool) -> Self {
        self.config.parallel = enable;
        self
    }

    /// Set maximum number of threads
    pub fn max_threads(mut self, threads: usize) -> Self {
        self.config.max_threads = Some(threads);
        self
    }

    /// Set buffer size for pre-loading chunks
    pub fn buffer_chunks(mut self, chunks: usize) -> Self {
        self.config.buffer_chunks = chunks;
        self
    }

    /// Build a chunked reader from a path
    pub fn open<P: AsRef<Path>>(
        self,
        path: P,
    ) -> Result<DynChunkedReader<std::io::BufReader<std::fs::File>>> {
        use crate::DynReader;
        let reader = DynReader::open(path)?;
        Ok(ChunkedReader::new(reader, self.config))
    }

    /// Build a chunked reader from an existing reader
    pub fn from_reader<Record, R>(self, reader: Reader<Record, R>) -> ChunkedReader<Record, R>
    where
        R: BufRead,
        Record: PcdDeserialize,
    {
        ChunkedReader::new(reader, self.config)
    }
}

impl Default for ChunkedReaderBuilder {
    fn default() -> Self {
        Self::new()
    }
}
