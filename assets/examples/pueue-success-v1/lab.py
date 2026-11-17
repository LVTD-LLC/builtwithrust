#!/usr/bin/env python3
"""Pueue 4.0.4 success-boundary lab. Linux; Python 3.11+; Bash.

Downloads nothing. Starts only a private foreground daemon, using an explicit
temporary config/socket/data directory. Never connects to a user's queue.
All tasks are synthetic, have bounded waits, and touch only generated files.
"""
import argparse
import json
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile
import time


def main():
    if not __debug__:
        raise RuntimeError('Run without Python optimization; verification assertions are required')
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pueue', required=True, type=Path)
    ap.add_argument('--pueued', required=True, type=Path)
    args = ap.parse_args()
    client, daemon = args.pueue.resolve(), args.pueued.resolve()
    for binary in (client, daemon):
        version = subprocess.check_output([str(binary), '--version'], text=True, timeout=5).strip()
        if version.split()[-1] != '4.0.4':
            raise RuntimeError(f'Expected 4.0.4, got {version}')
    bash = shutil.which('bash')
    if not bash or os.name != 'posix' or not Path('/proc').exists():
        raise RuntimeError('This exercise requires Linux and Bash')
    results = []
    with tempfile.TemporaryDirectory(prefix='bwr-pueue-') as folder:
        root = Path(folder)
        config = root / 'config.yml'
        config.write_text('shared:\n'
                          f'  pueue_directory: {json.dumps(str(root / "data"))}\n'
                          '  use_unix_socket: true\n'
                          f'  unix_socket_path: {json.dumps(str(root / "queue.socket"))}\n'
                          'daemon:\n'
                          '  pause_group_on_failure: false\n'
                          '  pause_all_on_failure: false\n')
        # Do not copy the operator's environment into task records.
        env = {'PATH': os.environ.get('PATH', '/usr/bin:/bin'),
               'HOME': str(root), 'XDG_RUNTIME_DIR': str(root),
               'XDG_CONFIG_HOME': str(root), 'XDG_DATA_HOME': str(root),
               'LANG': 'C.UTF-8'}

        def run(*argv):
            return subprocess.run([str(client), '-c', str(config), *argv],
                                  cwd=root, env=env, capture_output=True,
                                  text=True, timeout=20)

        def state():
            reply = run('status', '--json')
            if reply.returncode:
                raise RuntimeError('Private daemon unavailable')
            return json.loads(reply.stdout)['tasks']

        def add(name, script, after=None):
            path = root / f'{name}.sh'
            path.write_text(script)
            flags = ['add', '--print-task-id']
            if after is not None:
                flags += ['--after', after]
            reply = run(*flags, '--', shlex.join([bash, str(path)]))
            if reply.returncode or not reply.stdout.strip().isdigit():
                raise RuntimeError('Enqueue failed')
            return reply.stdout.strip(), reply.returncode

        def check(name, task_id, expected_wait, marker, expected_marker, expected_result):
            # A wait on a missing ID is not sufficient evidence in 4.0.4.
            if task_id not in state():
                raise RuntimeError('Task missing before wait')
            reply = run('wait', '--quiet', '--status', 'success', task_id)
            tasks = state()
            if task_id not in tasks:
                raise RuntimeError('Task missing after wait')
            status = tasks[task_id]['status']
            actual_result = status['Done']['result'] if isinstance(status, dict) and 'Done' in status else status
            actual_marker = (root / marker).exists()
            assert reply.returncode == expected_wait, (name, reply.returncode)
            assert actual_marker == expected_marker, (name, actual_marker)
            assert actual_result == expected_result, (name, actual_result)
            results.append({'case': name, 'success_wait_exit': reply.returncode,
                            'result': actual_result, 'marker_exists': actual_marker})

        with (root / 'daemon.log').open('w') as log:
            proc = subprocess.Popen([str(daemon), '-c', str(config)], cwd=root,
                                    env=env, stdout=log, stderr=log)
            try:
                for _ in range(100):
                    if proc.poll() is not None:
                        raise RuntimeError('Private daemon exited')
                    if run('status', '--json').returncode == 0:
                        break
                    time.sleep(0.05)
                else:
                    raise RuntimeError('Private daemon did not become ready')
                failed, enqueue_exit = add('failed', 'exit 7\n')
                ordinary = run('wait', '--quiet', failed)
                assert ordinary.returncode == 0
                results.append({'case': 'enqueue-and-done', 'add_exit': enqueue_exit,
                                'done_wait_exit': ordinary.returncode})
                check('explicit-failure', failed, 1, 'never', False, {'Failed': 7})
                dependent, _ = add('blocked', 'printf ran > blocked.marker\n', failed)
                check('failed-dependency', dependent, 1, 'blocked.marker', False, 'DependencyFailed')
                masked, _ = add('masked', 'false | cat\nprintf ran > masked.marker\n')
                check('masked-pipeline', masked, 0, 'masked.marker', True, 'Success')
                strict, _ = add('strict', 'set -e -o pipefail\nfalse | cat\nprintf ran > strict.marker\n')
                check('strict-pipeline', strict, 1, 'strict.marker', False, {'Failed': 1})
                good, _ = add('producer', 'printf sample > input.txt\n')
                check('successful-producer', good, 0, 'input.txt', True, 'Success')
                consumer, _ = add('consumer', 'test "$(cat input.txt)" = sample && printf checked > checked.txt\n', good)
                check('successful-dependency', consumer, 0, 'checked.txt', True, 'Success')
                assert (root / 'checked.txt').read_text() == 'checked'
                print(json.dumps({'pueue': '4.0.4', 'platform': 'Linux',
                                  'checks': results, 'artifact_content_checked': True}, indent=2))
            finally:
                # Address only the explicit private config, never a default queue.
                try:
                    run('shutdown')
                    proc.wait(timeout=5)
                except (subprocess.TimeoutExpired, OSError):
                    proc.terminate()
                    try:
                        proc.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                        proc.wait(timeout=5)


if __name__ == '__main__':
    main()
