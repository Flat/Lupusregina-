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

use lavalink_rs::{gateway::*, model::*, LavalinkClient};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::channel::Message;
use serenity::{
    async_trait,
    client::bridge::voice::ClientVoiceManager,
    prelude::{Context, Mutex, RwLock, TypeMapKey},
};
use std::sync::Arc;
use std::{collections::HashSet, time::Duration};

pub struct VoiceManager;

impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

pub struct Lavalink;

impl TypeMapKey for Lavalink {
    type Value = Arc<Mutex<LavalinkClient>>;
}

pub struct VoiceGuildUpdate;

impl TypeMapKey for VoiceGuildUpdate {
    type Value = Arc<RwLock<HashSet<GuildId>>>;
}

pub struct LavalinkHandler;

#[async_trait]
impl LavalinkEventHandler for LavalinkHandler {
    async fn track_start(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackStart) {
        println!("Track started!\nGuild: {}", event.guild_id);
    }
    async fn track_finish(&self, _client: Arc<Mutex<LavalinkClient>>, event: TrackFinish) {
        println!("Track finished!\nGuild: {}", event.guild_id);
    }
}

#[command]
#[only_in("guilds")]
async fn play(context: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg
        .guild(context)
        .await
        .ok_or("Failed to get guild from Message.")?;

    let guild_id = guild.id;

    let voice_channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let voice_channel = match voice_channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(context, "You need to be in a voice channel!")
                .await?;
            return Err("User not in voice channel.".into());
        }
    };

    let query = args.rest().to_string();

    let voice_manager_lock = context
        .data
        .read()
        .await
        .get::<VoiceManager>()
        .cloned()
        .ok_or("Unable to get VoiceManager from data.")?;

    {
        let mut voice_manager = voice_manager_lock.lock().await;

        if let Some(handler) = voice_manager.get(&guild_id) {
            if let Some(channel_id) = handler.channel_id {
                if channel_id != voice_channel {
                    msg.channel_id
                        .say(context, "The bot is already in use in a voice channel.")
                        .await?;
                    return Err("Bot already in voice channel.".into());
                }
            }
        }

        let _ = voice_manager
            .join(&guild_id, voice_channel)
            .ok_or("Unable to join channel.")?;
    }

    loop {
        let mut voice_manager = voice_manager_lock.lock().await;

        let handler = voice_manager
            .get_mut(&guild_id)
            .ok_or("Unable to get voice handler.")?;

        if handler.token.is_some() && handler.session_id.is_some() && handler.endpoint.is_some() {
            break;
        }

        tokio::time::delay_for(tokio::time::Duration::from_millis(500)).await;
    }

    let data = context.data.read().await;

    let lava_lock = data
        .get::<Lavalink>()
        .ok_or("Unable to get Lavalink client from data.")?;

    let mut lava_client = lava_lock.lock().await;

    let query_info = lava_client.auto_search_tracks(&query).await?;

    if query_info.tracks.is_empty() {
        msg.channel_id
            .say(
                context,
                "Could not find any results matching the search query.",
            )
            .await?;
        return Err(format!("No results found from query: {}", &query).into());
    }
    let handler = voice_manager_lock
        .lock()
        .await
        .get(&guild_id)
        .ok_or("Unable to get voice handler.")?
        .clone();
    lava_client.create_session(guild_id, &handler).await?;

    drop(lava_client);

    LavalinkClient::play(guild_id.0, query_info.tracks[0].clone())
        .queue(Arc::clone(lava_lock))
        .await?;
    let track_title = match query_info.tracks[0].info.as_ref() {
        Some(info) => info.title.clone(),
        None => "Unknown".into(),
    };
    msg.channel_id
        .say(context, format!("Added {} to the queue!", track_title))
        .await?;

    Ok(())
}

#[command]
#[only_in("guilds")]
async fn stop(context: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("GuildId not found from message.")?;

    let voice_manager_lock = context
        .data
        .read()
        .await
        .get::<VoiceManager>()
        .cloned()
        .ok_or("Unable to get VoiceManager from data.")?;

    let mut voice_manager = voice_manager_lock.lock().await;

    if voice_manager.get(&guild_id).is_some() {
        voice_manager.remove(&guild_id);
        {
            let data = context.data.read().await;

            let lava_client_lock = data
                .get::<Lavalink>()
                .ok_or("Unable to get Lavalink client from data.")?;

            lava_client_lock.lock().await.destroy(guild_id.0).await?;
        }
    } else {
        return Err("Bot not currently in voice channel.".into());
    }

    Ok(())
}

#[command]
#[only_in("guilds")]
#[aliases("np")]
async fn nowplaying(context: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.ok_or("GuildId not found from message.")?;

    let data = context.data.read().await;

    let lava_client_lock = data
        .get::<Lavalink>()
        .ok_or("Unable to get Lavalink client from data.")?;

    let lava_client = lava_client_lock.lock().await;

    if let Some(node) = lava_client.nodes.get(&guild_id.0) {
        let track = &node.now_playing;

        if let Some(x) = track {
            let track_info = x
                .track
                .info
                .as_ref()
                .ok_or("Could not obtain reference to track.info.")?;
            let track_len = Duration::from_millis(track_info.length);
            let track_pos = Duration::from_millis(track_info.position);

            msg.channel_id
                .send_message(context, |m| {
                    m.embed(|e| {
                        e.title(format!("{} by {}", track_info.title, track_info.author))
                            .url(track_info.uri.clone())
                            .field(
                                "Position",
                                format!(
                                    "{} / {}",
                                    _parse_duration(track_pos),
                                    _parse_duration(track_len)
                                ),
                                false,
                            )
                    })
                })
                .await?;
        } else {
            msg.channel_id
                .say(context, "Nothing is playing at the moment.")
                .await?;
        }
    } else {
        msg.channel_id
            .say(context, "Nothing is playing at the moment.")
            .await?;
    }

    Ok(())
}

fn _parse_duration(duration: Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = duration.as_secs() / 60;
    format!("{}:{:02}", minutes, seconds)
}
