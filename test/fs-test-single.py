import sys
import requests
import random
import string
import time

session = requests.Session()

def generate_random_text_file(filename, size):
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

def download_file(filename, server_url):
    url = f'{server_url}/{filename}'
    response = session.get(url)
    if response.status_code == 200:
        with open(filename, 'wb') as file:
            file.write(response.content)
    return response.status_code

def delete_file(filename, server_url):
    url = f'{server_url}/{filename}'
    response = session.delete(url)
    return response.status_code

def main():
    if len(sys.argv) < 4:
        print('Please provide a filename, file size, and server URL as command line arguments')
        return

    filename = sys.argv[1]
    size = int(sys.argv[2])
    server_url = sys.argv[3]

    # Try to download the file
    start_time = time.time()
    download_status = download_file(filename, server_url)
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
        download_status = download_file(filename, server_url)
        end_time = time.time()
        print(f'File downloaded: {filename}, Status: {download_status}, Time taken: {end_time - start_time:.2f}s')

    # Delete the file
    start_time = time.time()
    delete_status = delete_file(filename, server_url)
    end_time = time.time()
    print(f'File deleted: {filename}, Status: {delete_status}, Time taken: {end_time - start_time:.2f}s')

if __name__ == '__main__':
    main()
