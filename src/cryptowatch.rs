extern crate reqwest;
extern crate serde_json;
extern crate serde;

use serde::Deserialize;

// JSON data structure for CryptoWatch API
#[derive(Deserialize, Debug)] 
pub struct CwResponse { 
    pub result: CwResult, 
    pub allowance: CwAllowance 
}

#[derive(Deserialize, Debug)] 
pub struct CwResult { 
    pub price: f64 
}

#[derive(Deserialize, Debug)] 
pub struct CwAllowance { 
    pub cost: f32, 
    pub remaining: f32, 
    pub upgrade: String 
}

/**
 * Makes API Request for Price
 * 
 * TODO:
 *   - Add error checking
 */
#[tokio::main]
pub async fn fetch_api(url: String) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;
    let body = res.text().await?;

    Ok(body)
}