#![no_std]
use aidoku::{
    AidokuError, Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Home, HomeLayout, Listing,
    ListingProvider, Manga, MangaPageResult, Page, Result, Source,
    alloc::{String, Vec},
    helpers::uri::encode_uri,
    imports::net::Request,
    prelude::*,
};

pub mod parse;

use helper::{USER_AGENT, get_base_url};

use crate::parse::get_all_releases;

struct AnimeSama;

impl Source for AnimeSama {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let url = get_base_url()?;
        let body = match &query {
            Some(q) => Request::get(format!(
                "{url}/catalogue/?type[]=Scans&search={}",
                encode_uri(q.to_lowercase())
            ))?,
            None => Request::get(format!("{url}/catalogue/?type[]=Scans"))?,
        }
        .header("Referer", &url)
        .header("User-Agent", USER_AGENT)
        .string()?;

        let results = get_all_releases(&body, page)?;

        Ok(results)
    }

    fn get_manga_update(
        &self,
        _manga: Manga,
        _needs_details: bool,
        _needs_chapters: bool,
    ) -> Result<Manga> {
        Err(AidokuError::Unimplemented)
    }

    fn get_page_list(&self, _manga: Manga, _chapter: Chapter) -> Result<Vec<Page>> {
        Err(AidokuError::Unimplemented)
    }
}

impl ListingProvider for AnimeSama {
    fn get_manga_list(&self, _listing: Listing, _page: i32) -> Result<MangaPageResult> {
        Err(AidokuError::Unimplemented)
    }
}

impl Home for AnimeSama {
    fn get_home(&self) -> Result<HomeLayout> {
        Err(AidokuError::Unimplemented)
    }
}

impl DeepLinkHandler for AnimeSama {
    fn handle_deep_link(&self, _url: String) -> Result<Option<DeepLinkResult>> {
        Err(AidokuError::Unimplemented)
    }
}

register_source!(AnimeSama, ListingProvider, Home, DeepLinkHandler);

#[cfg(test)]
mod tests {
    use super::*;
    use aidoku_test::aidoku_test;
    use helper::set_base_url;

    const BASE_URL: &str = "https://anime-sama.tv";

    #[aidoku_test]
    fn test_get_all() {
        set_base_url(BASE_URL);
        let manga_moins = AnimeSama::new();

        let mangas = manga_moins
            .get_search_manga_list(None, 1, std::vec![])
            .unwrap();

        assert!(mangas.entries.len() > 0);
    }
}
