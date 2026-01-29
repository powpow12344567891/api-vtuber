
use reqwest;
use scraper::{Html, Selector};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct VTuber {
    name: String,
    status: String,
}

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

#[tokio::main]
async fn main() {
    //Vecteur qui contiens les VTubers.
    let mut all = Vec::new();

    all.extend(scrape_category("https://ovu.moe/ovulating", "Ovulating").await);
    all.extend(scrape_category("https://ovu.moe/fertile", "Fertile").await);
    all.extend(scrape_category("https://ovu.moe/menstruating", "Menstruating").await);

   println!("{}", serde_json::to_string_pretty(&all).unwrap());

}

