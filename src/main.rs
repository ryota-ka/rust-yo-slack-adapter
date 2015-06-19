use std::env;
use std::collections::HashMap;

extern crate hyper;

extern crate iron;
use iron::prelude::*;
use iron::status;


extern crate rustc_serialize;
use rustc_serialize::json;

extern crate url;
use url::percent_encoding;

#[derive(Debug)]
enum Accessory {
    Link(String),
    Location(f64, f64)
}

#[derive(Debug)]
struct YoQuery {
    username:  Option<String>,
    accessory: Option<Accessory>
}

impl YoQuery {
    fn build_json(&self) -> Option<String> {
        let google_maps_url = self.get_static_map_url();

        match self.username {
            None => None,
            Some(ref username) => {
                let text = match self.accessory {
                    Some(Accessory::Link(ref link)) => {
                        format!("Yo Link from {} : {}", username, link)
                    },
                    Some(Accessory::Location(lat, lng)) => {
                        format!("Yo Location from {} : {}, {}\n{}", username, lat, lng, google_maps_url.unwrap())
                    },
                    None => {
                        format!("Yo from {}", username)
                    }
                };
                let mut hashmap = HashMap::new();
                hashmap.insert("username", username.to_string());
                hashmap.insert("text", text);
                let json = json::encode(&hashmap);
                match json {
                    Ok(val) => { Some(val) },
                    Err(_) => { None }
                }
            }
        }
    }

    fn get_static_map_url(&self) -> Option<String> {
        match self.accessory {
            Some(Accessory::Location(lat, lng)) => {
                let coordinate_str = &format!("{},{}", lat, lng);

                let mut params: HashMap<&str, &str> = HashMap::new();
                params.insert("center", coordinate_str);
                params.insert("format", "png");
                params.insert("maptype", "roadmap");
                params.insert("markers", coordinate_str);
                params.insert("sensor", "false");
                params.insert("size", "640x640");
                params.insert("zoom", "14");

                let base_url = "https://maps.googleapis.com/maps/api/staticmap?".to_string();
                let url = params.iter().fold(base_url, |acc, (key, val)| {
                    format!("{}&{}={}", acc, key, val)
                });
                Some(url)
            },
            _ => { None }
        }
    }

    fn from_raw_query(ref raw: &String) -> YoQuery {
        let mut query = YoQuery { username: None, accessory: None };

        let params: Vec<&str> = raw.split("&").collect();
        for p in params {
            if let Some((key, value)) = split_string_into_pair(p, '=') {
                match key {
                    "username" => {
                        query.username = Some(value.to_string())
                    },
                    "link" => {
                        // let decoded_url = url::decode(value.to_string());
                        let decoded_url_bytes = percent_encoding::percent_decode(value.as_bytes());
                        match String::from_utf8(decoded_url_bytes) {
                            Ok(url) => {
                                query.accessory = Some(Accessory::Link(url))
                            },
                            Err(_) => { }
                        }
                    },
                    "location" => {
                        let coordinate = split_string_into_pair(value, ';');
                        if coordinate.is_some() {
                            let (lat_str, lng_str) = coordinate.unwrap();
                            let lat = lat_str.parse::<f64>();
                            let lng = lng_str.parse::<f64>();
                            match (lat, lng) {
                                (Ok(lat), Ok(lng)) => {
                                        query.accessory = Some(Accessory::Location(lat, lng))
                                },
                                (_, _) => {}
                            }
                        }
                    },
                    _ => { println!("key: {}, value: {}", key, value) }
                }
            }
        };
        query
    }
}

fn main() {
    let slack_api_endpoint = match env::var("SLACK_API_ENDPOINT") {
        Ok(url) => { url },
        Err(_)  => { panic!("Error: SLACK_API_ENDPOINT is not set") }
    };

    Iron::new(move |req: &mut Request| -> IronResult<Response> {
        match req.url.query {
            None => {
                Ok(Response::with((status::BadRequest, "Parameters are missing")))
            },
            Some(ref raw_query) => {
                let query = YoQuery::from_raw_query(raw_query);
                match query.build_json() {
                    None => {
                        let msg = "Username is required";
                        Ok(Response::with((status::BadRequest, msg)))
                    },
                    Some(json) => {
                        let mut client = hyper::Client::new();
                        let result = client.post(&slack_api_endpoint)
                            .body(&json)
                            .header(hyper::header::ContentType::json())
                            .send().unwrap();
                        println!("{:?}", result);

                        Ok(Response::with((status::Ok, "Yo")))
                    }
                }
            }
        }
    }).http("localhost:8080").unwrap();
}

fn split_string_into_pair<'a>(string: &'a str, letter: char) -> Option<(&'a str, &'a str)> {
    let vec: Vec<&str> = string.split(letter).collect();
    let slice = &vec;
    match (slice.get(0), slice.get(1)) {
        (Some(k), Some(v)) => Some((k, v)),
        _ => None
    }
}
