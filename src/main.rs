use shaco::api::api_endpoints::{get_lol_match_history_v1_products_lol_current_summoner_matches, get_lol_match_history_v1_games_by_game_id};
use futures::future::join_all;

#[tokio::main]
async fn main() {
    let client = shaco::rest::RESTClient::new().unwrap();
    let req_client = reqwest::Client::new();

    println!("Checking 200 matches...");

    let match_history = get_lol_match_history_v1_products_lol_current_summoner_matches(&client, None, Some(200))
        .await
        .unwrap()
        .games
        .games;

    let match_history: Vec<_> = match_history
        .into_iter()
        .filter(|x| x.map_id == 11 && x.game_type == "CUSTOM_GAME" && x.game_mode == "CLASSIC")
        .map(|x| get_lol_match_history_v1_games_by_game_id(&client, x.game_id))
        .collect();

    println!("Downloaded {} potential custom games", match_history.len());

    let post_responses: Vec<_> = join_all(match_history)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .into_iter()
        .filter(|x| x.participants.len() == 10)
        .map(|x| req_client
            .post("https://api.påsan.com/match")
            .json(&x)
            .send()
        )
        .collect();

    println!("Attempting to send {} custom games", post_responses.len());

    let responses = join_all(post_responses)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    let num_inserted: i32 = responses
        .into_iter()
        .map(|x| x.status().is_success() as i32)
        .sum();

    println!("Successfully sent {num_inserted} custom games");

    let _ = std::process::Command::new("cmd.exe").arg("/c").arg("pause").status();
}
