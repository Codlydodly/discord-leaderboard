use anyhow::Context as _;
use serenity::all::{Mention, UserId};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::{error, info};
use regex::Regex;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{Arc, RwLock};

struct Bot {
    leaderboard: Leaderboard,
}

struct Leaderboard {
    leaderboard: Arc<RwLock<HashMap<UserId, HashMap<i32, i32>>>>,
}

impl Leaderboard {

    fn new(leaderboard: Arc<RwLock<HashMap<UserId, HashMap<i32, i32>>>>) -> Self {
        Leaderboard { leaderboard }
    }
    
    async fn check_message(&self, user_id: UserId, wordle_id: i32, score: i32) -> String {
        let leaderboard = self.leaderboard.clone();
        if let Ok(mut leaderboard) = leaderboard.write() {
            match leaderboard.entry(user_id) {
                Entry::Occupied(mut user_entry) => {
                    if let Some(_current_wordle_id) = user_entry.get().get(&wordle_id) {
                        return format!("{} No Cheating! You've already done this wordle!", Mention::from(user_id))
                    } else {
                        user_entry.get_mut().insert(wordle_id, score);
                    }
                }
                Entry::Vacant(user_entry) => {
                    let mut temp_user_scores = HashMap::new();
                    temp_user_scores.insert(wordle_id, score);
                    user_entry.insert(temp_user_scores);
                }
            }
        };
        format!("{} Your score has been saved", Mention::from(user_id))        
    }

    async fn wordle_leaderboard(&self) -> String {
        if let Ok(leaderboard) = self.leaderboard.read() {
            let mut user_to_average_score: HashMap<UserId, f32> = HashMap::new();
            for (user_id, wordle_id_to_score) in leaderboard.iter() {
                let mut wordle_count = 0;
                let mut sum_score = 0;
                for (_wordle_id, score) in wordle_id_to_score.iter() {
                    wordle_count += 1;
                    sum_score += score;
                }
                let user_average_score = sum_score as f32 / wordle_count as f32;
                user_to_average_score.insert(*user_id, user_average_score);
            }
            let mut sorted_pairs: Vec<_> = user_to_average_score.iter().collect();
            sorted_pairs.sort_by(|(_, &a), (_, &b)| a.partial_cmp(&b).unwrap());
            let mut leaderboard_as_string: String = "".to_owned();
            for (i, user_id_to_score) in sorted_pairs.iter().enumerate() {
                leaderboard_as_string = format!("{}{}) {}, Score: {} \n", leaderboard_as_string, i+1, Mention::from(*user_id_to_score.0), user_id_to_score.1);
            }
            if leaderboard_as_string.is_empty(){
                return format!("Leaderboard is empty")
            }
            else {
                return leaderboard_as_string
            }            
        }
        return format!("Error, try again")
    }

}


#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let user_id = msg.author.id;
        if let Some((wordle_id, score)) = extract_wordle_score(&msg.content) {
            let reply_text = self.leaderboard.check_message(user_id, wordle_id, score).await;
            
            if let Err(e) = msg.channel_id.say(&ctx.http, &reply_text).await {
                error!("Error sending message: {:?}", e);
            }
        } else if msg.content == "!wordle" {
            let reply_text = self.leaderboard.wordle_leaderboard().await;

            if let Err(e) = msg.channel_id.say(&ctx.http, &reply_text).await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

fn extract_wordle_score(content: &str) -> Option<(i32, i32)> {
    if let Some(score_match) = Regex::new(r"\d+/\d+").unwrap().find(content) {
        let score = score_match.as_str().split('/').next()?.parse().ok()?;
        if let Some(id_match) = Regex::new(r"(\d+),?(\d*)").unwrap().find(content) {
            let wordle_id = id_match.as_str().replace(",", "").parse().ok()?;
            return Some((wordle_id, score));
        }
    }
    None
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let leaderboard_data = Arc::new(RwLock::new(HashMap::new()));
    let leaderboard = Leaderboard::new(leaderboard_data);
    let my_bot = Bot{leaderboard};
    let client = Client::builder(&token, intents)
        .event_handler(my_bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
