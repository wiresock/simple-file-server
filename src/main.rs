use actix_multipart::Multipart;
use std::ops::Deref;
use std::path::PathBuf;
use actix_web::{delete, get, post, web, App, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryStreamExt};
use sanitize_filename::sanitize;
use tokio::{fs::File, io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader}};
use std::fs;
use std::path::Path;
use tokio_util::io::ReaderStream;
use clap::{Command, arg};
use rustls::{ServerConfig, Certificate};
use rustls_pemfile::{certs, rsa_private_keys, pkcs8_private_keys, ec_private_keys};

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

/// Entry point for the File Server application.
///
/// This function sets up the command line arguments, reads the arguments provided by the user,
/// sets up the HTTP server, and runs the server until it is shut down.
///
/// Command line arguments:
/// * `--port [PORT]`: The port to listen on. Defaults to 3000.
/// * `--tls-cert [CERT]`: The path to the TLS certificate file. Optional.
/// * `--tls-key [KEY]`: The path to the TLS key file. Optional.
///
/// If both `--tls-cert` and `--tls-key` are provided, the server will use HTTPS. Otherwise, it will use HTTP.
///
/// The server provides the following services:
/// * `upload`: Upload a file to the server.
/// * `download`: Download a file from the server.
/// * `chunked_download`: Download a file from the server in chunks.
/// * `delete`: Delete a file from the server.
///
/// The server can be shut down by pressing ENTER.
///
/// # Errors
///
/// Returns an `std::io::Error` if an error occurs while setting up the server or running the server.
/// This includes errors like failing to bind to the specified port, failing to read the TLS certificate or key,
/// or failing to set up the server configuration.
///
/// # Examples
///
/// Run the server on port 3000 without TLS:
///
/// ```
/// cargo run -- --port 3000
/// ```
///
/// Run the server on port 3000 with TLS:
///
/// ```
/// cargo run -- --port 3000 --tls-cert cert.pem --tls-key key.pem
/// ```
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Define command line arguments
    let matches = Command::new("File Server")
    .version("1.0")
    .author("Vadim Smirnov")
    .about("Serves files over HTTP/HTTPS")
    .arg(arg!(--port [PORT] "Port to listen on").default_value("3000"))
    .arg(arg!(--"tls-cert" [CERT] "Path to the TLS certificate file"))
    .arg(arg!(--"tls-key" [KEY] "Path to the TLS key file"))
    .get_matches();

    // Get the port from the command line arguments
    let port = matches.get_one::<String>("port").unwrap().as_str();
    let bind_address = format!("0.0.0.0:{}", port);

    // Create a one-shot channel for shutting down the server
    let (tx, rx) = tokio::sync::oneshot::channel();

    // Spawn a new task that waits for a line from stdin and then sends a signal to the channel
    tokio::spawn(async move {
        let mut reader = BufReader::new(io::stdin());
        let mut buffer = String::new();
        reader.read_line(&mut buffer).await.expect("Failed to read line from stdin");
        tx.send(()).unwrap();
    });

    // Create a new HTTP server
    let server = HttpServer::new(|| {
        App::new()
            .service(upload)
            .service(download)
            .service(chunked_download)
            .service(delete)
    });

    // If the TLS certificate and key are provided, configure the server to use HTTPS
    let server = if let (Some(cert_path), Some(key_path)) = (matches.get_one::<String>("tls-cert"), matches.get_one::<String>("tls-key")) {
        // Read the certificate chain from the certificate file
        let cert_file = std::fs::File::open(cert_path)?;
        let mut cert_reader = std::io::BufReader::new(cert_file);
        let cert_chain = certs(&mut cert_reader)
            .filter_map(Result::ok)
            .map(|der| Certificate(der.deref().to_vec())) // Convert &[u8] to Vec<u8>
            .collect::<Vec<Certificate>>();

        // Read the private keys from the key file
        let key_file = std::fs::File::open(key_path)?;
        let mut key_reader = std::io::BufReader::new(key_file);
        
        // Try to read the private keys in RSA format
        let mut keys: Vec<_> = rsa_private_keys(&mut key_reader)
            .filter_map(Result::ok)
            .map(|key| rustls::PrivateKey(key.secret_pkcs1_der().to_vec())) // Convert &[u8] to Vec<u8>
            .collect();

        // If no RSA keys were found, try to read the private keys in PKCS8 format
        if keys.is_empty() {
            let mut key_reader = std::io::BufReader::new(std::fs::File::open(key_path)?);
            keys = pkcs8_private_keys(&mut key_reader)
                .filter_map(Result::ok)
                .map(|key| rustls::PrivateKey(key.secret_pkcs8_der().to_vec())) // Convert &[u8] to Vec<u8>
                .collect();
        }

        // If no PKCS8 keys were found, try to read the private keys in EC format
        if keys.is_empty() {
            let mut key_reader = std::io::BufReader::new(std::fs::File::open(key_path)?);
            keys = ec_private_keys(&mut key_reader)
                .filter_map(Result::ok)
                .map(|key| rustls::PrivateKey(key.secret_sec1_der().to_vec())) // Convert &[u8] to Vec<u8>
                .collect();
        }
    
        // If no certificate or key was found, return an error
        if cert_chain.is_empty() || keys.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid certificate or key"));
        }
    
        // Create a new server configuration with the certificate and key
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, keys.remove(0))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid certificate or key"))?;
    
        // Bind the server to the address with the configuration
        println!("Listening on https://{}", bind_address);
        server.bind_rustls(bind_address, config)?
    } else {
        // If no certificate or key was provided, bind the server to the address without TLS
        println!("Listening on http://{}", bind_address);
        server.bind(bind_address)?
    };

    // Run the server
    let server = server.run();

    // Wait for either the server to finish or a signal from the channel
    tokio::select! {
        _ = server => {},
        _ = rx => {
            println!("ENTER pressed, shutting down");
        }
    }

    Ok(())
}
