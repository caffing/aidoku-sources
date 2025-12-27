#![no_std]
use aidoku::{
    AidokuError, Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Home, HomeLayout, Listing,
    ListingProvider, Manga, MangaPageResult, Page, PageContent, Result, Source,
    alloc::{String, Vec, format},
    helpers::uri::encode_uri,
    imports::{
        html::{Html, HtmlError},
        net::Request,
    },
    register_source,
};
use helper::USER_AGENT;

use crate::parse::get_all_releases;

pub mod parse;

const BASE_URL: &str = "https://mangamoins.com";

struct MangaMoins;

impl Source for MangaMoins {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let url = format!("{BASE_URL}/?p={page}");
        let body = match &query {
            Some(q) => Request::get(format!("{url}&q={}", encode_uri(q.to_lowercase())))?,
            None => Request::get(url)?,
        }
        .header("Referer", BASE_URL)
        .header("User-Agent", USER_AGENT)
        .string()?;

        let results = get_all_releases(&body, page)?;

        Ok(results)
    }

    fn get_manga_update(
        &self,
        manga: Manga,
        _needs_details: bool,
        _needs_chapters: bool,
    ) -> Result<Manga> {
        let body = Request::get(format!(
            "{BASE_URL}/?q={}",
            encode_uri(manga.key.clone()).to_lowercase()
        ))?
        .header("Referer", BASE_URL)
        .header("User-Agent", USER_AGENT)
        .string()?;

        let results = get_all_releases(&body, 1)?;
        for result in &results.entries {
            if result.key == manga.key {
                return Ok(result.clone());
            }
        }
        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let body = match chapter.url {
            Some(url) => Request::get(url),
            None => Request::get(format!("{BASE_URL}/?scan={}", chapter.key)),
        }?
        .header("Referer", BASE_URL)
        .header("User-Agent", USER_AGENT)
        .string()?;

        let document = Html::parse(&body)?;

        let mut pages = Vec::new();

        let elements = document
            .select("link[rel='preload'][as='image']")
            .ok_or(HtmlError::NoResult)?;

        for els in elements {
            let href = els.attr("href").ok_or(HtmlError::NoResult)?;
            let url = href
                .strip_prefix("./")
                .map(|s| format!("{BASE_URL}/{s}"))
                .ok_or(HtmlError::NoResult)?;

            pages.push(Page {
                content: PageContent::url(url),
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
        let manga_moins = MangaMoins::new();

        let mangas = manga_moins
            .get_search_manga_list(None, 1, std::vec![])
            .unwrap();

        assert!(mangas.entries.len() > 0);
        assert!(mangas.entries[0].chapters.clone().unwrap().len() > 0);
        let chapers = mangas.entries[0].chapters.clone().unwrap();
        let pages: Vec<Page> = manga_moins
            .get_page_list(mangas.clone().entries[0].clone(), chapers[0].clone())
            .unwrap();

        assert!(pages.len() > 0)
    }
}
