use actix_files;
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    middleware, App, HttpServer,
};
use std::env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut server_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());
    let mut public_folder = env::args()
        .nth(2)
        .unwrap_or_else(|| "./public/".to_string());
    if !public_folder.ends_with('/') {
        public_folder.push_str("/");
    }
    eprintln!("Serve {} at {}", public_folder, server_addr);

    let mut public_folder_index = public_folder.to_owned();
    public_folder_index.push_str("/index.html");

    let server = HttpServer::new(move || {
        // clone and move into threads
        let public_folder_index_thread = Arc::new(public_folder_index.clone());
        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::normalize::TrailingSlash::MergeOnly,
            ))
            .service(
                actix_files::Files::new("/", &public_folder)
                    .use_last_modified(true)
                    .redirect_to_slash_directory()
                    .use_etag(true)
                    .index_file("index.html")
                    .prefer_utf8(true)
                    .disable_content_disposition()
                    .default_handler(move |req: ServiceRequest| {
                        let public_folder_index_scope = public_folder_index_thread.clone();

                        let (http_req, _payload) = req.into_parts();
                        async move {
                            let response = actix_files::NamedFile::open(
                                &*public_folder_index_scope,
                            )?
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
