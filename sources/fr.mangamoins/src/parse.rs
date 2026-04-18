use core::str::FromStr;

use aidoku::alloc::vec;
use aidoku::{
    AidokuError, Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Result,
    UpdateStrategy, Viewer,
    alloc::{String, format, string::ToString, vec::Vec},
    imports::net::{Request, RequestError},
};

use serde::{Deserialize, Serialize};

use crate::{BASE_URL, USER_AGENT};

#[derive(Debug, Serialize, Deserialize)]
pub struct AllMangas {
    pub status: String,
    pub search_query: String,
    pub total: i32,
    pub page: i32,
    pub limit: i32,
    pub data: Vec<AllMangasChapter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllMangasChapter {
    pub title: String,
    pub chapitre: String,
    pub slug: String,
    pub cover: String,
    #[serde(rename = "mangaSlug")]
    pub manga_slug: String,
    pub timestamp: i64,
    pub time_human: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaMoinsRoot {
    pub info: MangaMoinsInfo,
    pub chapters: Vec<MangaMoinsChapter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaMoinsInfo {
    pub title: String,
    pub author: String,
    pub status: String,
    pub cover: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaMoinsChapter {
    pub slug: String,
    pub num: u32,
    pub title: String,
    pub time: u64,
    pub keywords: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterContent {
    pub slug: String,

    #[serde(rename = "pageNumbers")]
    pub page_numbers: u32,

    #[serde(rename = "chapterTitle")]
    pub chapter_title: String,

    #[serde(rename = "chapterNum")]
    pub chapter_num: u32,

    #[serde(rename = "pagesBaseUrl")]
    pub pages_base_url: String,

    pub manga: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchApiResponse {
    pub status: String,
    pub total: u32,
    pub page: u32,
    pub limit: u32,
    pub data: Vec<SearchManga>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchManga {
    pub title: String,
    pub slug: String,
    pub author: String,
    pub status: String,
    pub genres: Vec<String>,
    pub cover: String,
    pub synopsis: String,
}

fn extract_chapter(manga: &str) -> Result<Manga> {
    let url = format!("{BASE_URL}/api/v1/manga?manga={manga}");
    let response = Request::get(&url)?
        .header("Referer", BASE_URL)
        .header("User-Agent", USER_AGENT)
        .string()?;

    let parsed: MangaMoinsRoot = serde_json::from_str(&response)?;
    let status: MangaStatus = MangaStatusWrapper::from_str(&parsed.info.status)?.0;
    let authors = vec![parsed.info.author];

    let chapters: Vec<aidoku::Chapter> = parsed
        .chapters
        .into_iter()
        .map(|c| Chapter {
            key: c.slug,
            title: Some(c.title),
            #[allow(clippy::cast_precision_loss)]
            chapter_number: Some(c.num as f32),
            volume_number: None,
            date_uploaded: Some(c.time.cast_signed()),
            scanlators: None,
            url: None,
            language: None,
            thumbnail: None,
            locked: false,
        })
        .collect();

    Ok(Manga {
        key: manga.to_string(),
        title: parsed.info.title,
        cover: Some(parsed.info.cover),
        artists: None,
        authors: Some(authors),
        description: Some(parsed.info.description),
        url: Some(url),
        tags: None,
        status,
        content_rating: ContentRating::Unknown,
        viewer: Viewer::Unknown,
        update_strategy: UpdateStrategy::Always,
        next_update_time: None,
        chapters: Some(chapters),
    })
}

struct MangaStatusWrapper(pub MangaStatus);

impl FromStr for MangaStatusWrapper {
    type Err = AidokuError;

    fn from_str(s: &str) -> Result<Self> {
        let normalized = s.trim().to_lowercase();

        let status = match normalized.as_str() {
            "hiatus" => MangaStatus::Hiatus,

            // French → English mapping
            "en cours" => MangaStatus::Ongoing,
            "terminé" | "termine" => MangaStatus::Completed,

            _ => MangaStatus::Unknown,
        };

        Ok(Self(status))
    }
}

pub fn get_all_releases(response: &str) -> Result<MangaPageResult> {
    let mut entries: Vec<Manga> = Vec::new();

    let parsed: AllMangas = serde_json::from_str(response)?;

    if parsed.status != "success" {
        return Err(AidokuError::RequestError(RequestError::InvalidString));
    }

    for els in parsed.data {
        entries.push(extract_chapter(&els.manga_slug)?);
    }

    Ok(MangaPageResult {
        entries,
        has_next_page: false,
    })
}

pub fn search_manga(response: &str) -> Result<Manga> {
    let parsed: SearchApiResponse = serde_json::from_str(response)?;

    if parsed.status != "success" {
        return Err(AidokuError::Message("error with API request".to_string()));
    }

    extract_chapter(&parsed.data[0].slug)
}

pub fn search(response: &str) -> Result<MangaPageResult> {
    let mut entries: Vec<Manga> = Vec::new();

    let parsed: SearchApiResponse = serde_json::from_str(response)?;

    if parsed.status != "success" {
        return Err(AidokuError::Message("error with API request".to_string()));
    }

    for els in parsed.data {
        entries.push(extract_chapter(&els.slug)?);
    }

    Ok(MangaPageResult {
        entries,
        has_next_page: false,
    })
}
