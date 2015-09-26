//! A simple web service for sharing crates.
//!
//! Rebar is a simple system for sorting content based on time and votes.
//! Rebar is inspired by the voting systems of Hacker News and Reddit.
//!
//! Rebar is used for sharing crates, but can be modified to other stuff
//! aswell.
//!
//! The query syntax is given by `[ip]:80/query?action=[action]{...}`. The
//! actions are described below. Input is given through GET requests.
//!
//! # `add`
//! Add new content.
//!
//! ## Input
//!
//! **Required**:
//!
//! - `host`: The place were the crate is hosted. Currently supported hosts are:
//!   - `github`
//!
//! **Optional**:
//!
//! - `desc`: A description of the crate (given by uploader).
//!
//! If GitHub is used following values are required:
//!
//! - `username`: The username of the uploader (the github username).
//! - `reponame`: The name of the repo.
//!
//! ## Output
//! Output is either:
//!
//! - `SUCC`: The query was sucessful.
//! - `ERROR: [error message]`: Query failed.
//!
//! # `list`
//! List the crates ordered.
//!
//! ## Input
//! No input is given.
//!
//! ## Output
//! An ordered list of ids seperated by `,`. They're ordered after the number
//! of votes and when they were uploaded. See `crate_data.rs` for details.
//! 
//! # `info`
//! Get info about a uploaded crate.
//!
//! ## Input
//! **Required**:
//!
//! - `id`: The id of the uploaded crate.
//!
//! ## Output
//! The output is of the form:
//! `github:[username]:[repo]:[description]
//!
//! # `vote`
//! Upvote a uploaded crate.
//!
//! ## Input
//! **Required**:
//!
//! - `id`: The id of the crate that the user wish to upload.
//!
//! ## Output
//! Output is either:
//!
//! - `SUCC`: The query was sucessful.
//! - `ERROR: [error message]`: Query failed.
//!
//! # `update`
//! Force the server to update
//!
//! ## Input
//! **Required**:
//!
//! - `pass`: The admin password.
//!
//! ## Output
//! Output is either:
//!
//! - `SUCC`: The query was sucessful.
//! - `ERROR: [error message]`: Query failed.

#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

#[macro_use] extern crate nickel;
extern crate time;
extern crate hyper;
extern crate serde;
extern crate serde_json;

use std::io::prelude::*;
use std::fs::File;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use nickel::{
    Nickel, QueryString, HttpRouter
};
use time::precise_time_s as now;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use hyper::header::AccessControlAllowOrigin as ACAO;
use std::env;

pub mod showcase;
pub mod crate_data;
use showcase::*;
use crate_data::*;

