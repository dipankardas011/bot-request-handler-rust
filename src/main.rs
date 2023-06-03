
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use hyper::StatusCode;
use std::convert::Infallible;

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

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&hyper::Method::GET, "/") => {
            Ok(Response::new(Body::from("Hello, World!")))
        },
        (&hyper::Method::GET, "/ping") => {
            Ok(Response::new(Body::from("Pong!")))
        },
        (&hyper::Method::GET, "/bot") => {
            Ok(Response::new(Body::from("bot response!")))
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
