extern crate chrono;
extern crate form_urlencoded;
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate ureq;
extern crate url;

use std::collections::HashMap;
use std::time::Duration;
use std::thread::sleep;
use serde::{Serialize, Deserialize};
use url::Url;
use crate::url::ENDPOINT;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GachaLogEntry {
  pub uid: String,
  pub gacha_type: String,
  pub item_id: String,
  pub count: String,
  pub time: String,
  pub name: String,
  pub lang: String,
  pub item_type: String,
  pub rank_type: String,
  pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GachaLogData {
  pub page: String,
  pub size: String,
  pub total: String,
  pub list: Vec<GachaLogEntry>,
  pub region: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GachaLog {
  pub retcode: i32,
  pub message: String,
  pub data: Option<GachaLogData>
}

/*
 * Fetch Gacha log
 */

fn fetch_gacha_log(url: &Url, gacha_type: &str, end_id: &str) -> GachaLog {
  ureq::request_url("GET", url)
    .query("gacha_type", gacha_type)
    .query("page", "1")
    .query("size", "20")
    .query("end_id", end_id)
    .call()
    .expect("Request failed")
    .into_json()
    .unwrap()
}

pub fn fetch_gacha_logs(
  gacha_url: &str,
  gacha_type: &str,
  counter_fn: &Box<dyn Fn(u32) -> ()>
) -> Vec<GachaLogEntry> {
  let endpoint_start = gacha_url.find(ENDPOINT).expect("Invalid gacha url");
  let base_url = &gacha_url[0..endpoint_start + ENDPOINT.len()];
  let query_str = &gacha_url[endpoint_start + ENDPOINT.len()..];

  let mut queries: HashMap<String, String> = form_urlencoded::parse(query_str.as_bytes()).into_owned().collect();
  queries.remove("gacha_type");
  queries.remove("page");
  queries.remove("size");
  queries.remove("end_id");

  let url = Url::parse_with_params(base_url, queries).unwrap();
  let mut end_id = String::from("0");
  let mut gacha_logs: Vec<GachaLogEntry> = Vec::new();
  let mut count: u32 = 0;

  loop {
    if count % 5 == 0 {
      // Avoid visit too frequently
      sleep(Duration::from_secs(3));
    }

    // Consume count
    counter_fn(count + 1);

    // Fetch
    let gacha_log = fetch_gacha_log(&url, gacha_type, &end_id);
    count = count + 1;

    if let Some(data) = gacha_log.data {
      if !data.list.is_empty() {
        // Get the last end id
        end_id = data.list.last().unwrap().id.clone();
        gacha_logs.extend(data.list);

        // Same as above
        sleep(Duration::from_secs(1));

        // Continue to request the next end id
        continue;
      }
    }

    // Otherwise the data is none or the data list is empty
    break;
  }

  gacha_logs
}
