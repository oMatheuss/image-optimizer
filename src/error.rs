use std::io::Cursor;

use rocket::http::Status;
use rocket::response::{Responder, Response};

#[derive(Responder)]
#[response(status = 400, content_type = "text")]
pub struct BadRequest(pub &'static str);

pub struct ReqwestError(reqwest::Error);

impl<'a, 'b: 'a> Responder<'a, 'b> for ReqwestError {
    fn respond_to(self, _: &'a rocket::Request<'_>) -> rocket::response::Result<'b> {
        let mut builder = Response::build();
        let err = self.0;

        if let Some(code) = err.status() {
            builder.status(Status::new(code.as_u16()));
        } else if err.is_timeout() {
            builder.status(Status::GatewayTimeout);
        } else {
            builder.status(Status::BadGateway);
        }

        let message = err.to_string();
        builder.sized_body(message.len(), Cursor::new(message)).ok()
    }
}

pub struct ImageError(image::ImageError);

impl<'a, 'b: 'a> Responder<'a, 'b> for ImageError {
    fn respond_to(self, _: &'a rocket::Request<'_>) -> rocket::response::Result<'b> {
        let status = match self.0 {
            image::ImageError::Decoding(..) => Status::BadGateway,
            image::ImageError::Encoding(..) => Status::NotAcceptable,
            image::ImageError::Parameter(..) => Status::InternalServerError,
            image::ImageError::Limits(..) => Status::ServiceUnavailable,
            image::ImageError::Unsupported(..) => Status::NotImplemented,
            image::ImageError::IoError(..) => Status::InternalServerError,
        };

        Response::build().status(status).ok()
    }
}

#[derive(Responder)]
pub enum AppError {
    BadRequest(BadRequest),
    ReqwestError(ReqwestError),
    ImageError(ImageError),
}

impl From<&'static str> for AppError {
    fn from(value: &'static str) -> Self {
        AppError::BadRequest(BadRequest(value))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        AppError::ReqwestError(ReqwestError(value))
    }
}

impl From<image::ImageError> for AppError {
    fn from(value: image::ImageError) -> Self {
        AppError::ImageError(ImageError(value))
    }
}
