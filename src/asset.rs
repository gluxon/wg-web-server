// This makes rust_embed much nicer to work with when using it with rocket. It implements a from
// trait that transforms the asset struct into a vector of routes that can be mounted anywhere.
// This should be submitted as a pull request into rust_embed directly at some point in future.

use rocket::handler::Outcome;
use rocket::http::{ContentType, Method, Status};
use rocket::{Data, Request, Response, Route};
use rust_embed::RustEmbed;
use std::ffi::OsStr;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(RustEmbed)]
#[folder = "static"]
pub struct Asset;

fn handler<'r>(request: &'r Request, _data: Data) -> Outcome<'r> {
    // We can prepend a dot, or remove a leading forward slash to convert from URI form to the
    // relative path form rust_embed expects. Prepending a dot below since it's easier of the 2.
    let file_path = format!(".{}", request.uri().path());
    let bytes = Asset::get(&file_path);

    match bytes {
        Some(bytes) => {
            let mut response = Response::build();
            response.sized_body(Cursor::new(bytes));

            let content_type = PathBuf::from(&file_path)
                .extension()
                .and_then(OsStr::to_str)
                .and_then(ContentType::from_extension);

            if let Some(content_type) = content_type {
                // We should only set the Content-Type header if it can be guessed from the file
                // extension. If it can't be guessed, omitting this header it will allow browsers
                // to guess from the file's contents.
                //
                // See: https://tools.ietf.org/html/rfc7231#section-3.1.1.5
                response.header(content_type);
            }

            Outcome::from(request, response.finalize())
        }
        None => Outcome::failure(Status::NotFound),
    }
}

impl From<Asset> for Vec<Route> {
    fn from(_: Asset) -> Self {
        // TODO: Ignoring the object argument and using static methods doesn't seem like the
        // cleanest way to do this. This can probably be done better inside the rust_embed library
        // itself once this file becomes a pull request.
        Asset::iter()
            .map(|path| Route::new(Method::Get, format!("/{}", path), handler))
            .collect()
    }
}
