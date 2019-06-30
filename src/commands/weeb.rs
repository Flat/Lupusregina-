/*
 * Copyright 2019 Kenneth Swenson
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
use serenity::framework::standard::{macros::command, Args, CommandError, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::Context;
use serenity::utils::Colour;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/anilist/schema.graphql",
    query_path = "src/anilist/AnimeQuery.graphql",
    response_derives = "Debug"
)]
struct AnimeQuery;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/anilist/schema.graphql",
    query_path = "src/anilist/MangaQuery.graphql",
    response_derives = "Debug"
)]
struct MangaQuery;

#[command]
#[description = "Shows information about an anime from Anilist."]
#[usage = "<Anime Title>"]
#[example = "Tate no Yuusha no Nariagari"]
#[min_args(1)]
#[bucket = "anilist"]
fn anime(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.rest();
    let anime = anime_query(anime_query::Variables {
        title: Some(query.to_string()),
    })?;
    let anime = anime
        .data
        .and_then(|data| {
            data.page
                .and_then(|page| page.media.and_then(|mut media| media.remove(0)))
        })
        .ok_or_else(|| CommandError("Unable to get anime from response.".to_string()))?;
    let id = anime.id;
    let title = anime
        .title
        .ok_or_else(|| CommandError("Unable to get title field from anime.".to_string()))?;
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
                sd.year.map_or_else(|| 0, |y| y),
                sd.month.map_or_else(|| 0, |m| m),
                sd.day.map_or_else(|| 0, |d| d)
            )
        },
    );
    let end_date = anime.end_date.map_or_else(
        || "0000/00/00".to_string(),
        |ed| {
            format!(
                "{:04}/{:02}/{:02}",
                ed.year.map_or_else(|| 0, |y| y),
                ed.month.map_or_else(|| 0, |m| m),
                ed.day.map_or_else(|| 0, |d| d)
            )
        },
    );

    msg.channel_id
        .send_message(&context, |m| {
            m.embed(|mut e| {
                e = e
                    .color(Colour::BLUE)
                    .url(format!("https://anilist.co/anime/{}", id));
                if title.romaji.is_some() && title.native.is_some() {
                    let title = format!("{} | {}", title.romaji.unwrap(), title.native.unwrap());
                    e = e.title(title);
                } else if title.romaji.is_some() && title.native.is_none() {
                    e = e.title(title.romaji.unwrap());
                } else if title.romaji.is_none() && title.native.is_some() {
                    e = e.title(title.native.unwrap())
                } else {
                    e = e.title("Title unavailable.");
                }
                if let Some(description) = description {
                    e = e.description(format_desc(description));
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
                            .filter_map(|g| g)
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
                e = e.timestamp(&Utc::now()).footer(|f| {
                    f.text("Data provided by Anilist.co")
                        .icon_url("https://anilist.co/img/icons/apple-touch-icon-152x152.png")
                });
                e
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

#[command]
#[description = "Shows information about a manga from Anilist."]
#[usage = "<Manga Title>"]
#[example = "Tate no Yuusha no Nariagari"]
#[bucket = "anilist"]
#[min_args(1)]
fn manga(context: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let query = args.rest();
    let manga = manga_query(manga_query::Variables {
        title: Some(query.to_string()),
    })?;
    let manga = manga
        .data
        .and_then(|data| {
            data.page
                .and_then(|page| page.media.and_then(|mut media| media.remove(0)))
        })
        .ok_or_else(|| CommandError("Unable to get manga from response.".to_string()))?;
    let id = manga.id;
    let title = manga
        .title
        .ok_or_else(|| CommandError("Unable to get title field from manga.".to_string()))?;
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
                sd.year.map_or_else(|| 0, |y| y),
                sd.month.map_or_else(|| 0, |m| m),
                sd.day.map_or_else(|| 0, |d| d)
            )
        },
    );
    let end_date = manga.end_date.map_or_else(
        || "0000/00/00".to_string(),
        |ed| {
            format!(
                "{:04}/{:02}/{:02}",
                ed.year.map_or_else(|| 0, |y| y),
                ed.month.map_or_else(|| 0, |m| m),
                ed.day.map_or_else(|| 0, |d| d)
            )
        },
    );

    msg.channel_id
        .send_message(&context, |m| {
            m.embed(|mut e| {
                e = e
                    .color(Colour::BLUE)
                    .url(format!("https://anilist.co/manga/{}", id));
                if title.romaji.is_some() && title.native.is_some() {
                    let title = format!("{} | {}", title.romaji.unwrap(), title.native.unwrap());
                    e = e.title(title);
                } else if title.romaji.is_some() && title.native.is_none() {
                    e = e.title(title.romaji.unwrap());
                } else if title.romaji.is_none() && title.native.is_some() {
                    e = e.title(title.native.unwrap())
                } else {
                    e = e.title("Title unavailable.");
                }
                if let Some(description) = description {
                    e = e.description(format_desc(description));
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
                            .filter_map(|g| g)
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
                e = e.timestamp(&Utc::now()).footer(|f| {
                    f.text("Data provided by Anilist.co")
                        .icon_url("https://anilist.co/img/icons/apple-touch-icon-152x152.png")
                });
                e
            })
        })
        .map_or_else(|e| Err(CommandError(e.to_string())), |_| Ok(()))
}

fn anime_query(
    variables: anime_query::Variables,
) -> Result<Response<anime_query::ResponseData>, failure::Error> {
    let request_body = AnimeQuery::build_query(variables);
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://graphql.anilist.co")
        .json(&request_body)
        .send()?;
    let response_body: Response<anime_query::ResponseData> = res.json()?;
    Ok(response_body)
}

fn manga_query(
    variables: manga_query::Variables,
) -> Result<Response<manga_query::ResponseData>, failure::Error> {
    let request_body = MangaQuery::build_query(variables);
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://graphql.anilist.co")
        .json(&request_body)
        .send()?;
    let response_body: Response<manga_query::ResponseData> = res.json()?;
    Ok(response_body)
}

fn format_desc(desc: String) -> String {
    desc.replace("<br>", "\n")
        .replace("<i>", "*")
        .replace("</i>", "*")
        .replace("<b>", "**")
        .replace("</b>", "**")
        .replace("&rsquo;", "'")
        .replace("&hellip;", "…")
}
