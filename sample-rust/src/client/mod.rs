use std::collections::HashMap;

use tokio_tools::run_in_tokio;

mod error;
mod tokio_tools;

pub fn run_client_test() -> std::result::Result<(), error::MyError> {
    run_in_tokio(async {
        println!("Hello from client test");

        let resp = reqwest::get("http://localhost:8079/hams/version")
            .await?
            .json::<HashMap<String, String>>()
            .await?;
        println!("{resp:#?}");

        let resp = reqwest::get("http://localhost:8079/hams/version")
            .await?
            .json::<HashMap<String, String>>()
            .await?;
        println!("{resp:#?}");

        let resp = reqwest::get("http://localhost:8079/hams/alive")
            .await?
            .text()
            // .json::<HashMap<String, String>>()
            .await?;
        println!("{resp:#?}");

        let resp = reqwest::get("http://localhost:8079/hams/alive_verbose")
            .await?
            .text()
            // .json::<HashMap<String, String>>()
            .await?;
        println!("{resp:#?}");

        let resp = reqwest::get("http://localhost:8079/hams/ready")
            .await?
            .text()
            // .json::<HashMap<String, String>>()
            .await?;
        println!("{resp:#?}");

        let resp = reqwest::get("http://localhost:8079/hams/ready_verbose")
            .await?
            .text()
            // .json::<HashMap<String, String>>()
            .await?;
        println!("{resp:#?}");

        Ok(())
    })
}
