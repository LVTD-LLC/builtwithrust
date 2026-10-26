#!/usr/bin/env python3
"""Four disposable review cases; no repository/configuration changes or network calls.

Run with Python 3, Git and Difftastic on PATH. Output was checked with
Difftastic 0.71.0, Git 2.39.5 and Python 3.11.2 on Linux, 2026-10-27.
The temporary directory is removed automatically. This is not a benchmark
or a proof that a real change preserves behavior.
"""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


CASES = [
    (
        "layout-only", ".js",
        "const config = { retries: 3, timeout: 1000 };\n",
        "const config = {\n  retries: 3,\n  timeout: 1000\n};\n",
        0, "No syntactic changes",
    ),
    (
        "value-edit", ".js",
        "const config = { retries: 3, timeout: 1000 };\n",
        "const config = {\n  retries: 4,\n  timeout: 1000\n};\n",
        1, "JavaScript",
    ),
    (
        "significant-indent", ".py",
        "if enabled:\n    first()\n    second()\n",
        "if enabled:\n    first()\nsecond()\n",
        1, "Python",
    ),
    (
        "parse-fallback", ".js",
        "const answer = 1;\n",
        "const answer = ;\n",
        1, "parse error",
    ),
]


def main():
    for tool in ("git", "difft"):
        if shutil.which(tool) is None:
            raise SystemExit("Install " + tool + " and put it on PATH first.")
    # Avoid silently inheriting syntax overrides, ignored comments or size limits.
    env = {k: v for k, v in os.environ.items() if not k.startswith(("DFT_", "GIT_"))}
    # Inherited Git routing/configuration must never select a caller's index or filters.
    env.update(GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull,
               GIT_CONFIG_SYSTEM=os.devnull)
    for command in (["git", "--version"], ["difft", "--version"]):
        print(subprocess.check_output(command, text=True, env=env).strip())
    with tempfile.TemporaryDirectory(prefix="difft-review-") as folder:
        root = Path(folder)
        for name, suffix, before, after, expected, marker in CASES:
            case = root / name
            case.mkdir()
            old, new = case / ("before" + suffix), case / ("after" + suffix)
            old.write_text(before, encoding="utf-8")
            new.write_text(after, encoding="utf-8")
            commands = [
                ["git", "--no-pager", "diff", "--no-index", "--no-ext-diff",
                 "--no-textconv", "--color=never", str(old), str(new)],
                ["difft", "--color=never", "--exit-code", str(old), str(new)],
            ]
            print("\n=== " + name + " ===")
            for index, command in enumerate(commands):
                result = subprocess.run(command, cwd=root, env=env, text=True,
                                        capture_output=True, check=False)
                print(("git" if index == 0 else "difft") + " exit: " + str(result.returncode))
                print(result.stdout, end="")
                if result.stderr:
                    print(result.stderr, end="")
                want = 1 if index == 0 else expected
                if result.returncode != want:
                    raise SystemExit("Unexpected result; inspect this version before adopting it.")
                if index == 1 and marker.lower() not in result.stdout.lower():
                    raise SystemExit("Expected display marker missing; inspect the output/version.")
        # A separate, disposable Git index: no commit, identity or global config.
        repo = root / "patch-round-trip"
        repo.mkdir()
        subprocess.run(["git", "init", "--quiet", "--template=", str(repo)], check=True, env=env)
        source = repo / "sample.js"
        before, after = CASES[1][2:4]
        source.write_text(before, encoding="utf-8")
        subprocess.run(["git", "add", "sample.js"], cwd=repo, env=env, check=True)
        source.write_text(after, encoding="utf-8")
        review = subprocess.run(
            ["git", "--no-pager", "-c", "diff.external=difft", "diff",
             "--ext-diff", "--", "sample.js"], cwd=repo, env=env,
            text=True, capture_output=True, check=True)
        if "JavaScript" not in review.stdout:
            raise SystemExit("External diff trial did not use the expected language.")
        patch = subprocess.check_output(
            ["git", "--no-pager", "diff", "--no-ext-diff", "--no-textconv",
             "--binary", "--", "sample.js"], cwd=repo, env=env)
        source.write_text(before, encoding="utf-8")
        for args in (["--check"], []):
            subprocess.run(["git", "apply"] + args + ["-"], input=patch,
                           cwd=repo, env=env, check=True)
        if source.read_text(encoding="utf-8") != after:
            raise SystemExit("Patch did not recreate the expected file.")
        print("\nTemporary Git opt-in review and patch round-trip: passed.")
    print("Four cases checked. Syntax comparison is not behavior verification.")


if __name__ == "__main__":
    main()
