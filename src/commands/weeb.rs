/*
 * Copyright 2020 Kenneth Swenson
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

use chrono::Utc;
use graphql_client::{GraphQLQuery, Response};
use html2text::from_read_with_decorator;
use serde::Deserialize;

use reqwest::Client as ReqwestClient;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::convert::TryFrom;
use poise::serenity_prelude::Colour;
use crate::{Context, Error};

use crate::util::DiscordMarkdownDecorator;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/anilist/schema.graphql",
    query_path = "src/anilist/AnimeQuery.graphql",
    response_derives = "Debug,Clone"
)]
struct AnimeQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/anilist/schema.graphql",
    query_path = "src/anilist/MangaQuery.graphql",
    response_derives = "Debug,Clone"
)]
struct MangaQuery;

const ANILIST_ICON: &str = "https://anilist.co/img/icons/apple-touch-icon-152x152.png";
const ANILIST_API_ENDPOINT: &str = "https://graphql.anilist.co";
const ANILIST_MANGA_PATH: &str = "https://anilist.co/manga/";
const ANILIST_ANIME_PATH: &str = "https://anilist.co/anime/";
const VIRTUALYOUTUBER_WIKI_API: &str = "https://virtualyoutuber.fandom.com/api.php";


#[poise::command(slash_command, description_localized("en", "Shows information about an Anime from Anilist"), global_cooldown = 1)]
pub async fn anime(context: Context<'_>, anime_title: String) -> Result<(), Error> {
    let anime = anime_query(anime_query::Variables {
        title: Some(anime_title),
    })
    .await?;
    let anime = anime
        .data
        .and_then(|data| {
            data.page.and_then(|page| {
                page.media
                    .and_then(|media| media.first().cloned().map_or_else(|| None, |m| m))
            })
        })
        .ok_or("Unable to get anime from response.")?;
    let id = anime.id;
    let title = anime.title.ok_or("Unable to get title field from anime.")?;
    let cover_image = anime.cover_image.and_then(|img| img.large);
    let description = anime.description;
    let status = anime.status;
    let episodes = anime.episodes;
    let genres = anime.genres;
    let average_score = anime.average_score;
    let season = anime.season;
    let start_date = anime.start_date.map_or_else(
        || "0000/00/00".to_string(),
        |sd| {
            format!(
                "{:04}/{:02}/{:02}",
                sd.year.unwrap_or(0),
                sd.month.unwrap_or(0),
                sd.day.unwrap_or(0)
            )
        },
    );
    let end_date = anime.end_date.map_or_else(
        || "0000/00/00".to_string(),
        |ed| {
            format!(
                "{:04}/{:02}/{:02}",
                ed.year.unwrap_or(0),
                ed.month.unwrap_or(0),
                ed.day.unwrap_or(0)
            )
        },
    );

    context
        .send(|m| {
            m.embed(|mut e| {
                e = e
                    .color(Colour::BLUE)
                    .url(format!("{}{}", ANILIST_ANIME_PATH, id));
                match (&title.romaji, &title.native) {
                    (Some(romaji), Some(native)) => e = e.title(format!("{} | {}", romaji, native)),
                    (Some(title), None) | (None, Some(title)) => e = e.title(title),
                    (None, None) => e = e.title("Title unavailable."),
                }
                if let Some(description) = description {
                    e = e.description(format_desc(description));
                } else {
                    e = e.description("No description available.");
                }
                if let Some(cover_image) = cover_image {
                    e = e.thumbnail(cover_image);
                }
                if let Some(status) = status {
                    e = e.field("Status", format!("{:?}", status), true);
                }
                if let Some(episodes) = episodes {
                    e = e.field("Episodes", episodes, true);
                }
                if let Some(genres) = genres {
                    e = e.field(
                        "Genres",
                        genres
                            .into_iter()
                            .flatten()
                            .collect::<Vec<String>>()
                            .join(", "),
                        true,
                    );
                }
                if let Some(score) = average_score {
                    e = e.field("Average Score", format!("{}%", score), true);
                }
                if let Some(season) = season {
                    e = e.field("Season", format!("{:?}", season), true);
                }
                if start_date != "0000/00/00" {
                    e = e.field("Start Date", start_date, true);
                }
                if end_date != "0000/00/00" {
                    e = e.field("End Date", end_date, true)
                }
                e = e
                    .timestamp(Utc::now())
                    .footer(|f| f.text("Data provided by Anilist.co").icon_url(ANILIST_ICON));
                e
            })
        })
        .await?;
    Ok(())
}


#[poise::command(slash_command, description_localized("en", "Shows information about a manga from Anilist"), global_cooldown = 1)]
pub async fn manga(context: Context<'_>, manga_title: String) -> Result<(), Error> {
    let manga = manga_query(manga_query::Variables {
        title: Some(manga_title),
    })
    .await?;
    let manga = manga
        .data
        .and_then(|data| {
            data.page.and_then(|page| {
                page.media
                    .and_then(|media| media.first().cloned().map_or_else(|| None, |m| m))
            })
        })
        .ok_or("Unable to get manga from response.")?;
    let id = manga.id;
    let title = manga.title.ok_or("Unable to get title field from manga.")?;
    let cover_image = manga.cover_image.and_then(|img| img.large);
    let description = manga.description;
    let status = manga.status;
    let chapters = manga.chapters;
    let genres = manga.genres;
    let average_score = manga.average_score;
    let start_date = manga.start_date.map_or_else(
        || "0000/00/00".to_string(),
        |sd| {
            format!(
                "{:04}/{:02}/{:02}",
                sd.year.unwrap_or(0),
                sd.month.unwrap_or(0),
                sd.day.unwrap_or(0)
            )
        },
    );
    let end_date = manga.end_date.map_or_else(
        || "0000/00/00".to_string(),
        |ed| {
            format!(
                "{:04}/{:02}/{:02}",
                ed.year.unwrap_or(0),
                ed.month.unwrap_or(0),
                ed.day.unwrap_or(0)
            )
        },
    );

    context
        .send(|m| {
            m.embed(|mut e| {
                e = e
                    .color(Colour::BLUE)
                    .url(format!("{}{}", ANILIST_MANGA_PATH, id));
                match (&title.romaji, &title.native) {
                    (Some(romaji), Some(native)) => e = e.title(format!("{} | {}", romaji, native)),
                    (Some(title), None) | (None, Some(title)) => e = e.title(title),
                    (None, None) => e = e.title("Title unavailable."),
                }
                if let Some(description) = description {
                    e = e.description(format_desc(description));
                } else {
                    e = e.description("No description available.");
                }
                if let Some(cover_image) = cover_image {
                    e = e.thumbnail(cover_image);
                }
                if let Some(status) = status {
                    e = e.field("Status", format!("{:?}", status), true);
                }
                if let Some(chapters) = chapters {
                    e = e.field("Chapters", chapters, true);
                }
                if let Some(genres) = genres {
                    e = e.field(
                        "Genres",
                        genres
                            .into_iter()
                            .flatten()
                            .collect::<Vec<String>>()
                            .join(", "),
                        true,
                    );
                }
                if let Some(score) = average_score {
                    e = e.field("Average Score", format!("{}%", score), true);
                }
                if start_date != "0000/00/00" {
                    e = e.field("Start Date", start_date, true);
                }
                if end_date != "0000/00/00" {
                    e = e.field("End Date", end_date, true)
                }
                e = e
                    .timestamp(Utc::now())
                    .footer(|f| f.text("Data provided by Anilist.co").icon_url(ANILIST_ICON));
                e
            })
        })
        .await?;
    Ok(())
}

async fn anime_query(
    variables: anime_query::Variables,
) -> Result<Response<anime_query::ResponseData>, Box<dyn std::error::Error + Send + Sync>> {
    let request_body = AnimeQuery::build_query(variables);
    let client = ReqwestClient::new();
    let res = client
        .post(ANILIST_API_ENDPOINT)
        .json(&request_body)
        .send()
        .await?;
    res.json().await.map_err(From::from)
}

async fn manga_query(
    variables: manga_query::Variables,
) -> Result<Response<manga_query::ResponseData>, Box<dyn std::error::Error + Send + Sync>> {
    let request_body = MangaQuery::build_query(variables);
    let client = ReqwestClient::new();
    let res = client
        .post(ANILIST_API_ENDPOINT)
        .json(&request_body)
        .send()
        .await?;
    res.json().await.map_err(From::from)
}

#[derive(Deserialize, Clone)]
struct WikiOpenSearchResults(String, Vec<String>, Vec<String>, Vec<String>);

#[derive(Deserialize, Clone)]
struct WikiQueryResults {
    batchcomplete: String,
    query: Query,
}

#[derive(Deserialize, Clone)]
struct Query {
    pages: Vec<Map<String, Value>>,
}

#[derive(Deserialize, Clone)]
struct ParseDetails {
    parse: Parse,
}

#[derive(Deserialize, Clone)]
struct Parse {
    title: String,
    pageid: u64,
    text: HashMap<String, String>,
}

#[derive(Deserialize, Clone)]
struct ArticleImage {
    image: HashMap<String, String>,
}

#[poise::command(slash_command, description_localized("en", "Shows information about a Virtual Youtuber"), global_cooldown = 1)]
pub async fn vtuber(context: Context<'_>, name: String) -> Result<(), Error> {
    let search = search_vtuber_wiki(name).await?;
    let title = search.1[0].clone();
    let url = search.3[0].clone();
    let text: String = get_vtuber_article_text(title.clone())
        .await?
        .parse
        .text
        .get("*")
        .ok_or("Failed to get text")?
        .clone();
    let start = text
        .find("</aside>")
        .ok_or("Unable to find start of description")?;
    let image = get_vtuber_article_image(title.clone()).await?;
    let parsed_text = from_read_with_decorator(
        text[start..].as_bytes(),
        256,
        DiscordMarkdownDecorator::new(),
    );
    let end = parsed_text
        .find("##")
        .ok_or("Unable to find end of description")?;
    let desc = &parsed_text[..end];
    context
        .send(|m| {
            m.embed(|e| {
                e.title(&title).url(url).description(desc);
                if let Some(thumbnail) = image.image.get("imageserving") {
                    e.thumbnail(thumbnail);
                }
                e
            })
        })
        .await?;
    Ok(())
}

async fn search_vtuber_wiki(
    search: String,
) -> Result<WikiOpenSearchResults, Box<dyn std::error::Error + Send + Sync>> {
    let client = ReqwestClient::new();
    client
        .get(VIRTUALYOUTUBER_WIKI_API)
        .query(&[
            ("action", "opensearch"),
            ("limit", "1"),
            ("search", &search),
            ("redirects", "resolve"),
            ("format", "json"),
        ])
        .send()
        .await?
        .json::<WikiOpenSearchResults>()
        .await
        .map_err(|_| Box::try_from(format!("No search results for {}", &search)).unwrap())
}

async fn get_vtuber_article_text(
    title: String,
) -> Result<ParseDetails, Box<dyn std::error::Error + Send + Sync>> {
    let client = ReqwestClient::new();
    client
        .get(VIRTUALYOUTUBER_WIKI_API)
        .query(&[
            ("action", "parse"),
            ("page", &title),
            ("format", "json"),
            ("prop", "text"),
        ])
        .send()
        .await?
        .json()
        .await
        .map_err(|_| Box::try_from(format!("Unable to get article {}", &title)).unwrap())
}

async fn get_vtuber_article_image(
    title: String,
) -> Result<ArticleImage, Box<dyn std::error::Error + Send + Sync>> {
    let client = ReqwestClient::new();
    client
        .get(VIRTUALYOUTUBER_WIKI_API)
        .query(&[
            ("action", "imageserving"),
            ("wisTitle", &title),
            ("format", "json"),
        ])
        .send()
        .await?
        .json()
        .await
        .map_err(|_| Box::try_from(format!("Unable to get image for article {}", &title)).unwrap())
}

fn format_desc(desc: String) -> String {
    desc.replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</br>", "\n")
        .replace("</ br>", "\n")
        .replace("<i>", "*")
        .replace("</i>", "*")
        .replace("<b>", "**")
        .replace("</b>", "**")
        .replace("&rsquo;", "'")
        .replace("&hellip;", "…")
}
