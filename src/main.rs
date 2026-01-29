
use reqwest;
use scraper::{Html, Selector};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Debug)]
struct VTuber {
    name: String,
    status: String,
}
type SharedData = Arc<Mutex<Vec<VTuber>>>;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};


// Fonction récupère les VTubers d'une page donnée
async fn scrape_category(url: &str, status: &str) -> Vec<VTuber> {

    let body = reqwest::get(url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let document = Html::parse_document(&body);

    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();
    let a_selector = Selector::parse("a").unwrap();

    let mut vtubers = Vec::new();

    for row in document.select(&tr_selector) {

        let mut tds = row.select(&td_selector);
        let _icon = tds.next();
        let name_td = tds.next();

        if let Some(name_td) = name_td {

            let name = name_td
                .select(&a_selector)
                .next()
                .map(|a| a.text().collect::<String>())
                .unwrap_or("unknown".to_string());

            vtubers.push(VTuber {
                name,
                status: status.to_string(),
            });
        }
    }

    vtubers
}

struct Handler {
    data: SharedData,
}

#[async_trait]
impl EventHandler for Handler {

    async fn ready(&self, _: Context, ready: Ready) {
        println!("Bot connecté: {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {

        let content = msg.content.trim();

        if content == "!help" {
            let help = "
🤖 Ovulation Bot Commands

!vtubers        -> Stats
!status NAME   -> Statut d'un vtuber
!help          -> Aide
!list(all, ovulating, fertile or menstruating) -> for all menstruating vtubers 
";
            let _ = msg.channel_id.say(&ctx.http, help).await;
            return;
        }
if content == "!list all" {

   
let lines: Vec<String> = {
    let data = self.data.lock().unwrap();

    let mut lines = Vec::new();

    lines.push("-----OVULATING-----".to_string());
    for v in data.iter().filter(|v| v.status == "Ovulating") {
        lines.push(v.name.clone());
    }

    lines.push("\n------FERTILE-----".to_string());
    for v in data.iter().filter(|v| v.status == "Fertile") {
        lines.push(v.name.clone());
    }

    lines.push("\n-----MENSTRUATING------".to_string());
    for v in data.iter().filter(|v| v.status == "Menstruating") {
        lines.push(v.name.clone());
    }

    lines
};

    let mut buffer = String::new();

    for line in lines {

        if buffer.len() + line.len() + 1 > 1900 {
            let _ = msg.channel_id.say(&ctx.http, buffer.clone()).await;
            buffer.clear();
        }

        buffer.push_str(&line);
        buffer.push('\n');
    }

    if !buffer.is_empty() {
        let _ = msg.channel_id.say(&ctx.http, buffer).await;
    }

    return;
}
if content.starts_with("!list ") {

    let category = content.replace("!list ", "").to_lowercase();

    let status = match category.as_str() {
        "ovulating" => "Ovulating",
        "fertile" => "Fertile",
        "menstruating" => "Menstruating",
        _ => {
            let _ = msg.channel_id.say(
                &ctx.http,
                " Catégories: ovulating | fertile | menstruating"
            ).await;
            return;
        }
    };

    let list = {
        let data = self.data.lock().unwrap();

        data.iter()
            .filter(|v| v.status == status)
            
            .map(|v| v.name.clone())
            .collect::<Vec<_>>()
    };

    if list.is_empty() {
        let _ = msg.channel_id.say(&ctx.http, "Aucun résultat").await;
        return;
    }

    let response = format!(
        "**{} VTubers **\n{}",
        status,
        list.join("\n")
    );

    let _ = msg.channel_id.say(&ctx.http, response).await;
}
       if content == "!vtubers" {

            let (o, f, m) = {
                let data = self.data.lock().unwrap();

                let mut o = 0;
                let mut f = 0;
                let mut m = 0;

                for v in data.iter() {
                    match v.status.as_str() {
                        "Ovulating" => o += 1,
                        "Fertile" => f += 1,
                        "Menstruating" => m += 1,
                        _ => {}
                    }
                }

                (o, f, m)
            };

            let response = format!(
                "🩸 Ovule: {}\n🌸 Fertile: {}\n💀 Menstrue : {}",
                o, f, m
            );

            let _ = msg.channel_id.say(&ctx.http, response).await;
            return;
        }
        if content.starts_with("!status ") {

            let query = content.replace("!status ", "").to_lowercase();

            let result = {
                let data = self.data.lock().unwrap();

                data.iter()
                    .find(|v| v.name.to_lowercase() == query)
                    .map(|v| format!("{} : {}", v.name, v.status))
            };

            if let Some(r) = result {
                let _ = msg.channel_id.say(&ctx.http, r).await;
            } else {
                let _ = msg.channel_id.say(&ctx.http, " VTuber introuvable").await;
            }
        }
    }
}


#[tokio::main]
async fn main() {

    // Chargement initial
    let mut all = Vec::new();
    all.extend(scrape_category("https://ovu.moe/ovulating", "Ovulating").await);
    all.extend(scrape_category("https://ovu.moe/fertile", "Fertile").await);
    all.extend(scrape_category("https://ovu.moe/menstruating", "Menstruating").await);

    // Stockage partagé
    let data: SharedData = Arc::new(Mutex::new(all));

    // Discord
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing DISCORD_TOKEN");

    let handler = Handler {
        data: data.clone(),
    };

   let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .unwrap();

    println!("Bot lancé...");

    client.start().await.unwrap();
}

