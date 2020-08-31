use hyper::{service::{make_service_fn, service_fn}, Body, Client, Request,
            Response, Server};
use std::net::SocketAddr;
use eyre::Result;

use crate::settings::Settings;

async fn serve_req(req: Request<Body>, conf: Settings) -> Result<Response<Body>, hyper::Error> {
    println!("received request at {:?}", req.uri());
    println!("method {:?}", req.method());
    println!("headers {:?}", req.headers());
    // use the echo server for now
    let lcs_host = conf.proxy.remote_host;
    let url_str = match req.uri().query() {
        Some(qstring) => format!("{}{}?{}", lcs_host, req.uri().path(), qstring),
        None => format!("{}{}", lcs_host, req.uri().path()),
    };
    let mut client_req_builder = Request::builder()
        .method(req.method())
        .uri(url_str);
    for (k, v) in req.headers().iter() {
        // the hyper client we invoke later will set the host, so we 
        // don't want to set that here
        if k == "host" {
            continue;
        }
        client_req_builder = client_req_builder.header(k, v);
    }
    let client_req = client_req_builder.body(req.into_body()).expect("error building client request");
    let res = Client::new().request(client_req).await?;
    Ok(res)
}

pub async fn run_server(conf: &Settings) -> Result<()> {
    let addr: SocketAddr = conf.proxy.host.parse()?;
    println!("Listening on http://{}", addr);
    let make_svc = make_service_fn(move |_| {
        let conf = conf.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                let conf = conf.clone();
                async move { serve_req(_req, conf).await }
            }))
        }
    });
    let serve_future = Server::bind(&addr)
        .serve(make_svc);
    if let Err(e) = serve_future.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}