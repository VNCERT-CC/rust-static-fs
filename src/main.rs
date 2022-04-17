use actix_files;
use actix_web::{
    dev::{fn_service, ServiceRequest, ServiceResponse},
    middleware, App, HttpServer,
};
use clap;
use std::collections::HashSet;
use std::env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let matches = clap::Command::new(env!("CARGO_CRATE_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("Vinh Duong <vinhjaxt@xn--lun-lna.vn>")
        .about("Rust static file server")
        .arg(
            clap::Arg::new("bind-address")
                .short('b')
                .long("--bind-addr")
                .default_value("127.0.0.1:8080")
                .help("Bind address, unix:/tmp/http.sock to bind unix socket file"),
        )
        .arg(
            clap::Arg::new("public-folder")
                .short('f')
                .long("--folder")
                .default_value("./public/")
                .help("Public folder"),
        )
        .arg(
            clap::Arg::new("404-file")
                .short('4')
                .long("--404")
                .default_value("404.html")
                .help("File will be served when 404"),
        )
        .arg(
            clap::Arg::new("include-exts")
                .short('e')
                .long("--include-exts")
                .conflicts_with("exclude-exts")
                .multiple_occurrences(true)
                .takes_value(true)
                .help("Include file extensions (lowercase), eg: html,htm,css,txt,ico,jpeg,png"),
        )
        .arg(
            clap::Arg::new("exclude-exts")
                .short('x')
                .long("--exclude-exts")
                .conflicts_with("include-exts")
                .multiple_occurrences(true)
                .takes_value(true)
                .help("Exclude file extensions (lowercase), eg: php,asp,aspx,jsp,htaccess"),
        )
        .get_matches();

    let mut server_addr = matches.value_of("bind-address").unwrap();

    let mut public_folder = matches.value_of("public-folder").unwrap().to_string();
    if !public_folder.ends_with('/') {
        public_folder.push_str("/");
    }
    let notfound_file = matches.value_of("404-file").unwrap();

    let mut public_folder_404 = public_folder.to_owned();
    public_folder_404.push_str(&notfound_file);

    eprintln!(
        "Serve {}, 404: {} at {}",
        public_folder, public_folder_404, server_addr
    );

    let is_include_exts = matches.is_present("include-exts");
    let is_exclude_exts = matches.is_present("exclude-exts");

    let mut include_exts: HashSet<String> = HashSet::new();
    let mut exclude_exts: HashSet<String> = HashSet::new();

    if is_include_exts {
        for exts in matches.values_of("include-exts").unwrap() {
            for ext in exts.split(',') {
                let ext = ext.trim();
                if ext.len() == 0 {
                    continue;
                }
                println!("Include extension: {}", ext);
                include_exts.insert(ext.to_string());
            }
        }
    }
    if is_exclude_exts {
        for exts in matches.values_of("exclude-exts").unwrap() {
            for ext in exts.split(',') {
                let ext = ext.trim();
                if ext.len() == 0 {
                    continue;
                }
                println!("Exclude extension: {}", ext);
                exclude_exts.insert(ext.to_string());
            }
        }
    }

    let include_exts = Arc::new(include_exts);
    let exclude_exts = Arc::new(exclude_exts);
    let public_folder_404 = Arc::new(public_folder_404);

    let server = HttpServer::new(move || {
        // clone and move into threads
        let public_folder_404 = public_folder_404.clone();
        let include_exts = include_exts.clone();
        let exclude_exts = exclude_exts.clone();
        App::new()
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::MergeOnly,
            ))
            .service(
                actix_files::Files::new("/", &public_folder)
                    .path_filter(move |path, _| {
                        let ext = path.extension();
                        if ext.is_none() {
                            return true;
                        }
                        let ext = ext.unwrap().to_str().unwrap().to_lowercase();
                        if is_include_exts {
                            let include_exts = include_exts.clone();
                            return include_exts.contains(&ext);
                        }
                        if is_exclude_exts {
                            let exclude_exts = exclude_exts.clone();
                            return !exclude_exts.contains(&ext);
                        }
                        true
                    })
                    .use_last_modified(true)
                    .redirect_to_slash_directory()
                    .use_etag(true)
                    .index_file("index.html")
                    .prefer_utf8(true)
                    .disable_content_disposition()
                    .default_handler(fn_service(move |req: ServiceRequest| {
                        let public_folder_404 = public_folder_404.clone();
                        async move {
                            let (http_req, _payload) = req.into_parts();
                            let mut response =
                                actix_files::NamedFile::open_async(&*public_folder_404)
                                    .await?
                                    .use_last_modified(true)
                                    .prefer_utf8(true)
                                    .disable_content_disposition()
                                    .use_etag(true)
                                    .into_response(&http_req);
                            *response.status_mut() = actix_web::http::StatusCode::NOT_FOUND;
                            Ok(ServiceResponse::new(http_req, response))
                        }
                    })),
            )
    });

    if server_addr.starts_with("unix:") {
        #[cfg(unix)]
        return server.bind_uds(server_addr[5..].to_string())?.run().await;
        #[cfg(not(unix))]
        {
            eprintln!("Unix socket file not supported on windows, fallback to tcp:8080");
            server_addr = ":8080";
        }
    }
    server.bind(server_addr)?.run().await
}
