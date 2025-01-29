#[macro_use]
extern crate rocket;

mod error;
mod processing;
mod utils;

use std::io::Cursor;

use error::AppError;
use processing::process_image;
use reqwest::Url;
use rocket::http::{Accept, ContentType, Header, Status};
use rocket::State;
use utils::get_content_type;

struct ProgramState {
    client: reqwest::Client,
}

struct Processed {
    bytes: Vec<u8>,
    filename: String,
    content_type: ContentType,
}

impl<'r> rocket::response::Responder<'r, 'static> for Processed {
    fn respond_to(self, _: &'r rocket::request::Request<'_>) -> rocket::response::Result<'static> {
        rocket::response::Response::build()
            .header(self.content_type)
            .header(Header::new("Vary", "Accept"))
            .header(Header::new(
                "Cache-Control",
                "public, max-age=604800, stale-while-revalidate=86400, stale-if-error=80",
            ))
            .header(Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", self.filename),
            ))
            .header(Header::new(
                "Content-Security-Policy",
                "script-src 'none'; frame-src 'none'; sandbox;",
            ))
            .sized_body(self.bytes.len(), Cursor::new(self.bytes))
            .status(Status::Ok)
            .ok()
    }
}

#[get("/?<url>&<w>&<q>")]
async fn optimize(
    url: &str,
    w: u32,
    q: u8,
    accept: &Accept,
    state: &State<ProgramState>,
) -> Result<Processed, AppError> {
    let url = Url::parse(url).map_err(|_| "\"url\" parameter is invalid")?;
    let host = url
        .host()
        .ok_or("\"url\" parameter is invalid due to an invalid host")?
        .to_string();

    if !host.ends_with("mangadex.org") {
        Err("\"url\" parameter must be a domain or subdomain of mangadex.org")?;
    }

    if q < 1 || q > 100 {
        Err("\"q\" parameter (quality) must be an integer between 1 and 100")?;
    }

    let upstream = state
        .client
        .get(url)
        .header("User-Agent", "Reqwest/0.12.12")
        .header("Accept", accept.to_string())
        .send()
        .await?
        .error_for_status()?;

    let target_type = get_content_type(accept, &upstream);
    let in_bytes = upstream.bytes().await?;
    let mut bytes: Vec<u8> = Vec::new();

    process_image(
        Cursor::new(in_bytes),
        &mut Cursor::new(&mut bytes),
        w,
        q,
        target_type,
    )?;

    let content_type = ContentType::parse_flexible(target_type.to_mime_type()).unwrap();
    let filename = format!("image.{}", target_type.extensions_str()[0]);

    Ok(Processed {
        bytes,
        content_type,
        filename,
    })
}

#[launch]
fn rocket() -> _ {
    let state = ProgramState {
        client: reqwest::Client::new(),
    };

    rocket::build()
        .manage(state)
        .mount("/img", routes![optimize])
}
