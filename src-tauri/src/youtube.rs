use anyhow::Result;
use std::sync::{Arc, Mutex};
use reqwest::Client;
use url::Url;
use serde::Deserialize;
use futures::stream::try_unfold;
use futures::{Stream, TryStreamExt};
use crate::models::CommentData;

#[derive(Deserialize, Debug)]
pub struct CommentThreadResponse {
    pub items: Vec<CommentThread>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CommentThread {
    pub snippet: CommentThreadSnippet,
}

#[derive(Deserialize, Debug)]
pub struct CommentThreadSnippet {
    #[serde(rename = "topLevelComment")]
    pub top_level_comment: Comment,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    pub id: String,
    pub snippet: CommentSnippet,
}

#[derive(Deserialize, Debug)]
pub struct CommentSnippet {
    #[serde(rename = "textOriginal")]
    pub text_original: String,
}

pub struct YoutubeFetcher<'a> {
    client: &'a Client,
    api_key: &'a str,
}

impl<'a> YoutubeFetcher<'a> {
    pub fn new(client: &'a Client, api_key: &'a str) -> Self {
        YoutubeFetcher { client, api_key }
    }

    pub async fn retrieve_comments_stream(
        &self,
        youtube_url: &str,
    ) -> Result<Vec<CommentData>> {
        let video_id = match Self::extract_video_id(youtube_url) {
            Some(id) => id,
            None => {
                return Err(anyhow::anyhow!("Failed to extract video ID from URL: {}", youtube_url));
            }
        };

        let comments = Arc::new(Mutex::new(Vec::new()));
        let stream = self.create_comments_stream(&video_id, &comments);
        Self::find_max_page_count(stream).await?;
        Ok(Arc::try_unwrap(comments).unwrap().into_inner().unwrap_or_default())
    }

    fn extract_video_id(youtube_url: &str) -> Option<String> {
        let url = Url::parse(youtube_url).ok()?;

        if url.host_str() == Some("youtu.be") {
            url.path_segments().and_then(|segments| segments.last().map(|id| id.to_string()))
        } else {
            url.query_pairs().find(|(key, _)| key == "v").map(|(_, value)| value.into_owned())
        }
    }

    fn create_comments_stream<'b>(
        &'b self,
        video_id: &'b str,
        comments: &'b Arc<Mutex<Vec<CommentData>>>,
    ) -> impl Stream<Item = Result<i32, anyhow::Error>> + 'b {
        try_unfold((None, 0), {
            let comments = Arc::clone(comments);
            move |(page_token, page_count)| {
                let video_id = video_id.to_owned();
                let comments = Arc::clone(&comments);
                async move {
                    let result = self
                        .process_comments_page(&video_id, page_token, page_count, &mut comments.lock().unwrap())
                        .await?;
                    Ok(result.map(|(next_page_token, next_page_count)| {
                        (next_page_count, (next_page_token, next_page_count))
                    }))
                }
            }
        })
    }

    async fn process_comments_page(
        &self,
        video_id: &str,
        page_token: Option<String>,
        page_count: i32,
        comments: &mut Vec<CommentData>,
    ) -> Result<Option<(Option<String>, i32)>> {
        println!("Fetching page {} of comments...", page_count + 1);
        let response = self.fetch_comments_page(video_id, &page_token).await?;
        println!("Retrieved {} comments.", response.items.len());

        Self::insert_comments(response.items, comments);

        let next_page_token = response.next_page_token;
        let next_page_count = page_count + 1;

        Ok(next_page_token.map(|token| (Some(token), next_page_count)))
    }

    async fn fetch_comments_page(
        &self,
        video_id: &str,
        page_token: &Option<String>,
    ) -> Result<CommentThreadResponse> {
        let url = self.build_comments_url(video_id, page_token);
        let response = self.client.get(&url).send().await?;
        let comment_thread_response: CommentThreadResponse = response.json().await?;
        println!("URL: {}", url);
        Ok(comment_thread_response)
    }

    fn build_comments_url(&self, video_id: &str, page_token: &Option<String>) -> String {
        format!(
            "https://www.googleapis.com/youtube/v3/commentThreads?part=snippet&videoId={}&maxResults=100&key={}{}",
            video_id,
            self.api_key,
            page_token
                .as_ref()
                .map_or(String::new(), |token| format!("&pageToken={}", token))
        )
    }

    fn insert_comments(items: Vec<CommentThread>, comments: &mut Vec<CommentData>) {
        items.into_iter().for_each(|comment_thread| {
            let comment_id = comment_thread.snippet.top_level_comment.id;
            let comment_text = comment_thread.snippet.top_level_comment.snippet.text_original;
            comments.push(CommentData {
                id: comment_id,
                text: comment_text,
                sentiment: String::new(),
                score: 0.0,
            });
        });
    }

    async fn find_max_page_count(
        stream: impl Stream<Item = Result<i32, anyhow::Error>>,
    ) -> Result<i32> {
        stream
            .try_fold(0, |max_page_count, page_count| async move {
                if page_count > max_page_count {
                    Ok(page_count)
                } else {
                    Ok(max_page_count)
                }
            })
            .await
    }
}