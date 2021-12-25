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
 *  - Send API request to Discord with alerts
 *  - Include insults
 * 
 * V5: Calculate price differentials over periods of time
 *  - Convert date objects to dates
 *  - Apply math at different intervals (15m, 30m, 1h, 4h, 8h, 1d, 1w, 2w, 3w)
 */

const DISCORD_WEBHOOK_URL: &str = "";

extern crate reqwest;
extern crate serde_json;
extern crate serde;
extern crate chrono;
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use chrono::prelude::*;
use std::collections::HashMap;

// Let's go!
fn main() {
    match fetch_api() {
        Ok(response) => {
            let parsed: CwResponse = serde_json::from_str(&response).unwrap();
            create_record(Record::new(String::from("btcusd"), parsed.result.price));
        },
        Err(e) => {
            println!("Error\n:{}", e.to_string())
        }
    }

    // Compare last two dates and report the difference
    // TODO: abstract to its own function
    let last_2_lines = get_last_n_records(2, true);
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
        match send_zapier_alert(&newest, &oldest, &message) {
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


////////////////////////////////////////////////////////////////////////////////


// Data structure for data store
#[derive(Debug)]
struct Record {
    id:        u32,
    timestamp: String,
    pair:      String,
    mark:      f64
}
impl Record {
    pub fn new(pair: String, mark: f64) -> Self {
        Self { id: 0, timestamp: Local::now().to_string(), pair: pair, mark: mark }
    }
    pub fn from_row(row: String) -> Self {
        let vals = row.split("|");
        let vec = vals.collect::<Vec<&str>>();
    
        Record { 
            id:        vec[0].parse::<u32>().unwrap(), 
            pair:      String::from(vec[1]), 
            mark:      vec[2].parse::<f64>().unwrap(),
            timestamp: String::from(vec[3]), 
        }
    }
    fn to_row(&self) -> String {
        format!("{}|{}|{}|{}\n", self.id, self.pair, self.mark, self.timestamp)
    }
}


/**
 * Creates a new record in the data store
 */
fn create_record(mut record: Record) {
    let path = Path::new("db");
    let path_display = path.display();

    // Get next auto-increment value
    let last_row = get_last_record();
    let next_id = last_row.id.clone() + 1;
    record.id = next_id;

    let mut file = OpenOptions::new().append(true).create(true).open(&path).expect(
        "cannot open file"
    );
    file.write_all(record.to_row().as_bytes()).expect("insert failed");
    println!("record inserted successfully: {}", path_display);
}

/**
 * Gets the last record from the data store
 * TODO:
 *   - Add filters for pair
 */
fn get_last_record() -> Record {
    let f = match File::open("db") {
        Err(e) => panic!("couldn't open file: {}", e),
        Ok(file) => file,
    };
    let reader = BufReader::new(f);
    let last_line = reader.lines().last().unwrap().ok().unwrap();
    Record::from_row(last_line)
}

fn get_last_n_records(n: usize, newest_first: bool) -> Vec<Record> {
    let f = match File::open("db") {
        Err(e) => panic!("couldn't open file: {}", e),
        Ok(file) => file,
    };
    let reader = BufReader::new(f);
    let lines = reader.lines();
    
    //println!("Total lines: {}\n", lines.count());
    let mut last_n_lines = Vec::<String>::new();    

    for line in lines {
        let line_str = line.unwrap();
        //println!("{}", line_str);

        // Save the last n rows as iteration happens
        last_n_lines.insert(0, line_str);
        if last_n_lines.len() > n {
            last_n_lines.pop();
        }
    }

    if !newest_first {
        last_n_lines.reverse(); // orders same as in db (top to bottom)
    }

    let mut records: Vec<Record> = last_n_lines.iter().map(|l| Record::from_row(l.to_string())).collect();
    return records
}


////////////////////////////////////////////////////////////////////////////////


// JSON data structure for CryptoWatch API
#[derive(Deserialize, Debug)] struct CwResponse { result: CwResult, allowance: CwAllowance }
#[derive(Deserialize, Debug)] struct CwResult { price: f64 }
#[derive(Deserialize, Debug)] struct CwAllowance { cost: f32, remaining: f32, upgrade: String }

/**
 * Makes API Request for Price
 * 
 * TODO:
 *   - Add error checking
 */
#[tokio::main]
async fn fetch_api() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get("https://api.cryptowat.ch/markets/kraken/btcusd/price").send().await?;
    let body = res.text().await?;

    Ok(body)
}


////////////////////////////////////////////////////////////////////////////////


// Zapier API call for Discord
#[tokio::main]
async fn send_zapier_alert(newest: &Record, oldest: &Record, percent_diff: &String) -> Result<String, reqwest::Error> {
    let mut map = HashMap::new();
    map.insert("content", &percent_diff);
    //map.insert("timestamp", &newest.timestamp);
    //map.insert("percent_change", &percent_diff);

    let client = reqwest::Client::new();
    let res = client.post(DISCORD_WEBHOOK_URL)
                        .json(&map)
                        .send()
                        .await?;
    let body = res.text().await?;

    Ok(body)
}


