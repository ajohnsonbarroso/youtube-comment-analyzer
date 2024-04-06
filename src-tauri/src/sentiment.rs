use pyo3::prelude::*;
use rayon::prelude::*;
use std::thread;
use crate::models::CommentData;
use once_cell::sync::OnceCell;

const NUM_THREADS: usize = 6;
const MAX_COMMENT_LENGTH: usize = 1850;

pub struct SentimentAnalyzer {
    analyzer: PyObject,
}

impl SentimentAnalyzer {
    pub fn initialize_analyzer() -> Result<(), String> {
        Python::with_gil(|py| {
            match Self::setup_analyzer(py) {
                Ok(_) => {
                    println!("Sentiment analyzer initialized successfully");
                    Ok(())
                }
                Err(e) => {
                    let error_message = format!("Error initializing sentiment analyzer: {}", e);
                    println!("{}", error_message);
                    Err(error_message)
                }
            }
        })
    }

    fn setup_analyzer(py: Python) -> PyResult<()> {
        let transformers = py.import_bound("transformers")?;
        let analyzer = transformers.getattr("pipeline")?.call1((
            "text-classification",
            "lxyuan/distilbert-base-multilingual-cased-sentiments-student",
        ))?;
        ANALYZER.set(analyzer.into()).map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Failed to set sentiment analyzer")
        })?;
        Ok(())
    }

    pub fn get_analyzer() -> Self {
        SentimentAnalyzer {
            analyzer: ANALYZER.get().unwrap().clone(),
        }
    }

    pub fn analyze_comments(&self, comments: &[CommentData]) -> PyResult<Vec<CommentData>> {
        let analyzer = &self.analyzer;

        let num_threads = thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::new(1).unwrap());
        println!("Number of available threads: {}", num_threads);

        Python::with_gil(|py| {
            let result = py.allow_threads(move || {
                self.process_comments_in_parallel(comments, analyzer)
            });

            result
        })
    }

    fn process_comments_in_parallel(
        &self,
        comments: &[CommentData],
        analyzer: &PyObject,
    ) -> PyResult<Vec<CommentData>> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .unwrap();

        pool.install(|| {
            comments
                .par_iter()
                .map(|comment_data| self.process_single_comment(comment_data, analyzer))
                .collect()
        })
    }

    fn process_single_comment(
        &self,
        comment_data: &CommentData,
        analyzer: &PyObject,
    ) -> PyResult<CommentData> {
        let thread_id = thread::current().id();
        println!("Processing comment in thread: {:?}", thread_id);

        Python::with_gil(|py| {
            let trimmed_text = self.trim_comment_text(&comment_data.text);

            let analyzer = analyzer.bind(py);
            let result = analyzer.call1((trimmed_text,))?;
            let sentiment_dict = result.get_item(0)?;
            let sentiment_label: String = sentiment_dict.get_item("label")?.extract()?;
            let sentiment_score: f64 = sentiment_dict.get_item("score")?.extract()?;

            Ok(CommentData {
                id: comment_data.id.to_owned(),
                text: comment_data.text.to_owned(),
                sentiment: sentiment_label,
                score: sentiment_score,
            })
        })
    }

    fn trim_comment_text(&self, text: &str) -> String {
        text.chars().take(MAX_COMMENT_LENGTH).collect()
    }
}

lazy_static::lazy_static! {
    static ref ANALYZER: OnceCell<PyObject> = OnceCell::new();
}