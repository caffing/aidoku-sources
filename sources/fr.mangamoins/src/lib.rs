#![no_std]
use aidoku::{
    AidokuError, Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Home, HomeLayout, Listing,
    ListingProvider, Manga, MangaPageResult, Page, PageContent, Result, Source,
    alloc::{String, Vec, format},
    helpers::uri::encode_uri,
    imports::net::Request,
    register_source,
};

use crate::parse::{ChapterContent, get_all_releases, search, search_manga};

pub mod parse;

const BASE_URL: &str = "https://mangamoins.com";
pub const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64; rv:149.0) Gecko/20100101 Firefox/149.0";

struct MangaMoins;

impl Source for MangaMoins {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        _page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let url = format!("{BASE_URL}/api/v1/mangas");

        let results = if let Some(q) = &query {
            let body = Request::get(format!(
                "{BASE_URL}/api/v1/explore?q={}&page=1&limit=20",
                encode_uri(q.to_lowercase())
            ))?
            .header("Referer", BASE_URL)
            .header("User-Agent", USER_AGENT)
            .string()?;
            search(&body)?
        } else {
            let body = Request::get(url)?
                .header("Referer", BASE_URL)
                .header("User-Agent", USER_AGENT)
                .string()?;
            get_all_releases(&body)?
        };

        Ok(results)
    }

    fn get_manga_update(
        &self,
        manga: Manga,
        _needs_details: bool,
        _needs_chapters: bool,
    ) -> Result<Manga> {
        let url = format!("{BASE_URL}/api/v1/explore?q={}&page=1&limit=20", manga.key);
        let response = Request::get(&url)?
            .header("Referer", BASE_URL)
            .header("User-Agent", USER_AGENT)
            .string()?;

        search_manga(&response)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let response = Request::get(format!("{BASE_URL}/api/v1/scan?slug={}", chapter.key))?
            .header("Referer", BASE_URL)
            .header("User-Agent", USER_AGENT)
            .string()?;

        let parsed: ChapterContent = serde_json::from_str(&response)?;

        let mut pages = Vec::new();

        for i in 1..parsed.page_numbers {
            pages.push(Page {
                content: PageContent::url(format!(
                    "{}/{i:02}.webp",
                    parsed.pages_base_url.trim_end_matches('/')
                )),
                ..Default::default()
            });
        }

        Ok(pages)
    }
}

impl ListingProvider for MangaMoins {
    fn get_manga_list(&self, _listing: Listing, _page: i32) -> Result<MangaPageResult> {
        Err(AidokuError::Unimplemented)
    }
}

impl Home for MangaMoins {
    fn get_home(&self) -> Result<HomeLayout> {
        Err(AidokuError::Unimplemented)
    }
}

impl DeepLinkHandler for MangaMoins {
    fn handle_deep_link(&self, _url: String) -> Result<Option<DeepLinkResult>> {
        Ok(None)
    }
}

register_source!(MangaMoins, ListingProvider, Home, DeepLinkHandler);

#[cfg(test)]
mod tests {

    use aidoku_test::aidoku_test;

    use super::*;

    #[aidoku_test]
    fn test_get_all() {
        let a = MangaMoins::new();

        let f = a.get_search_manga_list(None, 1, std::vec![]).unwrap();

        let page_update = a
            .get_manga_update(f.clone().entries[0].clone(), false, false)
            .unwrap();

        assert!(page_update.chapters.is_some());

        let chapers = f.clone().entries[0].clone().chapters.unwrap()[0].clone();

        let f = a
            .get_page_list(f.clone().entries[0].clone(), chapers)
            .unwrap();

        assert!(f.len() > 0);
    }
}
