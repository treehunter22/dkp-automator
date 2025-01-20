use std::env;

use sheets4::{hyper, hyper_rustls, oauth2, Sheets};

pub async fn get_names_from_sheets() -> Option<Vec<String>> {
    let spreadsheet_id = env::var("spreadsheet_id").expect("Cannot read spreadsheet_id from .env");
    let range_name = env::var("range_name").expect("Cannot read range_name from .env");

    let secret: oauth2::ApplicationSecret = oauth2::read_application_secret("credentials.json")
        .await
        .expect("Cannot read credentials.json");

    let auth = oauth2::InstalledFlowAuthenticator::builder(
        secret,
        oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("token.json")
    .build()
    .await
    .unwrap();

    let hub = Sheets::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        auth,
    );

    let result = hub
        .spreadsheets()
        .values_get(&spreadsheet_id, &range_name)
        .doit()
        .await;

    let values = match result {
        Err(e) => {
            println!("{}", e);
            return None;
        }
        Ok(res) => res.1,
    };

    let names: Vec<String> = values
        .values
        .expect("No named found")
        .concat()
        .into_iter()
        .filter_map(|n| match n {
            serde_json::Value::String(name) => Some(name),
            _ => None,
        })
        .collect();

    Some(names)
}
