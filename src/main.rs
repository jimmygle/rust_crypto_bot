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
 *  - Compare old price to new price at different thresholds
 *  - Alert via stdout at certain thresholds
 * 
 * V4:
 *  - Send API request to Discord with alerts
 *  - Include insults
 */

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


// Let's go!
fn main() {
    match fetch_api() {
        Ok(response) => {
            let parsed: CwResponse = serde_json::from_str(&response).unwrap();
            create_record(Record::new(String::from("btcusd"), parsed.result.price));
            println!("Here we go!!\n: {}", parsed.allowance.remaining);
        },
        Err(e) => {
            println!("Error\n:{}", e.to_string())
        }
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
    let vals = last_line.split("|");
    let vec = vals.collect::<Vec<&str>>();
    
    Record { 
        id:        vec[0].parse::<u32>().unwrap(), 
        timestamp: String::from(vec[3]), 
        pair:      String::from(vec[1]), 
        mark:      vec[2].parse::<f64>().unwrap() 
    }
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

