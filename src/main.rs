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
 *  x Request price via API
 *  x Parse API response from API
 *  - Add error logic for bad HTTP request to API
 *  - Add error logic for bad API response (hit free request limit?)
 * 
 * V2: Save price data to datastore
 *  x Store price history to data store (flat file?)
 *  x Read price data from data store
 * 
 * V3: Calculate price differentials
 *  x Compare old price to new price at different thresholds
 *  x Alert via stdout at certain thresholds
 * 
 * V4:
 *  x Send API request to Discord with alerts
 *  - Include insults
 *
 * V4.5: 
 *  x Add config file
 *  x Split code into modules
 *  
 * V5: Calculate price differentials over periods of time
 *  - Convert date objects to dates
 *  - Apply math at different intervals (15m, 30m, 1h, 4h, 8h, 1d, 1w, 2w, 3w)
 * 
 * VNext: 
 *  - Add basic chart
 */

const CONFIG_FILE_PATH: &str = "./config.yml";

mod config;
mod datastore;
mod cryptowatch;
mod discord;

use crate::datastore::*;
use crate::cryptowatch::*;
use crate::discord::*;

// Let's go!
fn main() {
    let cfg = config::load(CONFIG_FILE_PATH);

    match fetch_api(cfg.cryptowatch_api_url) {
        Ok(response) => {
            let parsed: CwResponse = serde_json::from_str(&response).unwrap();
            create_record(cfg.db_price_history, Record::new(String::from("btcusd"), parsed.result.price));
        },
        Err(e) => {
            println!("Error\n:{}", e.to_string())
        }
    }

    // Compare last two dates and report the difference
    // TODO: abstract to its own function
    let last_2_lines = get_last_n_records(cfg.db_price_history, 2, true);
    for line in &last_2_lines {
        println!("{}  {}  {}  {}", line.id, line.timestamp, line.pair, line.mark)
    }
    let newest = &last_2_lines[0];
    let oldest = &last_2_lines[1];
    let diff = newest.mark - oldest.mark;
    let avg = (newest.mark + oldest.mark) / 2.00;
    let full_percent = diff / avg * 100.00;
    let percent = format!("{:.1$}", full_percent, 2);
    println!("Percent Change: {}%\n\n-----------\n", percent);

    // Alert if change is dramatic enough
    let threshold_percent = 1.00;
    if full_percent.abs() > threshold_percent {
        println!("ALERT: Change exceeding threshold since last check (+/- {}%) [old: {}, new: {}, change: {}%", threshold_percent, oldest.mark, newest.mark, percent);
        // Make post request to zapier
        let message  = format!("```\nPAIR:  {}\nPREV:  {}  ({})\nNEW :  {}  ({})\nCHNG:  {}%\n```", 
                                    &newest.pair, 
                                    &newest.mark, &newest.timestamp,
                                    &oldest.mark, &oldest.timestamp,
                                    &percent);
        match send_discord_alert(cfg.discord_webhook_url, &newest, &oldest, &message) {
            Ok(response) => {
                println!("Success:\n{}", response);
            },
            Err(e) => {
                println!("Error:\n{}", e.to_string())
            }
        }
    } else {
        println!("No alert triggered");
    }
}

