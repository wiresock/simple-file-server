import sys
import requests
import random
import string
import time
import os

session = requests.Session()

def generate_random_text_file(filename, size):
    # Check if the file exists and if it has the correct size
    if os.path.exists(filename) and os.path.getsize(filename) == size:
        print(f'File: {filename} already exists with the correct size of {size} bytes.')
        return

    block_size = 1024
    block = ''.join(random.choices(string.ascii_letters + string.digits, k=block_size))

    with open(filename, 'w') as file:
        generated_size = 0
        while generated_size < size:
            chunk_size = min(size - generated_size, block_size)
            file.write(block[:chunk_size])
            generated_size += chunk_size

def upload_file(filename, server_url):
    url = f'{server_url}/upload'
    with open(filename, 'rb') as f:
        files = {'file': f}
        response = session.post(url, files=files)
    return response.status_code

def download_file_chunked(filename, server_url):
    download_path = f'{server_url}/download-chunked/{filename}'
    response = session.get(download_path)
    if response.status_code == 200:
        with open(filename, 'wb') as file:
            file.write(response.content)
    return response.status_code

def download_file_regular(filename, server_url):
    download_path = f'{server_url}/download/{filename}'
    response = session.get(download_path)
    if response.status_code == 200:
        with open(filename, 'wb') as file:
            file.write(response.content)
    return response.status_code

def delete_file(filename, server_url):
    url = f'{server_url}/{filename}'
    response = session.delete(url)
    return response.status_code

def main():
    if len(sys.argv) < 5:
        print('Please provide a command (download-chunked or download), filename, file size, and server URL as command line arguments')
        return

    command = sys.argv[1]
    filename = sys.argv[2]
    size = int(sys.argv[3])
    server_url = sys.argv[4]

    # Try to download the file
    start_time = time.time()
    if command == 'download-chunked':
        download_status = download_file_chunked(filename, server_url)
    elif command == 'download':
        download_status = download_file_regular(filename, server_url)
    else:
        print('Invalid command. Please use "download-chunked" or "download".')
        return
    end_time = time.time()

    if download_status == 200:
        print(f'File downloaded: {filename}, Status: {download_status}, Time taken: {end_time - start_time:.2f}s')
    else:
        print('File does not exist on the server. Uploading the file...')
        # Generate a random text file
        start_time = time.time()
        generate_random_text_file(filename, size)
        end_time = time.time()
        print(f'File generated: {filename}, Size: {size} bytes, Time taken: {end_time - start_time:.2f}s')

        # Upload the file
        start_time = time.time()
        upload_status = upload_file(filename, server_url)
        end_time = time.time()
        print(f'File uploaded: {filename}, Status: {upload_status}, Time taken: {end_time - start_time:.2f}s')

        # Download the file
        start_time = time.time()
        if command == 'download-chunked':
            download_status = download_file_chunked(filename, server_url)
        elif command == 'download':
            download_status = download_file_regular(filename, server_url)
        end_time = time.time()
        print(f'File downloaded: {filename}, Status: {download_status}, Time taken: {end_time - start_time:.2f}s')

    # Delete the file
    start_time = time.time()
    delete_status = delete_file(filename, server_url)
    end_time = time.time()
    print(f'File deleted: {filename}, Status: {delete_status}, Time taken: {end_time - start_time:.2f}s')

if __name__ == '__main__':
    main()


