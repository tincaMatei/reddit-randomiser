use std::fs;
use std::fs::{File};
use std::io::Write;
use tide::Response;

fn get_image_html(link: String) -> String {
    format!("<img src={} class=\"center-fit\">", link).to_string()
}

fn get_gfycat_embed(link: tide::http::Url) -> String {
    let mut constructed_link = String::from("http://gfycat.com/ifr") + link.path();
    
    format!("<iframe src=\'{}\' frameborder=\'0\' scrolling=\'no\' width=\'100%\'
        height=\'100%\' style=\'position:absolute;top:0;left:0\' allowfullscreen></iframe>", 
        constructed_link).to_string()
}

fn get_iframe(link: String) -> String {
    format!("<iframe class=\"center-fit\" src=\"{}\"></iframe>\n", link).to_string()
}

fn get_vreddit(mut permalink: String) -> String {
    permalink.push_str(".json");
    let body = reqwest::blocking::get(&permalink).unwrap().text().unwrap();
    println!("new permalink: {}", permalink);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let mut video_url = v[0]["data"]["children"][0]["data"]["media"]["reddit_video"]["fallback_url"].clone();
    if video_url == serde_json::Value::Null {
        video_url = v[0]["data"]["children"][0]["data"]["crosspost_parent_list"][0]["media"]["reddit_video"]["fallback_url"].clone();
    }
    println!("Video url: {}", video_url);

    format!("<video class=\"center-fit\"controls><source src={}></video>", video_url).to_string()
}

fn get_html_segment(link: String, permalink: String) -> String {
    let mut contents = String::new();
    let permalink = String::from("https://www.reddit.com") + &permalink;

    let mut contents = format!("<a href=\"{}\" target=\"_blank\">Reddit link</a>\n
<div class=\"imgbox\">\n", permalink);
    
    let mut parsed_url = tide::http::Url::parse(&link).unwrap();
    
    println!("{}", link);
   
    let embed = if parsed_url.domain().unwrap() == "gfycat.com" {
        get_gfycat_embed(parsed_url)
    } else if link.ends_with(".gifv") {
        get_iframe(link)
    } else if parsed_url.domain().unwrap() == "v.redd.it"{
        get_vreddit(permalink)
    } else {
        get_image_html(link)
    };

    contents.push_str(&embed);

    contents.push_str("</div>\n");

    contents
}

fn build_html(piece: String) -> String {
    let mut contents = String::new();

    contents.push_str("
<!DOCTYPE html>
<html>

<head>
    <style>
    * {
        margin: 0;
        padding: 0;
    }

    .imgbox {
        display: grid;
        height:100%;
    }

    .center-fit {
        max-width: 100%;
        max-height: 100vh;
        margin: auto;
    }
    </style>
</head>

<body>
    ");

    contents.push_str(&piece);

    contents.push_str("
</body>

</html>
    ");

    contents
}

fn get_random_post(subreddit: String) -> String {
    let mut url_req: String = String::from("https://www.reddit.com");
    url_req.push_str(&subreddit);
    url_req.push_str("random.json");
    
    let body = reqwest::blocking::get(&url_req).unwrap().text().unwrap();
    
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let image_url = v[0]["data"]["children"][0]["data"]["url"].clone();
    let permalink = v[0]["data"]["children"][0]["data"]["permalink"].clone();

    if let serde_json::Value::String(image_url) = image_url {
        let permalink = if let serde_json::Value::String(y) = permalink { y } else { String::new() };
        build_html(get_html_segment(image_url, permalink))
    } else {
        let html_contents = fs::read_to_string("bad.html").unwrap();
        html_contents
    }
}

async fn request_subreddit_post(req: tide::Request<()>) -> tide::Result<tide::Response> {
    Ok(Response::builder(200)
        .body(get_random_post(req.url().path().to_string()))
        .header("asdf", "fdsa")
        .content_type(tide::http::mime::HTML)
        .build())
}

async fn request_image(req: tide::Request<()>) -> tide::Result<tide::Response> {
    let contents = fs::read("image.img").unwrap();
    Ok(Response::builder(200)
        .body(contents)
        .header("asdf", "fdsa")
        .content_type(tide::http::mime::BYTE_STREAM)
        .build())
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error>{
    tide::log::start();
   
    let mut app = tide::new();
    app.at("*").get(|_| async { Ok("Do something lmao") } );
    app.at("/r/*/").get(request_subreddit_post);
    app.at("/debug").get(|_| async move {
        Ok(Response::builder(200)
            .body("<html>hi</html>")
            .header("asdf", "fdsa")
            .content_type(tide::http::mime::HTML)
            .build()) } );
    app.at("*.img").get(request_image);

    app.listen("0.0.0.0:8080").await;
    
    Ok(())
}

