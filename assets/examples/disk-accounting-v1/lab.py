#!/usr/bin/env python3
"""Built with Rust: read-only synthetic disk accounting, version 1 (2026-11-09).

Linux/Python 3.11, Dust 1.2.7 and dua 3.0.0 only. Install trusted upstream
binaries yourself. No downloads, user-directory scans or deletion commands.
Only this script's generated temporary fixture is removed at completion.

Syntax sources (also checked against each pinned executable's --help):
https://github.com/bootandy/dust/tree/v1.2.7
https://github.com/Byron/dua-cli/tree/v3.0.0
https://docs.python.org/3.11/library/os.html#os.stat_result
https://docs.python.org/3.11/library/tempfile.html#tempfile.TemporaryDirectory

Usage: python3 lab.py --dust /absolute/path/to/dust --dua /absolute/path/to/dua
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dust', required=True, type=Path)
    parser.add_argument('--dua', required=True, type=Path)
    args = parser.parse_args()
    if not sys.platform.startswith('linux'):
        raise RuntimeError('Linux only; other platforms are unchecked.')
    binaries = {}
    for name in ('dust', 'dua'):
        binary = getattr(args, name)
        if not binary.is_absolute() or not binary.is_file() or not os.access(binary, os.X_OK):
            raise RuntimeError(f'{name}: supply an absolute path to a trusted executable.')
        binaries[name] = str(binary.resolve())
    with tempfile.TemporaryDirectory(prefix='bwr-disk-lab-') as temp:
        root = Path(temp)
        fixture = root / 'fixture'
        fixture.mkdir()
        config = root / 'empty.toml'
        config.write_text('')
        env = {'PATH': '/usr/bin:/bin', 'HOME': str(root),
               'XDG_CONFIG_HOME': str(root / 'config'), 'LANG': 'C',
               'TERM': 'dumb', 'NO_COLOR': '1'}

        def run(command):
            r = subprocess.run(command, cwd=root, env=env, capture_output=True,
                               text=True, timeout=30)
            if r.returncode or r.stderr.strip():
                raise RuntimeError(f'{Path(command[0]).name}: execution did not complete cleanly.')
            return r.stdout

        versions = [run([binaries[n], '--version']).strip() for n in ('dust', 'dua')]
        if versions != ['Dust 1.2.7', 'dua 3.0.0']:
            raise RuntimeError(f'Unchecked versions: {versions}; expected Dust 1.2.7 / dua 3.0.0.')
        (fixture / 'ordinary').write_bytes(b'x' * 4096)
        os.link(fixture / 'ordinary', fixture / 'hardlink')
        with (fixture / 'sparse').open('wb') as stream:
            stream.truncate(8 * 1024 * 1024)
        names = ['ordinary', 'hardlink', 'sparse']
        paths = [str(fixture / name) for name in names]

        def manifest():
            files = sorted(fixture.iterdir())
            if sorted(x.name for x in files) != sorted(names):
                raise RuntimeError('Fixture paths changed.')
            return {f.name: {'size': f.stat().st_size, 'blocks': f.stat().st_blocks,
                             'device': f.stat().st_dev, 'inode': f.stat().st_ino,
                             'links': f.stat().st_nlink,
                             'sha256': hashlib.sha256(f.read_bytes()).hexdigest()}
                    for f in files}

        before = manifest()
        if before['ordinary']['inode'] != before['hardlink']['inode']:
            raise RuntimeError('Hard-link control unavailable; no acceptance result.')
        if before['sparse']['blocks'] * 512 >= before['sparse']['size']:
            raise RuntimeError('Sparse-file control unavailable; no acceptance result.')
        allocated = before['ordinary']['blocks'] * 512 + before['sparse']['blocks'] * 512
        unique_apparent = 4096 + 8 * 1024 * 1024
        per_path_apparent = 8192 + 8 * 1024 * 1024
        dust = [binaries['dust'], '--config', str(config), '-c', '-b', '-P', '-o', 'b']
        dua = [binaries['dua']]
        cases = [
            ('Dust allocated', dust + paths, allocated, 'dust'),
            ('Dust apparent', dust + ['-s'] + paths, per_path_apparent, 'dust'),
            ('dua allocated', dua + ['-f', 'bytes', 'aggregate'] + paths, allocated, 'dua'),
            ('dua apparent', dua + ['-A', '-f', 'bytes', 'aggregate'] + paths, unique_apparent, 'dua'),
            ('dua apparent, every link', dua + ['-A', '-l', '-f', 'bytes', 'aggregate'] + paths,
             per_path_apparent, 'dua'),
        ]
        observations = []
        for label, command, expected, tool in cases:
            output = run(command)
            pattern = r'^\s*(\d+)B[^\n]*\(total\)\s*$' if tool == 'dust' else r'^\s*(\d+) b total\s*$'
            totals = re.findall(pattern, output, re.MULTILINE)
            if len(totals) != 1 or int(totals[0]) != expected:
                raise RuntimeError(f'{label}: total mismatch; no acceptance result.')
            if manifest() != before:
                raise RuntimeError(f'{label}: fixture changed; no acceptance result.')
            observations.append({'case': label, 'observed_bytes': int(totals[0]),
                                 'expected_bytes': expected})
        print(json.dumps({'exercise': 'disk-accounting-v1', 'versions': versions,
                          'scope': 'Linux synthetic regular files; explicit file operands, no directories',
                          'observations': observations, 'fixture_unchanged': True,
                          'limits': 'Not deletion safety, reclaimable space, a speed benchmark or platform certification.'},
                         indent=2))


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, OSError, subprocess.TimeoutExpired) as error:
        print(f'EXERCISE FAILED: {error}', file=sys.stderr)
        sys.exit(1)
