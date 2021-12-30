use google_youtube3::YouTube;
use tokio::time::{ Duration, sleep_until, Instant};
#[tokio::main]
async fn main() {
    use yup_oauth2;

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
    //     .broadcast_status("active") //TODO: maybe check for "upcoming" if there are no "active" ones or search for all and do something clever
    //     .doit()
    //     .await;


    //TODO: how to get a given video id
   let result = hub.videos()
       .list(&vec!["liveStreamingDetails".into()])
       .add_id("P7O5fnEKZfI")
       .doit()
       .await;


    let live_chat_id: Option<String> = match result {
        Ok((_ , ref video_list_response)) => {
            // println!("{:#?}", video_list_response);
           let video = video_list_response.items.as_ref().expect("Should have video in response");
            let first = video.get(0).expect("No video in response");
           let details = first.live_streaming_details.as_ref().expect("Should have live streaming details");
             details.active_live_chat_id.clone()
        }
        Err(ref err) => {
            println!("{:?}", err);
            None
        }
    };
    // let mut result = hub.live_chat_messages()
    //     .list(live_chat_id.as_ref().expect("No live chat id").as_str(), &vec!["id".into(), "snippet".into()])
    //     .doit()
    //     .await;
    // let page_token =  match result {
    //     Ok((_, ref live_chat_response)) => {
    //         println!("{:#?}", live_chat_response);
    //         live_chat_response.next_page_token.clone()
    //     }
    //     Err(ref error) => {
    //         println!("{}", error);
    //         None
    //     }
    // };
    // let mut page_token = page_token.expect("No page continuation");
    let mut messages: Vec<(String, String)> = vec![];
    let mut page_token = "".to_string();
    //TODO: handle retries
    //https://developers.google.com/youtube/v3/live/docs/liveChatMessages#resource
    while let Ok((_, ref live_chat_response)) = hub.live_chat_messages()
            .list(live_chat_id.as_ref().expect("No live chat id").as_str(), &vec!["id".into(), "snippet".into()])
            .page_token(page_token.as_str())
            .max_results(2000)
            .doit()
            .await {

        let now = Instant::now();
        // println!("{:#?}", live_chat_response);
        // println!("Added {}", live_chat_response.items.as_ref().expect("No items?").len());
        for item in live_chat_response.items.as_ref().expect("No items?") {
            let snippet = item.snippet.as_ref().expect("No snippet?");
            let author_channel_id = snippet.author_channel_id.as_ref().expect("No author_channel_id?").clone();
            let display_message = snippet.display_message.as_ref().expect("No display message?").clone();
            // println!("{:?}", message);

            let channels_builder =
                hub.channels()
                    .list(&vec!["snippet".into()])
                    .add_id(author_channel_id.as_str()); //can't seem to add more than one id, which sucks

            // for message in &messages {
            //     channels_builder = channels_builder.add_id(message.0.as_str());
            // }

            let result = channels_builder.doit().await;

            match result {
                Ok((_, channel_list_response)) => {
                    // println!("{:#?}", channel_list_response);
                    let title = channel_list_response.items.expect("No channel items")[0].snippet.as_ref().expect("No channel snippet").title.as_ref().expect("No channel title").clone();
                    //TODO: get author channel name from channel id, because author_details has nothing most of the time
                    //TODO: check channel thumbnail image
                    //TODO: keep a LRU cache of channel_id to channel title
                    //TODO: keep a cache of channel images
                    //TODO: keep a cache of emote images
                    let message = (title, display_message);
                    println!("{:?}", message);
                    messages.push(message)
                }
                Err(error) => {
                    println!("{}", error);
                }
            }

            // let message = (author_channel_id, display_message);
            // messages.push(message);
            // println!("{:#?}", messages);
        }


        //TODO: Look up moderators for special representation
        // println!("{:#?}", messages);

    page_token = live_chat_response.next_page_token.as_ref().expect("No new page token").clone();
        let polling_interval = live_chat_response.polling_interval_millis.expect("No polling interval");
        sleep_until(now + Duration::from_millis(polling_interval as u64)).await; //TODO: necessary to follow or you get errors for rate limiting
    // println!("looped");
    }


    // match result {
    //     Ok((_, live_chat_response)) => {
    //         println!("{:#?}", live_chat_response);
    //     }
    //     Err(error) => {
    //         println!("{}", error);
    //     }
    // }


    // println!("{:?}", result);


    // match result {
    //     Err(e) => match e {
    //         The Error enum provides details about what exactly happened.
    //         You can also just use its `Debug`, `Display` or `Error` traits
            // Error::HttpError(_)
            // | Error::Io(_)
            // | Error::MissingAPIKey
            // | Error::MissingToken(_)
            // | Error::Cancelled
            // | Error::UploadSizeLimitExceeded(_, _)
            // | Error::Failure(_)
            // | Error::BadRequest(_)
            // | Error::FieldClash(_)
            // | Error::JsonDecodeError(_, _) => println!("{}", e),
        // },
        // Ok(res) => println!("Success: {:?}", res),
    // }
}
