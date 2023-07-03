import sys
import requests
import random
import string
import time

# Rust web server URL
SERVER_URL = 'http://localhost:3000'

def generate_random_text_file(filename, size):
    with open(filename, 'w') as file:
        generated_size = 0
        while generated_size < size:
            chunk_size = min(size - generated_size, 1024)
            chunk = ''.join(random.choices(string.ascii_letters + string.digits, k=chunk_size))
            file.write(chunk)
            generated_size += chunk_size

def upload_file(filename):
    url = f'{SERVER_URL}/upload'
    with open(filename, 'rb') as f:
        files = {'file': f}
        response = requests.post(url, files=files)
    return response.status_code

def download_file(filename):
    url = f'{SERVER_URL}/{filename}'
    response = requests.get(url)
    if response.status_code == 200:
        with open(filename, 'wb') as file:
            file.write(response.content)
    return response.status_code

def delete_file(filename):
    url = f'{SERVER_URL}/{filename}'
    response = requests.delete(url)
    return response.status_code

def main():
    if len(sys.argv) < 3:
        print('Please provide a filename and file size as command line arguments')
        return

    filename = sys.argv[1]
    size = int(sys.argv[2])

    # Generate a random text file
    start_time = time.time()
    generate_random_text_file(filename, size)
    end_time = time.time()
    print(f'File generated: {filename}, Size: {size} bytes, Time taken: {end_time - start_time:.2f}s')

    # Upload the file
    start_time = time.time()
    upload_status = upload_file(filename)
    end_time = time.time()
    print(f'File uploaded: {filename}, Status: {upload_status}, Time taken: {end_time - start_time:.2f}s')

    # Download the file
    start_time = time.time()
    download_status = download_file(filename)
    end_time = time.time()
    print(f'File downloaded: {filename}, Status: {download_status}, Time taken: {end_time - start_time:.2f}s')

    # Delete the file
    start_time = time.time()
    delete_status = delete_file(filename)
    end_time = time.time()
    print(f'File deleted: {filename}, Status: {delete_status}, Time taken: {end_time - start_time:.2f}s')

if __name__ == '__main__':
    main()
