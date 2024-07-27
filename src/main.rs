use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use html_escape::encode_safe;
use piped_rust_sdk::PipedClient;
use reqwest;
use serde::Deserialize;

mod util;

struct AppData {
    client: PipedClient,
    frontend_url: String,
}

#[derive(Deserialize)]
struct WatchQuery {
    v: String,
}

#[derive(Deserialize)]
struct PlaylistQuery {
    list: String,
}

#[get("/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok().body("Service up!")
}

fn build_response(
    url: String,
    title: String,
    description: String,
    thumbnail: String,
) -> HttpResponse {
    let body = format!(
        r#"<!DOCTYPE html>
<html>
<head>
  <meta http-equiv="refresh" content="0; url={}" />
  <title>{}</title>
  <meta name="description" content="{}">
  <meta property="og:title" content="{}">
  <meta property="og:description" content="{}">
  <meta property="og:site_name" content="Piped">
  <meta property="og:image" content="{}">
</head>
</html>"#,
        url, title, description, title, description, thumbnail
    );

    HttpResponse::Ok().content_type("text/html").body(body)
}

#[get("/watch")]
async fn video(app_data: web::Data<AppData>, query: web::Query<WatchQuery>) -> HttpResponse {
    let video_id = query.v.clone();
    let url = format!("{}/watch?v={}", app_data.frontend_url, video_id);

    match app_data.client.video_from_id(video_id.to_string()).await {
        Ok(video) => build_response(
            url,
            encode_safe(&video.title).to_string(),
            encode_safe(&video.description).to_string(),
            video.thumbnail_url,
        ),
        Err(_err) => build_response(url, String::new(), String::new(), String::new()),
    }
}

#[get("/playlist")]
async fn playlist(app_data: web::Data<AppData>, query: web::Query<PlaylistQuery>) -> HttpResponse {
    let playlist_id = query.list.clone();
    let url = format!("{}/playlist?list={}", app_data.frontend_url, playlist_id);
    match app_data
        .client
        .playlist_from_id(playlist_id.to_string())
        .await
    {
        Ok(playlist) => build_response(
            url,
            encode_safe(&playlist.name).to_string(),
            encode_safe(&playlist.uploader).to_string(),
            playlist.thumbnail_url,
        ),
        Err(_err) => build_response(url, String::new(), String::new(), String::new()),
    }
}

#[get("/channel/{id}")]
async fn channel(app_data: web::Data<AppData>, channel_id: web::Path<String>) -> HttpResponse {
    let url = format!("{}/channel/{}", app_data.frontend_url, channel_id);

    match app_data
        .client
        .channel_from_id(channel_id.to_string())
        .await
    {
        Ok(channel) => build_response(
            url,
            encode_safe(&channel.name).to_string(),
            encode_safe(&channel.description).to_string(),
            channel.avatar_url,
        ),
        Err(_err) => build_response(url, String::new(), String::new(), String::new()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let frontend_url =
            util::get_env_string("FRONTEND_URL", String::from("https://piped.video"));
        let backend_url = util::get_env_string(
            "BACKEND_URL",
            String::from("https://pipedapi.adminforge.de"),
        );

        let req = reqwest::Client::new();
        let client = PipedClient::new(&req, backend_url);
        let data = AppData {
            client,
            frontend_url,
        };
        App::new()
            .app_data(web::Data::new(data))
            .service(status)
            .service(video)
            .service(playlist)
            .service(channel)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
