#!/usr/bin/env python3
"""Synthetic local restore exercise, checked with rustic 0.11.4 / Python 3.11.

Read before running. Accepts a rustic binary, NOT a repository or source path.
Creates a new temporary workspace and leaves it for inspection. No deletion,
network backend, shell, inherited rustic options, or real backup data.

Command provenance (v0.11.4): https://github.com/rustic-rs/rustic/tree/v0.11.4/src/commands
Profile resolution: https://github.com/rustic-rs/rustic/blob/v0.11.4/src/config.rs
Python APIs: https://docs.python.org/3.11/library/index.html
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import sys
import tempfile


def manifest(directory):
    return {str(p.relative_to(directory)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(directory.rglob('*')) if p.is_file()}


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('rustic_binary', type=Path)
    args = parser.parse_args()
    binary = args.rustic_binary.resolve(strict=True)
    if not binary.is_file():
        raise RuntimeError('Expected the path to an installed rustic binary')
    root = Path(tempfile.mkdtemp(prefix='bwr-rustic-restore-'))
    root.chmod(0o700)
    print('Disposable workspace:', root, flush=True)
    (root / 'empty.toml').write_text('', encoding='utf-8')
    password = root / 'password'
    password.write_text(secrets.token_hex(32), encoding='utf-8')
    password.chmod(0o600)
    wrong = root / 'wrong-password'
    wrong.write_text(secrets.token_hex(32), encoding='utf-8')
    wrong.chmod(0o600)
    # Explicit empty profile, local repository/cache and whitelisted child env:
    # no inherited RUSTIC_* options, profile hooks or remote credentials.
    env = {'PATH': os.defpath, 'LANG': 'C.UTF-8'}
    base = [str(binary), '--use-profile', str(root / 'empty.toml'),
            '--no-progress', '--cache-dir', str(root / 'cache'),
            '--repository', str(root / 'repo'), '--password-file', str(password)]
    observations = []

    def run(name, commands, expected_success=True, invocation=None):
        result = subprocess.run((invocation or base) + commands, cwd=root,
                                env=env, stdin=subprocess.DEVNULL,
                                capture_output=True, text=True, timeout=60)
        # Do not print passwords or dump config/logs into reports.
        if (result.returncode == 0) != expected_success:
            raise RuntimeError(f'{name}: unexpected exit {result.returncode}; '
                               'inspect the fixture and local tool version')
        observations.append({'case': name, 'exit': result.returncode})
        return result

    version = run('version', ['--version']).stdout.strip()
    if version != 'rustic v0.11.4':
        raise RuntimeError(f'Fixture requires rustic v0.11.4, got {version!r}')
    source = root / 'source'
    (source / 'nested').mkdir(parents=True)
    (source / 'note.txt').write_bytes(b'first version\n')
    (source / 'nested' / 'empty.txt').write_bytes(b'')
    (source / 'nested' / 'data.bin').write_bytes(bytes(range(256)) * 17)
    expected_first = manifest(source)
    run('init', ['init'])
    first = json.loads(run('backup-first', ['backup', '--json', '--as-path',
                                          'fixture', 'source']).stdout)['id']
    run('restore-first', ['restore', '--no-ownership', f'{first}:fixture/', 'first'])
    require(manifest(root / 'first') == expected_first, 'First restore mismatch')
    (source / 'note.txt').write_bytes(b'second version, changed length\n')
    (source / 'nested' / 'added.txt').write_bytes(b'new file\n')
    expected_second = manifest(source)
    second = json.loads(run('backup-second', ['backup', '--json', '--as-path',
                                            'fixture', 'source']).stdout)['id']
    run('restore-second', ['restore', '--no-ownership', f'{second}:fixture/', 'second'])
    require(manifest(root / 'second') == expected_second, 'Second restore mismatch')
    run('restore-earlier', ['restore', '--no-ownership', f'{first}:fixture/', 'earlier'])
    require(manifest(root / 'earlier') == expected_first, 'Earlier snapshot mismatch')
    run('check-read-data', ['check', '--read-data'])
    target = root / 'second' / 'note.txt'
    original = target.read_bytes()
    before = target.stat()
    target.write_bytes(b'X' * len(original))
    os.utime(target, ns=(before.st_atime_ns, before.st_mtime_ns))
    require(target.stat().st_mtime_ns == before.st_mtime_ns, 'Cannot preserve mtime')
    run('same-metadata-default', ['restore', '--no-ownership', f'{second}:fixture/', 'second'])
    default_matches = manifest(root / 'second') == expected_second
    require(not default_matches, 'Expected default to trust size and mtime; investigate version/filesystem')
    run('same-metadata-verified', ['restore', '--no-ownership', '--verify-existing',
                                 f'{second}:fixture/', 'second'])
    require(manifest(root / 'second') == expected_second, 'Verified restore mismatch')
    (root / 'second' / 'local-only.txt').write_bytes(b'not in snapshot\n')
    run('additional-file-default', ['restore', '--no-ownership', f'{second}:fixture/', 'second'])
    extras_retained = (root / 'second' / 'local-only.txt').exists()
    require(extras_retained, 'Expected default to retain additional file')
    wrong_base = base.copy()
    wrong_base[-1] = str(wrong)
    run('wrong-password', ['restore', '--no-ownership', f'{second}:fixture/', 'wrong-output'],
        expected_success=False, invocation=wrong_base)
    require(not (root / 'wrong-output').exists(), 'Unexpected wrong-password output')
    absent_base = base.copy()
    absent_base[absent_base.index('--repository') + 1] = str(root / 'absent-repo')
    run('unavailable-repository', ['restore', '--no-ownership', f'{second}:fixture/', 'absent-output'],
        expected_success=False, invocation=absent_base)
    require(not (root / 'absent-output').exists(), 'Unexpected absent-repository output')
    report = {'version': version, 'cases': observations,
              'clean_first_second_earlier_hashes_match': True,
              'default_same_metadata_bytes_match': default_matches,
              'verify_existing_bytes_match': True,
              'additional_file_retained': extras_retained,
              'scope': 'synthetic file contents and paths only; not metadata, application, '
                       'remote storage, concurrency, durability or security certification'}
    (root / 'results.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))
    print('Complete. Workspace retained; it contains only generated fixture files and passwords.')
    print('Inspect results.json, then remove this exact temporary directory when finished.')


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, RuntimeError, AssertionError,
            subprocess.TimeoutExpired) as error:
        print('Exercise did not complete:', error, file=sys.stderr)
        sys.exit(1)
