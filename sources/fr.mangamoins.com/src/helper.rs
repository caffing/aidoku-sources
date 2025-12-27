use aidoku::{
    Chapter, ContentRating, HashMap, Manga, MangaPageResult, MangaStatus, Result, UpdateStrategy,
    Viewer,
    alloc::{String, format, string::ToString, vec::Vec},
    imports::html::{Element, ElementList, Html, HtmlError},
};

use crate::BASE_URL;

fn extract_chapter(els: &Element) -> Chapter {
    let chapter_cover: Option<String> = els
        .select_first("a > figure > img")
        .and_then(|img| img.attr("src"))
        .map(|s| {
            s.strip_prefix("./")
                .map_or_else(|| s.clone(), |s| format!("{BASE_URL}/{s}"))
        });

    let chapter_url: Option<String> = els
        .select_first("a")
        .and_then(|img| img.attr("href"))
        .map(|img| format!("{BASE_URL}{img}"));

    let chapter_key: String = els
        .select_first("a")
        .and_then(|img| img.attr("href"))
        .map_or_else(
            || "unknown".to_string(),
            |href| href.split('=').nth(1).unwrap_or("unknown").to_string(),
        );

    let chapter_title = els
        .select_first("div.sortiefooter > p")
        .and_then(|t| t.own_text())
        .unwrap_or_default();

    let chapter_number: Option<f32> = els
        .select_first("div.sortiefooter > h3")
        .and_then(|h3| h3.text())
        .and_then(|text| {
            text.trim()
                .strip_prefix('#')
                .and_then(|num| num.trim().parse::<f32>().ok())
        });

    let chapter_language: Option<String> = els
        .select_first("div.sortiefooter > h4")
        .and_then(|h4| h4.text())
        .map(|text| text.trim().to_lowercase());

    Chapter {
        key: chapter_key,
        title: Some(chapter_title),
        url: chapter_url,
        chapter_number,
        volume_number: None,
        date_uploaded: None,
        scanlators: None,
        language: chapter_language,
        thumbnail: chapter_cover,
        locked: false,
    }
}

pub fn get_all_releases(response: &str, current_page: i32) -> Result<MangaPageResult> {
    let document = Html::parse(response)?;

    let mut manga_map: HashMap<String, Manga> = HashMap::new();

    let sorties = document
        .select("div.LastSorties div.sortie")
        .ok_or(HtmlError::NoResult)?;

    for els in sorties {
        let key = els
            .select_first("figcaption > p")
            .and_then(|t| t.own_text())
            .unwrap_or_default();

        let authors: Option<Vec<String>> = els
            .select_first("figcaption > p")
            .and_then(|t| t.select_first("span"))
            .and_then(|s| s.own_text())
            .map(|text| text.split(", ").map(|s| s.trim().to_string()).collect());

        let chapter = extract_chapter(&els);

        manga_map
            .entry(key.to_lowercase())
            .or_insert_with(|| Manga {
                key: key.clone(),
                title: key,
                cover: None,
                artists: None,
                authors,
                description: None,
                url: None,
                tags: None,
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::Unknown,
                update_strategy: UpdateStrategy::Always,
                next_update_time: None,
                chapters: Some(Vec::new()),
            })
            .chapters
            .as_mut()
            .ok_or(HtmlError::NoResult)?
            .push(chapter);
    }

    let last_page: Option<i32> = document
        .select("div.bottom_pages div.pages a")
        .iter()
        .next()
        .and_then(ElementList::text)
        .and_then(|text| {
            text.split_whitespace()
                .filter_map(|s| s.parse::<i32>().ok())
                .max()
        });

    let has_next_page = last_page.is_some_and(|page| page > current_page);

    let entries = manga_map.into_values().collect();
    Ok(MangaPageResult {
        entries,
        has_next_page,
    })
}
