use std::fs;
use tide::Response;

/// Returns the reddit embed
fn get_reddit_embed(permalink: &String, title: &String, created: i64) -> String {
    let permalink = String::from("https://www.reddit.com") + &permalink;

    let mut contents = format!("<a href=\"{}\" target=\"_blank\">Reddit link</a>\n
<div class=\"imgbox\">\n", permalink);
    
    let mut embed = String::new();
    embed.push_str(format!("<blockquote class = \"reddit-card\" data-card-created=\"{}\"><a href=\"{}?ref=share&ref_source=embed\">{}</a></blockquote>", created, permalink, title).as_str());
    embed.push_str(format!("<script async src = \"//embed.redditmedia.com/widgets/platform.js\" charset=\"UTF-8\"></script>").as_str());

    contents.push_str(&embed);

    contents.push_str("</div>\n");

    contents
}

/// Replaces the template variables with the given data
fn fill_template(template: String, title: &str, contents: &str) -> String {
    let template = template.replace("%TITLE", title);
    let template = template.replace("%CONTENT", contents);

    template
}

/// Builds the entire html page with the reddit post page
fn get_random_post(subreddit: String) -> String {
    // Prepare template
    let template = fs::read_to_string("reddit-page.html");
    
    let template = match template {
        Ok(x) => { x }
        Err(x) => { return format!("Failed to load html file: {}", x).to_string(); }
    };

    // Make the request
    let mut url_req: String = String::from("https://www.reddit.com");
    url_req.push_str(&subreddit);
    url_req.push_str("random.json");
   
    let body = reqwest::blocking::get(&url_req);
    let body = match body {
        Ok(x) => { x }
        Err(x) => { return fill_template(template, "Whoopsie!", format!("Failed to fetch post: {}", x).as_str()); }
    }.text();
   
    let body = match body {
        Ok(x) => { x }
        Err(x) => { return fill_template(template, "Whoopsie!", format!("Failed to fetch post: {}", x).as_str()); }
    };

    let json = serde_json::from_str(&body);

    let json: serde_json::Value = match json {
        Ok(x) => { x }
        Err(x) => { return fill_template(template, "Whoopsie!", format!("Failed to fetch post: {}", x).as_str()); }
    };

    let json = match json[0] {
        serde_json::Value::Object(_) => { json[0].clone() }
        serde_json::Value::Null => { json }
        _ => { return fill_template(template, "Whoopsie!", "Failed to fetch post: JSON [0] is not a dictionary"); }
    };

    let permalink = json["data"]["children"][0]["data"]["permalink"].clone();
    let created = json["data"]["children"][0]["data"]["created"].clone();
    let title = json["data"]["children"][0]["data"]["title"].clone();
    
    let permalink = if let serde_json::Value::String(y) = permalink { y } 
        else { return fill_template(template, "Whoopsie!", "Failed to fetch post: JSON permalink is not a string"); };
    let title = if let serde_json::Value::String(y) = title { y } 
        else { return fill_template(template, "Whoopsie!", "Failed to fetch post: JSON title is not a string"); };
    
    let created = if let serde_json::Value::Number(y) = created {
        let y = y.as_i64();
        match y {
            Some(x) => { x }
            None => {
                println!("Failed to load 'created'");
                0
            }
        }
    } else {
        println!("JSON created is not a number");
        0
    };

    let embed = get_reddit_embed(&permalink, &title, created);
    fill_template(template, &title, &embed)
    //"stub".to_string()
}

async fn request_subreddit_post(req: tide::Request<()>) -> tide::Result<tide::Response> {
    Ok(Response::builder(200)
        .body(get_random_post(req.url().path().to_string()))
        .header("asdf", "fdsa")
        .content_type(tide::http::mime::HTML)
        .build())
}

async fn request_image(req: tide::Request<()>) -> tide::Result<tide::Response> {
    let img = req.url().path_segments();
    
    let img = match img {
        Some(x) => {
            x.last().unwrap()
        }
        None => {
            return Ok(Response::builder(404)
                .build());
        }
    };

    println!("{}", img);

    let contents = fs::read(img);

    let contents = match contents{
        Ok(x) => { x }
        Err(x) => { 
            println!("Failed to load image: {}", x);
            return Ok(Response::builder(404).build());
        }
    };

    Ok(Response::builder(200)
        .body(contents)
        .content_type(tide::http::mime::BYTE_STREAM)
        .build())
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error>{
    tide::log::start();
   
    let mut app = tide::new();
    app.at("*").get(|_| async { Ok("You should do (ip):(port)/r/subreddit/ (see the last '/')") } );
    
    app.at("/r/*/").get(request_subreddit_post);

    app.at("/debug").get(|_| async move {
        Ok(Response::builder(200)
            .body("<html>hi</html>")
            .header("asdf", "fdsa")
            .content_type(tide::http::mime::HTML)
            .build()) } );

    app.at("*.png").get(request_image);
    app.at("*.ico").get(request_image);

    app.listen("0.0.0.0:6969").await;
    
    Ok(())
}

