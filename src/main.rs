
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::StatusCode;
use std::convert::Infallible;
use std::println;

use anyhow::Result;
use reqwest;

use std::sync::Arc;
use tera::{Tera, Context};

const BOT_URL: &str= "https://dipankardas011-gpt2-bot.hf.space/generate";
const BOT_TEXT_FIELD: &str = "text";

async fn foo(bot_url: String) -> Result<String, reqwest::Error> {
    let response = reqwest::get(bot_url).await?;

    let mut resp = String::new();

    if response.status() == reqwest::StatusCode::OK {
        let text = response.text().await?;
        resp = text;
    } else {
        println!("BOT Response was not StatusCode::OK");
    }

    Ok(resp)
}

async fn extract_body(req: Request<Body>) -> Result<String, anyhow::Error> {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
    let body_str = String::from_utf8(body_bytes.to_vec())?;
    Ok(body_str)
}

fn create_response(status: StatusCode, message: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .body(Body::from(message))
        .unwrap()
}

fn parse_user_text(body_str: &str) -> String {
    let params: Vec<&str> = body_str.split('&').collect();
    for param in params {
        let key_value: Vec<&str> = param.split('=').collect();
        if key_value.len() == 2 && key_value[0] == "query" {
            return key_value[1].to_string();
        }
    }
    String::new()
}

async fn handle_request(req: Request<Body>, tera: Arc<Tera>) -> Result<Response<Body>, Infallible> {
    println!("{:?}", req.headers());
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => {
            // Render the index.html template
            let context = Context::new();
            let rendered = tera.render("index.html", &context).unwrap();


            Ok(Response::new(Body::from(rendered)))
        },
        (&hyper::Method::GET, "/ping") => {
            Ok(Response::new(Body::from("Pong!")))
        },
        // (&hyper::Method::GET, path) if path.starts_with("/static/") => {
        //     // Serve static files
        //     let file_path = format!(".{}", path);
        //     let file_contents = match tokio::fs::read(file_path).await {
        //         Ok(contents) => contents,
        //         Err(_) => {
        //             let mut not_found = Response::default();
        //             *not_found.status_mut() = StatusCode::NOT_FOUND;
        //             return Ok(not_found);
        //         }
        //     };

        //     Ok(Response::new(Body::from(file_contents)))
        // },
        (&hyper::Method::POST, "/bot") => {
            let body_str = match extract_body(req).await {
                Ok(body) => body,
                Err(_) => {
                    let mut error_response = Response::default();
                    *error_response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(error_response);
                }
            };

            let mut user_text = parse_user_text(&body_str);
            user_text = user_text.replace(" ", "%20");

            let bot_uri = format!("{}?{}={}", BOT_URL, BOT_TEXT_FIELD, user_text);

            let mut response_bot: String = String::new();
            match foo(bot_uri).await {
                Ok(bot_response) => {
                    response_bot = bot_response;
                },
                Err(e) => println!("Error occurred: {:?}", e),
            }

            // Render the index.html template with the answer
            let mut context = Context::new();
            context.insert("answer", &response_bot);
            let rendered = tera.render("index.html", &context).unwrap();

            return Ok(create_response(StatusCode::OK, rendered));
        },
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        },
    }
}

#[tokio::main]
async fn main() {
    let tera = Arc::new(Tera::new("templates/**/*.html").unwrap());

    let make_svc = make_service_fn(|_conn| {
        let tera = tera.clone();
        async { Ok::<_, Infallible>(service_fn(move |req| handle_request(req, tera.clone()))) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
