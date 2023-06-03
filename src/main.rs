
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::StatusCode;
use std::convert::Infallible;
use std::println;

use anyhow::Result;
use reqwest;

async fn foo(bot_url: String) -> Result<String, reqwest::Error> {
    let response = reqwest::get(bot_url).await?;

    let mut resp = String::new();

    if response.status() == reqwest::StatusCode::OK {
        let text = response.text().await?;
        resp = text;
    } else {
        println!("Response was not 200 OK");
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

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => {
            Ok(Response::new(Body::from("Hello, World!")))
        },
        (&hyper::Method::GET, "/ping") => {
            Ok(Response::new(Body::from("Pong!")))
        },
        (&hyper::Method::GET, "/bot") => {
            let body_str = match extract_body(req).await {
                Ok(body) => body,
                Err(_) => {
                    let mut error_response = Response::default();
                    *error_response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(error_response);
                }
            };
            println!("Body -> {body_str}");

            let mut mod_req: String = String::new();
            mod_req = body_str.replace(" ", "%20");

            println!("Body with %20 -> {mod_req}");

            let bot_uri = format!("https://dipankardas011-gpt2-bot.hf.space/generate?text={mod_req}");
            
            if mod_req.len() as i32 > 0 {
                let mut response_bot: String = String::new();
                match foo(bot_uri).await {
                    Ok(bot_response) => {
                        response_bot = bot_response;
                    },
                    Err(e) => println!("Error occurred: {:?}", e),
                }
                println!("Bot req is present");
                return Ok(create_response(StatusCode::OK, response_bot));

            } else {
                println!("Bot req is absent");
                return Ok(create_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error".to_string()));
            }
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
    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(handle_request)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("Server running on http://{}", addr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
