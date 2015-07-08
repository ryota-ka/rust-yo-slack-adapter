extern crate iron;
extern crate hyper;
mod yo;

use std::env;
use iron::prelude::*;
use iron::status;
use yo::accessory::Accessory;
use yo::query::Query;

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
                let query = Query::from_raw_query(raw_query);
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

                        Ok(Response::with((status::Ok, "Yo")))
                    }
                }
            }
        }
    }).http("localhost:8080").unwrap();
}

