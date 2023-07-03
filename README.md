# Simple File Server

Simple File Server is a minimalist, yet powerful, file server written in Rust. It's designed to handle uploading, downloading, and deleting files, all while offering a clean and straightforward REST API.

## Features
- **File Upload**: Easily upload files to the server.
- **File Download**: Download stored files using a simple API.
- **File Deletion**: Delete existing files from the server.
- **Simple Control**: Use the command line to control the server, including defining the server port.

## How to Use

First, ensure that you have Rust installed on your system. If not, follow the instructions [here](https://www.rust-lang.org/tools/install) to install it.

Then, you can clone this repository:

```sh
git clone https://github.com/username/simple-file-server.git
cd simple-file-server
```
*Note: Replace 'username' with your actual GitHub username.*

Now, you can compile and run the server:

```sh
cargo build --release
cargo run --release
```
By default, the server will listen on port 3000. To specify a different port, provide it as a command-line argument:

```sh
cargo run --release 8080
```
In this example, the server would listen on port 8080.

Once the server is running, you can use the following endpoints:

- `POST /upload`: Upload a file to the server.
- `GET /{filename}`: Download a specific file from the server.
- `DELETE /{filename}`: Delete a specific file from the server.

Press `ENTER` to stop the server.

## Disclaimer

This server is a simple implementation intended for learning and demonstration purposes, and as such, it may not be suitable for production environments without further enhancements.
