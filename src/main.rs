/**
 * Crypto Prices Alert Bot
 * 
 * Author: Jimmy Gleason <jimmygle@gmail.com>
 * 
 * The purpose of this app is to be a project to get my feet wet with Rust. It
 * could also be a fun price alerting bot to tie into Discord. So, this is going
 * to be a mess, and most likely won't get finished.
 * 
 * TODO:
 * 
 * V1: Get and parse price data
 * - Request price via API
 * - Parse API response from API
 * - Add error logic for bad HTTP request to API
 * - Add error logic for bad API response (hit free request limit?)
 * 
 * V2: Save price data to datastore
 * - Store price history to data store (flat file?)
 * - Read price data from data store
 * 
 * V3: Calculate price differentials
 * - Compare old price to new price at different thresholds
 * - Alert via stdout at certain thresholds
 * 
 * V4:
 * - Send API request to Discord with alerts
 * - Include insults
 */

extern crate reqwest;
extern crate serde_json;
extern crate serde;
use serde::Deserialize;

// JSON data structure from CryptoWatch API

#[derive(Deserialize, Debug)] struct CwResponse { result: CwResult }
#[derive(Deserialize, Debug)] struct CwResult { price: f64 }

// Let's go!
fn main() {
    match fetch_api() {
        Ok(response) => {
            let parsed: CwResponse = serde_json::from_str(&response).unwrap();
            println!("Here we go!!\n: {}", parsed.result.price);
        },
        Err(e) => {
            println!("Error\n:{}", e.to_string())
        }
    }
}

/**
 * Makes API Request for Price
 * 
 * TODO:
 * - Add error checking
 */
#[tokio::main]
async fn fetch_api() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get("https://api.cryptowat.ch/markets/kraken/btceur/price").send().await?;
    let body = res.text().await?;

    Ok(body)
}

