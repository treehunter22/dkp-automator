use sheets4::{hyper, hyper_rustls, oauth2, Sheets};

const SAMPLE_SPREADSHEET_ID: &str = "1Ye5d67h_Uljnt58Fa1gycCaEq9G90Qsk545OIlhlC7c";
const SAMPLE_RANGE_NAME: &str = "DKP Sheet!A3:A";

pub async fn get_names_from_sheets() -> Option<Vec<String>> {
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
        .values_get(SAMPLE_SPREADSHEET_ID, SAMPLE_RANGE_NAME)
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

    println!("{:?}", names);
    Some(names)
}
