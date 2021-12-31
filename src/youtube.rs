use google_youtube3::YouTube;
use std::collections::VecDeque;
use tokio::time::{sleep_until, Duration, Instant};
use yup_oauth2;
pub async fn fetch_youtube_live_chat() {
    // Get an ApplicationSecret instance by some means. It contains the `client_id` and
    // `client_secret`, among other things.
    // let secret: yup_oauth2::ApplicationSecret = Default::default();
    let secret = yup_oauth2::read_application_secret("clientsecret.json")
        .await
        .expect("clientsecret.json");
    // Instantiate the authenticator. It will choose a suitable authentication flow for you,
    // unless you replace  `None` with the desired Flow.
    // Provide your own `AuthenticatorDelegate` to adjust the way it operates and get feedback about
    // what's going on. You probably want to bring in your own `TokenStorage` to persist tokens and
    // retrieve them from storage.
    let auth = yup_oauth2::InstalledFlowAuthenticator::builder(
        secret,
        yup_oauth2::InstalledFlowReturnMethod::HTTPRedirect,
    )
    .persist_tokens_to_disk("tokencache.json")
    .build()
    .await
    .unwrap();
    // let scopes = &["https://www.googleapis.com/auth/youtube.readonly"];
    //
    // match auth.token(scopes).await {
    //     Err(e) => println!("error: {:?}", e),
    //     Ok(t) => println!("token: {:?}", t),
    // }
    /*
    //sets up an auth listener

        url_template += "%1";
    url_template += "?response_type=code";
    url_template += "&client_id=%2";
    url_template += "&redirect_uri=%3";
    url_template += "&state=%4";
    url_template += "&scope=https://www.googleapis.com/auth/youtube";
    */
    let hub = YouTube::new(
        hyper::Client::builder().build(hyper_rustls::HttpsConnector::with_native_roots()),
        auth,
    );

    //TODO: to get liveChatid property,
    // you need to first get livebroadcast resource, then for that broadcast get the livechatId
    //you can search for broadcastStatus= Active and possibly combine it with mine=true
    //https://developers.google.com/youtube/v3/live/docs/liveBroadcasts/list?apix_params=%7B%22part%22%3A%5B%22snippet%2CcontentDetails%2Cstatus%22%5D%2C%22broadcastStatus%22%3A%22active%22%2C%22broadcastType%22%3A%22all%22%7D

    // You can configure optional parameters by calling the respective setters at will, and
    // execute the final call using `doit()`.
    // Values shown here are possibly random and not representative !

    // let result = hub
    //     .live_broadcasts()
    //     .list(&vec!["snippet".into(), "contentDetails".into(), "status".into()])
    //     .broadcast_type("all")
    //     .broadcast_status("active") //TODO:  check for most recent "upcoming" if there are no "active" ones or search for all and do something clever
    //     .doit()
    //     .await;

    let result = hub
        .videos()
        .list(&vec!["liveStreamingDetails".into()])
        .add_id("") //As a shortcut for testing, put the video id here. i.e. the ?v="blah" query param when visiting the video in the browser
        .doit()
        .await;

    let live_chat_id: Option<String> = match result {
        Ok((_, ref video_list_response)) => {
            let video = video_list_response
                .items
                .as_ref()
                .expect("Should have video in response");
            let first = video.get(0).expect("No video listed");
            let details = first
                .live_streaming_details
                .as_ref()
                .expect("Should have live streaming details");
            details.active_live_chat_id.clone()
        }
        Err(ref err) => {
            println!("{:?}", err);
            None
        }
    };

    let mut messages = VecDeque::new();
    let mut page_token = "".to_string(); //empty page token seems to get the first page
                                         //TODO: handle retries
                                         //TODO: maybe: pick the proper currency string via hl
                                         //TODO: query limit of 10k a day
                                         //https://developers.google.com/youtube/v3/live/docs/liveChatMessages#resource
                                         //notably, there is no prev_page_token on this response, meaning chat can only go forward
    while let Ok((_, ref live_chat_response)) = hub
        .live_chat_messages()
        .list(
            live_chat_id.as_ref().expect("No live chat id").as_str(),
            &vec!["id".into(), "snippet".into(), "authorDetails".into()],
        )
        .page_token(page_token.as_str())
        .max_results(2000) //I've never seen it actually pull this much, but maybe I'm just not watching big enough streamers
        .doit()
        .await
    {
        let now = Instant::now();

        for item in live_chat_response.items.as_ref().expect("No items?") {
            let snippet = item.snippet.as_ref().expect("No snippet?");
            let author_details = item
                .author_details
                .as_ref()
                .expect("No author details?")
                .display_name
                .as_ref()
                .expect("No display_name")
                .clone();
            let display_message = snippet
                .display_message
                .as_ref()
                .expect("No display message?")
                .clone();

            let message = (author_details, display_message);
            println!("{:?}", message);
            // if messages.len() >= 1000 { //TODO: create a wrapper type with a fixed capacity
            //    messages.pop_front();
            // }
            messages.push_back(message);
            //TODO: get moderator status from authorDetails, as well as chat sponsor
            // TODO: check channel thumbnail image
            // TODO: keep a cache of channel images?
            // TODO: keep a cache of emote images
        }

        page_token = live_chat_response
            .next_page_token
            .as_ref()
            .expect("No new page token")
            .clone();
        let polling_interval = live_chat_response
            .polling_interval_millis
            .expect("No polling interval");
        // necessary to obey this interval or you get rate limiting errors
        sleep_until(now + Duration::from_millis(polling_interval as u64)).await;
    }
}