// TODO: Save the showcase.
fn main() {
    let backup_path = PathBuf::from(&env::var("REBAR_BACKUP").unwrap_or("rebar_data".to_string()));
    let showcase = Arc::new(Mutex::new(if env::args().nth(1).unwrap_or("".to_string()) == "-r".to_string() {
        println!("Reading backup file...");
        // Read backup file
        let mut string = String::new();
        File::open(&backup_path).expect("Failed to open backup file.")
                                .read_to_string(&mut string)
                                .expect("Failed to read backup file.");

        serde_json::from_str::<Showcase>(&string).expect("Couldn't parse backup file")

    } else {
        Showcase {
            crates: Vec::new(),
            shown_crates: Vec::new(),
            additions: 0,
            latest_upload: HashMap::new(),
            uploads: HashSet::new(),
        }
    }));
    let mut server = Nickel::new();
    let last_backup = AtomicUsize::new(0);
    let failed_backups = AtomicUsize::new(0);

    server.utilize(middleware! { |request|
        println!("request: {:?}", request.origin.uri);
    });

    let mut router = Nickel::router();

    router.get("/query", middleware! { |request, mut response|
        response.set(ACAO::Any);

        let ip   = Ip::new(request.origin.remote_addr);
        let data = request.query();
        if let Some(action) = data.get("action") {
            if last_backup.fetch_add(1, Ordering::Relaxed) > 30 {
                // Back up the data
                match File::create(&backup_path) {
                    Err(msg) => {
                         println!("Failed to create backup file: {}", msg);
                    },
                    Ok(mut file) => {
                        match serde_json::to_string(&*showcase.lock().unwrap()) {
                            Ok(ser) => {
                                match file.write_all(ser.as_bytes()) {
                                    Ok(_) => {
                                        last_backup.store(0, Ordering::Relaxed);
                                        println!("Backup successful");
                                    }
                                    Err(msg) => {
                                        println!("Failed to write to file: {}", msg);
                                    },
                                }
                            },
                            Err(msg) => {
                                println!("Failed to serialize showcase: {}", msg);
                                if failed_backups.fetch_add(1, Ordering::Relaxed) > 3 {
                                    last_backup.store(0, Ordering::Relaxed);
                                }
                            }
                        }
                   },
                };
            }


            match action {
                "add" => {
                    if let Some(host) = data.get("host") {
                        match host {
                            "github" => {
                                if let Some(username) = data.get("username") {
                                    if let Some(reponame) = data.get("reponame") {
                                        let username = username.trim();
                                        let reponame = reponame.trim();
                                        if username.contains(" ") || reponame.contains(" ") {
                                            format!("ERROR: Data may not contain whitespaces.")
                                        } else {
                                            let desc = data.get("desc").unwrap_or("");
                                            match showcase.lock().unwrap().add(Crate {
                                                description: desc.to_string(),
                                                repo: CrateStorage::Github(GithubRepo {
                                                    user: username.to_string(),
                                                    name: reponame.to_string(),
                                                }),
                                                uploaded: now(),
                                                uploader: ip,
                                            votes: 0,
                                                voters: HashSet::new(),
                                            }) {
                                                Ok(()) => format!("SUCC"),
                                                Err(UploadError::LimitReached) => format!("ERROR: Upload limit reached. Wait an hour."),
                                                Err(UploadError::Duplicate) => format!("ERROR: This content has already been uploaded."),
                                            }
                                        }
                                    } else {
                                        format!("ERROR: No repo name given.")
                                    }
                                } else {
                                    format!("ERROR: No Github username given.")
                                }
                            },
                            _ => {
                                format!("ERROR: Host not supported.")
                            }
                        }
                    } else {
                        format!("ERROR: No crate hoster provided.")
                    }
                },
                "list" => {
                    println!("Returning showcase...");
                    showcase.lock().unwrap().shown_crates.iter()
                                            .map(|s| s.to_string())
                                            .collect::<Vec<_>>().join(",")
                },
                "info" => {
                    if let Some(n) = data.get("id") {
                        let n = n.parse::<usize>().unwrap_or(0);
                        if let Some(requested_crate) = showcase.lock().unwrap().crates.get(n) {
                            requested_crate.to_string()
                        } else {
                            format!("ERROR: Non-existing crate requested.")
                        }


                    } else {
                        format!("ERROR: No id given.")
                    }
                },
                "vote" => {
                    if let Some(id) = data.get("id") {
                        if let Ok(parsed_id) = id.parse::<u64>() {
                            showcase.lock().unwrap().vote(parsed_id, ip);
                            format!("SUCC")
                        } else {
                            format!("ERROR: Invalid id.")
                        }
                    } else {
                        format!("ERROR: No id given.")
                    }
                },
                "update" => {
                    if let Some("pass") = data.get("pass") {
                        showcase.lock().unwrap().update();
                        format!("SUCC")
                    } else {
                        format!("ERROR: Wrong or no password.")
                    }
                },
                _ => {
                    format!("ERROR: Action not supported.")
                },
            }
            // TODO: my eyes are bleeding (bad formatting)

        } else {
            format!("ERROR: No action.")
        }
    });

    server.utilize(router);

    server.listen(env!("REBAR_LISTEN"));
}
