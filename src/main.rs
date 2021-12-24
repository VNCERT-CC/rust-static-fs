use actix_files;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    middleware, App, HttpServer,
};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::normalize::TrailingSlash::Trim,
            ))
            .service(
                actix_files::Files::new("/", "./public/")
                    .use_last_modified(true)
                    .redirect_to_slash_directory()
                    .use_etag(true)
                    .index_file("index.html")
                    .prefer_utf8(true)
                    .disable_content_disposition()
                    .default_handler(|req: ServiceRequest| {
                        let (http_req, _payload) = req.into_parts();
                        async {
                            let response = actix_files::NamedFile::open("./public/index.html")?
                                .use_last_modified(true)
                                .prefer_utf8(true)
                                .disable_content_disposition()
                                .use_etag(true)
                                .into_response(&http_req)?;
                            Ok(ServiceResponse::new(http_req, response))
                        }
                    }),
            )
    });

    let mut server_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    if server_addr.starts_with("unix:") {
        #[cfg(unix)]
        return server.bind_uds(server_addr[5..].to_string())?.run().await;
        #[cfg(not(unix))]
        {
            eprintln!("Unix socket file not supported on windows, fallback to tcp:8080");
            server_addr = ":8080".to_string();
        }
    }
    server.bind(server_addr)?.run().await
}
