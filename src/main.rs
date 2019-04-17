mod shodan;

extern crate mongodb;
#[macro_use]
extern crate log;
extern crate argparse;
extern crate simplelog;

use mongodb::coll::Collection;
use mongodb::db::ThreadedDatabase;
use mongodb::ordered::OrderedDocument;
use mongodb::Bson;
use mongodb::{Client, Error, ThreadedClient};

use simplelog::*;

use std::process;

use argparse::{ArgumentParser, Store};

#[test]
fn test_get_collection() {
    assert_eq!(
        get_collection(&"tmp-collection-shodan".to_string()).is_err(),
        false
    );
}

fn get_collection(collection_name: &String) -> Result<Collection, Error> {
    let client = Client::connect("localhost", 27017)?;
    Ok(client.db("test-db").collection(collection_name))
}

#[test]
fn test_put_doc() {
    use mongodb::{bson, doc};
    let coll = get_collection(&"tmp-collection-shodan".to_string()).expect("cannot get collection");
    assert_eq!(put_doc(&coll, doc! {"abc":"def"}).is_err(), false);
}

fn put_doc(coll: &Collection, doc: OrderedDocument) -> Result<(), ()> {
    coll.insert_one(doc.clone(), None).ok().unwrap_or_else(|| {
        error!("Cannot insert one. Is mongod running?");
        mongodb::coll::results::InsertOneResult::new(None, None)
    });
    Ok(())
}

fn put_results_in_db(coll: &Collection, doc: &String) -> Result<usize, serde_json::error::Error> {
    let json: serde_json::Value = serde_json::from_str(&doc)?;
    let matches = match &json["matches"] {
        serde_json::Value::Array(a) => Some(a.clone()),
        _ => {
            error!("No hosts found in Shodan. Is your API key active? Try changing your query.");
            process::exit(-1);
            // None
        },
    };

    let mut n : usize = 0;
    debug!("Inserting into db");
    for single_match in matches.expect("Cannot insert empty document.") {
        debug!("ins: {}", single_match["ip_str"]);
        //let bson = Bson::from_json(single_match);
        let bson: Bson = single_match.into();
        put_doc(&coll, mongodb::from_bson(bson).expect("Cannot put doc into database. Is mongod running?"))
            .unwrap_or_else(|_| error!("Failed to input doc. Is mongod running?"));
        n += 1;
    }
    info!("Inserted {} records into db", n);
    Ok(n)
}

fn search_pageless(api_key: &String, query: &String, coll: &Collection) {
    // perform shodan search, put results in db
    info!("Fetch data from Shodan");
    let doc = shodan::host_search(&api_key, &query).expect("Cannot get from API");
    //.replace("\\", "");
    put_results_in_db(&coll, &doc).unwrap_or_else(|_| {
        error!("Error parsing Shodan response");
        process::exit(-1);
    });
}

fn search_pageful(api_key: &String, query: &String, coll: &Collection, max_page: &String) {
    // try to convert max page number to usize
    let max_page_number = max_page.parse::<usize>().unwrap_or_else(|_| {
        error!("Cannot parse max_page. Using 0 instead.");
        return 0;
    });

    for page in 1..max_page_number+1 {
        info!("Fetch data from Shodan (page {})", page);
        let doc = shodan::host_search_paged(&api_key, &query, page).expect("Cannot get from API");
        //.replace("\\u", "");
        let inserted = put_results_in_db(&coll, &doc).unwrap_or_else(|_| {
            error!("Error parsing shodan response. Make sure you have paid API plan to use paging option.");
            process::exit(-1);
        });
        if inserted == 0 {
            info!("No more data found for this query");
            return;
        }
        info!("Page {} downloaded. Sleeping for 1 sec", page);
        std::thread::sleep(std::time::Duration::from_secs(1)); // sleep for API
    }
}

fn main() {
    // configure logging
    TermLogger::init(LevelFilter::Info, Config::default()).unwrap();
    info!("Start");

    // parse arguments
    let mut api_key = "".to_string();
    let mut query = "".to_string();
    let mut max_page = "".to_string();
    let mut collection = "tmp-collection-shodan".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut api_key)
            .add_option(&["-a", "--api-key"], Store, "Shodan API key")
            .required();
        ap.refer(&mut query)
            .add_option(
                &["-q", "--query"],
                Store,
                "Shodan query (reference: hhttps://help.shodan.io/the-basics/search-query-fundamentals)",
            )
            .required();
        ap.refer(&mut collection)
            .add_option(&["-c", "--collection"], Store, "MongoDB collection name (can be any string). Use separate collections for different sets of data. Uses 'tmp-collection-shodan' by default.");
        ap.refer(&mut max_page)
            .add_option(&["-p", "--max_page"], Store, "Choose maximum page number. This tool will collect results from page 1 to max_page. For paid plans only.");
        ap.parse_args_or_exit();
    }

    // display API stats
    info!("Get API stats");
    let doc_stats = shodan::api_info(&api_key).expect("Cannot get from API");
    let json_stats: serde_json::Value = serde_json::from_str(&doc_stats).expect("Cannot parse response");
    println!("{}", serde_json::to_string_pretty(&json_stats).expect("Cannot prettify response"));

    info!("Sleeping for 1 sec");
    std::thread::sleep(std::time::Duration::from_secs(1)); // sleep for API

    // try to open db collection
    let coll = get_collection(&collection).expect("Cannot get collection");

    if max_page == "" {
        search_pageless(&api_key, &query, &coll);
    } else {
        search_pageful(&api_key, &query, &coll, &max_page);
    }

    info!("Completed");
}
