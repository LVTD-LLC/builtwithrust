#!/usr/bin/env python3
"""Disposable loopback comparison: Dufs 0.47.0 and miniserve 0.35.0.

Python 3.11+, standard library only. Supply the two trusted executable paths.
No customer paths, production servers, inherited tool settings, or downloads.
"""
import argparse
import base64
import contextlib
import hashlib
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def request(port, method, path, body=None, account=None, headers=None):
    fields = dict(headers or {})
    if account:
        fields['Authorization'] = 'Basic ' + base64.b64encode(account.encode()).decode()
    req = urllib.request.Request(f'http://127.0.0.1:{port}{path}', data=body,
                                 method=method, headers=fields)
    # Do not use environment proxy settings, including for the loopback trial.
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, req, fp, code, msg, headers, newurl):
            return None

    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())
    try:
        with opener.open(req, timeout=5) as response:
            return response.status, response.read()
    except urllib.error.HTTPError as error:
        return error.code, error.read()


@contextlib.contextmanager
def server(binary, kind, root, flags):
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    # A free-port race can fail startup; never retry against a different host.
    interface = ['--bind', '127.0.0.1'] if kind == 'dufs' else ['--interfaces', '127.0.0.1']
    env = {'PATH': os.defpath, 'HOME': str(root.parent), 'LANG': 'C.UTF-8'}
    command = [str(binary), *interface, '--port', str(port), *flags, '--', str(root)]
    with tempfile.TemporaryFile() as log:
        process = subprocess.Popen(command, cwd=root.parent, env=env, stdout=log, stderr=log)
        try:
            deadline = time.monotonic() + 10
            while True:
                require(process.poll() is None, f'{kind} stopped during startup')
                try:
                    with socket.create_connection(('127.0.0.1', port), timeout=0.2):
                        break
                except OSError:
                    require(time.monotonic() < deadline, f'{kind} startup timed out')
                    time.sleep(0.05)
            yield port
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def upload(port, account, name, content):
    boundary = 'bwr' + secrets.token_hex(12)
    body = (f'--{boundary}\r\nContent-Disposition: form-data; name="path"; '
            f'filename="{name}"\r\nContent-Type: application/octet-stream\r\n\r\n').encode()
    body += content + f'\r\n--{boundary}--\r\n'.encode()
    return request(port, 'POST', '/upload?path=/inbox', body, account,
                   {'Content-Type': f'multipart/form-data; boundary={boundary}'})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dufs', required=True, type=Path)
    parser.add_argument('--miniserve', required=True, type=Path)
    args = parser.parse_args()
    binaries = {name: getattr(args, name).expanduser().resolve(strict=True)
                for name in ('dufs', 'miniserve')}
    versions = {}
    for name, binary in binaries.items():
        version = subprocess.check_output([str(binary), '--version'], text=True, timeout=10).strip()
        expected = {'dufs': 'dufs 0.47.0', 'miniserve': 'miniserve 0.35.0'}[name]
        require(version == expected, f'Expected {expected}; got {version}')
        versions[name] = {'version': version, 'sha256': hashlib.sha256(binary.read_bytes()).hexdigest()}
    results = []
    writer = 'writer:' + secrets.token_hex(16)
    reader = 'reader:' + secrets.token_hex(16)
    with tempfile.TemporaryDirectory(prefix='bwr-file-server-') as temporary:
        base = Path(temporary)

        def fixture(name):
            root = base / name / 'share'
            (root / 'inbox').mkdir(parents=True)
            (root / 'release.txt').write_bytes(b'release-v1\n')
            return root

        def record(case, status, expected, condition):
            require(status == expected and condition, f'{case}: status={status}, expected={expected}, state={condition}')
            results.append({'case': case, 'status': status, 'state_check': 'pass'})

        root = fixture('dufs-gate')
        with server(binaries['dufs'], 'dufs', root, ['--auth', writer + '@/:rw']) as port:
            status, body = request(port, 'GET', '/release.txt', account=writer)
            record('dufs: rw account reads', status, 200, body == b'release-v1\n')
            status, _ = request(port, 'PUT', '/inbox/no.txt', b'no\n', writer)
            record('dufs: rw without global upload cannot create', status, 403, not (root / 'inbox/no.txt').exists())

        root = fixture('dufs-roles')
        flags = ['--allow-upload', '--auth', writer + '@/inbox:rw', '--auth', reader + '@/:ro']
        with server(binaries['dufs'], 'dufs', root, flags) as port:
            status, _ = request(port, 'PUT', '/inbox/yes.txt', b'writer-v1\n', writer)
            record('dufs: writer creates inside allowed path', status, 201, (root / 'inbox/yes.txt').read_bytes() == b'writer-v1\n')
            status, body = request(port, 'GET', '/inbox/yes.txt', account=reader)
            record('dufs: reader reads exact uploaded bytes', status, 200, body == b'writer-v1\n')
            status, _ = request(port, 'PUT', '/inbox/reader.txt', b'no\n', reader)
            record('dufs: reader cannot upload', status, 403, not (root / 'inbox/reader.txt').exists())
            status, _ = request(port, 'PUT', '/outside.txt', b'no\n', writer)
            record('dufs: writer cannot create outside allowed path', status, 403, not (root / 'outside.txt').exists())
            status, _ = request(port, 'DELETE', '/inbox/yes.txt', account=writer)
            record('dufs: delete remains globally disabled', status, 403, (root / 'inbox/yes.txt').read_bytes() == b'writer-v1\n')

        root = fixture('miniserve-dav')
        with server(binaries['miniserve'], 'miniserve', root, ['--auth', writer, '--enable-webdav']) as port:
            status, _ = request(port, 'GET', '/release.txt')
            record('miniserve: anonymous read challenged', status, 401, (root / 'release.txt').read_bytes() == b'release-v1\n')
            status, body = request(port, 'GET', '/release.txt', account=writer)
            record('miniserve: authenticated read returns exact bytes', status, 200, body == b'release-v1\n')
            status, body = request(port, 'PROPFIND', '/', account=writer, headers={'Depth': '1'})
            record('miniserve: DAV listing works', status, 207, b'release.txt' in body)
            status, _ = request(port, 'PUT', '/inbox/dav.txt', b'no\n', writer)
            record('miniserve: DAV listing does not enable PUT', status, 404, not (root / 'inbox/dav.txt').exists())

        root = fixture('miniserve-upload')
        flags = ['--auth', writer, '--auth', reader, '--enable-webdav', '--upload-files', '/inbox']
        with server(binaries['miniserve'], 'miniserve', root, flags) as port:
            status, _ = upload(port, writer, 'writer.txt', b'writer-v1\n')
            record('miniserve: first authenticated account uploads', status, 303, (root / 'inbox/writer.txt').read_bytes() == b'writer-v1\n')
            status, _ = upload(port, reader, 'reader.txt', b'reader-v1\n')
            record('miniserve: second account also uploads', status, 303, (root / 'inbox/reader.txt').read_bytes() == b'reader-v1\n')
            status, _ = request(port, 'PUT', '/inbox/dav.txt', b'no\n', writer)
            record('miniserve: browser upload does not enable DAV PUT', status, 404, not (root / 'inbox/dav.txt').exists())

    print(json.dumps({'versions': versions, 'checks': results}, indent=2))


if __name__ == '__main__':
    main()
