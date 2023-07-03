use actix_multipart::Multipart;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryStreamExt};
use sanitize_filename::sanitize;
use std::env;
use tokio::fs::File;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::fs;
use std::path::Path;

/// Handles file uploads to the server. Uses the Multipart request payload to handle the uploaded file.
/// The function sanitizes the filename to ensure security, then writes the received data to a new file.
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

/// Handles download requests for files on the server.
/// The function checks if the requested file exists, and if it does, returns the file's content.
#[get("/{filename}")]
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

/// Handles delete requests for files on the server.
/// The function checks if the requested file exists, and if it does, deletes the file.
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

/// Main function to set up the server and start listening for requests.
/// The server listens on a user-specified port, or defaults to 3000 if no port is specified.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::args().nth(1).unwrap_or_else(|| "3000".to_string());
    let bind_address = format!("0.0.0.0:{}", port);
    println!("Listening on http://{}", bind_address);
    println!("Press ENTER to exit");

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
            .service(download)
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