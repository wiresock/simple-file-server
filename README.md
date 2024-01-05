# Simple HTTP File Server

Simple File Server is a minimalist, yet powerful, file server written in Rust. It's designed to handle uploading, downloading, and deleting files, all while offering a clean and straightforward REST API.

## Features
- **File Upload**: Easily upload files to the server.
- **File Download**: Download stored files using a simple API. The server supports both direct and chunked downloads.
- **File Deletion**: Delete existing files from the server.
- **Simple Control**: Use the command line to control the server, including defining the server port.
- **HTTPS Support**: Optionally run the server in HTTPS mode for secure data transmission, using TLS certificates.

## How to Use

First, ensure that you have Rust installed on your system. If not, follow the instructions [here](https://www.rust-lang.org/tools/install) to install it.

Then, clone this repository:

```sh
git clone https://github.com/wiresock/simple-file-server.git
cd simple-file-server
```

Compile and run the server:

```sh
cargo build --release
cargo run --release
```

By default, the server listens on port 3000. You can specify a different port using the `--port` argument:

```sh
cargo run --release -- --port 8080
```

In this example, the server listens on port 8080.

To enable HTTPS, specify the paths to your TLS certificate and key using the `--tls-cert` and `--tls-key` arguments:

```sh
cargo run --release -- --port 443 --tls-cert /path/to/cert.pem --tls-key /path/to/key.pem
```

Server Endpoints:

- `POST /upload`: Upload a file to the server.
- `GET /download/{filename}`: Download a specific file from the server as a single blob. Suitable for small to medium-sized files.
- `GET /download-chunked/{filename}`: Download a specific file from the server in chunks. More efficient for large files.
- `DELETE /{filename}`: Delete a specific file from the server.

Press `ENTER` to stop the server.

## Disclaimer

This server is intended for learning and demonstration purposes. While it now includes HTTPS support, further enhancements may be required for production use.
