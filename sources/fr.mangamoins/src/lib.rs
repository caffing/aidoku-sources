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

use crate::helper::get_all_releases;

pub mod helper;

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
            Some(q) => Request::get(format!("{url}&q={}", encode_uri(q.to_lowercase())))?.header(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:146.0) Gecko/20100101 Firefox/146.0",
            ),
            None => Request::get(url)?,
        }
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

// #[cfg(test)]
// mod tests {

//     use aidoku::alloc::string::ToString;
//     use aidoku_test::aidoku_test;

//     use super::*;

//     #[aidoku_test]
//     fn test_get_all() {
//         let a = MangaMoins::new();

//         let f = a
//             .get_search_manga_list(Some("One Piece".to_string()), 1, std::vec![])
//             .unwrap();

//         let chapers = f.clone().entries[0].clone().chapters.unwrap()[0].clone();
//         let f = a
//             .get_page_list(f.clone().entries[0].clone(), chapers)
//             .unwrap();

//         panic!("{:?}", f);
//     }
// }
