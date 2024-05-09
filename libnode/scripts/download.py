assert __name__ == "__main__"

import urllib.request
import tarfile
import os

from . import config

def download_file(url, filename):
    i = 0
    def reporthook(blocknum, blocksize, totalsize):
        readsofar = blocknum * blocksize
        if totalsize > 0:
            percent = readsofar * 1e2 / totalsize
            if percent > 100:
                percent = 100
            
            nonlocal i
            i += 1
            if i % 5 == 0:
                print("\r%5.1f%% (%*d / %d)" % (percent, len(str(totalsize)), readsofar, totalsize), end='')
        else:
            print("read %d" % (readsofar,), end='')
    
    urllib.request.urlretrieve(url, filename=filename, reporthook=reporthook)

def extract_tar(f):
    def extract_tar_iter(f):
        i = 0
        total_bytes = os.stat(f).st_size
        with open(f, "rb") as file_obj, tarfile.open(fileobj=file_obj, mode="r:gz") as tar:
            for member in tar.getmembers():
                f = tar.extractfile(member)
                i += 1
                if i % 100 == 0:
                    print(f"\r{file_obj.tell()/total_bytes*100:.1f}% ({file_obj.tell()} / {total_bytes})", end='')

                if f is not None:
                    content = f.read()
                    yield member.path, content
    
    for e in extract_tar_iter(f):
        os.makedirs(os.path.dirname(e[0]), exist_ok=True)
        with open(e[0], "wb") as f:
            f.write(e[1])

url = 'https://nodejs.org/dist/{}/node-{}.tar.gz'.format(config.nodeVersion, config.nodeVersion)

print('Downloading...')
download_file(url, "node_src.tar.gz")
print()

print("Extracting...")
extract_tar("node_src.tar.gz")

os.remove('node_src.tar.gz')
