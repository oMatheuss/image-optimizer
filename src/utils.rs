use image::ImageFormat;
use rocket::http::Accept;

fn get_accepted_content_type(accept: &Accept) -> Option<ImageFormat> {
    if accept.iter().any(|m| m.is_any() || m.is_avif()) {
        Some(ImageFormat::Avif)
    } else if accept.iter().any(|m| m.is_webp()) {
        Some(ImageFormat::WebP)
    } else {
        None
    }
}

fn get_upstream_content_type(upstream: &reqwest::Response) -> Option<ImageFormat> {
    match upstream.headers().get("accept")?.to_str() {
        Ok(mime_type) => ImageFormat::from_mime_type(mime_type),
        Err(_) => None,
    }
}

pub fn get_content_type(accept: &Accept, upstream: &reqwest::Response) -> ImageFormat {
    match get_accepted_content_type(accept) {
        Some(fmt) => fmt,
        None => match get_upstream_content_type(&upstream) {
            Some(fmt) => fmt,
            None => image::ImageFormat::Jpeg,
        },
    }
}
