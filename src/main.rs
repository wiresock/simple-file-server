use actix_multipart::Multipart;
use std::path::PathBuf;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryStreamExt};
use sanitize_filename::sanitize;
use std::env;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::fs;
use std::path::Path;
use tokio_util::io::ReaderStream;

/// Handles file uploads to the server.
///
/// This function uses the `Multipart` request payload to process the uploaded file. It goes through each part
/// of the payload until there are no more parts left.
///
/// If a part is a file (determined by the presence of a filename in the part's content-disposition),
/// the function sanitizes the filename to prevent directory traversal attacks and other security issues.
/// It then checks if a file with the same name already exists on the server.
///
/// If the file does not exist, the function creates a new file and writes the uploaded data to it.
/// If the file does exist, the function returns a `Conflict` response and does not overwrite the existing file.
///
/// # Arguments
///
/// * `payload` - A mutable reference to a `Multipart` payload, which represents the uploaded file data.
///
/// # Returns
///
/// An `HttpResponse` which can be:
/// * `Ok` with a success message as the body if the file was successfully uploaded.
/// * `BadRequest` if the filename is invalid or empty.
/// * `Conflict` if a file with the same name already exists on the server.
#[post("/upload")]
async fn upload(mut payload: Multipart) -> impl Responder {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        let filename = sanitize(content_disposition.get_filename().unwrap_or_default());
        if filename.is_empty() {
            return HttpResponse::BadRequest().body("Invalid filename");
        }
        let filepath = format!("./{}", filename);
        if Path::new(&filepath).exists() {
            return HttpResponse::Conflict().body("File already exists");
        }
        let mut f = File::create(&filepath).await.unwrap();
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f.write_all(&data).await.unwrap();
        }
    }
    HttpResponse::Ok().body("File uploaded successfully")
}

/// Handles download requests for files on the server by checking if the 
/// requested file exists, and if it does, returns the file's content in its entirety.
/// This may not be efficient for large files as it reads the entire file into memory.
///
/// # Arguments
///
/// * `filename` - A `web::Path<String>` representing the filename to download.
///
/// # Returns
///
/// An `HttpResponse` which can be `Ok` with the file's content as the body 
/// or `NotFound` if the file doesn't exist.
#[get("/download/{filename}")]
async fn download(filename: web::Path<String>) -> impl Responder {
    let filename = sanitize(filename.into_inner());
    let filepath = format!("./{}", filename);

    if Path::new(&filepath).exists() {
        let data = fs::read(filepath).unwrap();
        HttpResponse::Ok().body(data)
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

/// Handles download requests for files on the server by checking if the 
/// requested file exists, and if it does, returns the file's content in chunks.
/// This is efficient for large files as it streams the file in chunks rather than reading the 
/// entire file into memory.
///
/// # Arguments
///
/// * `path` - A `web::Path<String>` representing the path to the file to download.
///
/// # Returns
///
/// An `HttpResponse` which can be `Ok` with a `Stream` of the file's content as the body,
/// `InternalServerError` if there was a problem reading the file,
/// or `NotFound` if the file doesn't exist.
#[get("/download-chunked/{filename:.*}")]
async fn chunked_download(path: web::Path<String>) -> impl Responder {
    let filename = sanitize(path.into_inner());
    let file_path = PathBuf::from("./").join(filename);

    if file_path.exists() {
        match File::open(&file_path).await {
            Ok(file) => HttpResponse::Ok().streaming(ReaderStream::new(file)),
            Err(_) => HttpResponse::InternalServerError().body("Could not read file"),
        }
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

/// Handles delete requests for files on the server.
///
/// This function sanitizes the provided filename and checks if the file exists on the server.
/// If the file exists, it is deleted from the server. If the file does not exist, a response
/// indicating the file was not found is returned.
///
/// # Arguments
///
/// * `filename` - A `web::Path<String>` representing the filename to delete.
///
/// # Returns
///
/// An `HttpResponse` which can be:
/// * `Ok` with a success message as the body if the file was successfully deleted.
/// * `NotFound` if the file does not exist on the server.
#[delete("/{filename}")]
async fn delete(filename: web::Path<String>) -> impl Responder {
    let filename = sanitize(filename.into_inner());
    let filepath = format!("./{}", filename);

    if Path::new(&filepath).exists() {
        fs::remove_file(filepath).unwrap();
        HttpResponse::Ok().body("File deleted successfully")
    } else {
        HttpResponse::NotFound().body("File not found")
    }
}

/// Main function to set up two servers and start listening for requests on sequential ports.
/// The first server uses a flat download method, while the second uses a chunked download method.
///
/// Servers listen on a user-specified base port and the subsequent port number (base port + 1),
/// or default to 3000 and 3001 if no base port is specified.
///
/// Pressing ENTER will cause both servers to shut down.
///
/// # Returns
///
/// An `std::io::Result<()>` which is the result of the server operations. If successful,
/// the function will return `Ok(())`.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let base_port = env::args().nth(1).map(|s| s.parse::<u16>().unwrap()).unwrap_or(3000);

    let bind_address = format!("0.0.0.0:{}", base_port);
    println!("Listening on http://{}", bind_address);

    let (tx, rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let mut reader = BufReader::new(io::stdin());
        let mut buffer = String::new();
        reader.read_line(&mut buffer).await.expect("Failed to read line from stdin");
        tx.send(()).unwrap();
    });

    let server = HttpServer::new(|| {
        App::new()
            .service(upload)
            .service(download) // Flat download
            .service(chunked_download) // Chunked download
            .service(delete)
    })
    .bind(&bind_address)?
    .run();

    tokio::select! {
        _ = server => {},
        _ = rx => {
            println!("ENTER pressed, shutting down");
        }
    }

    Ok(())
}
