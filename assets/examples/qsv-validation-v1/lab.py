#!/usr/bin/env python3
"""Disposable qsv validation exercise. No import, network or real data access.

Python 3 standard library only. Install qsv separately; inspect this script first.
Runs in a new temporary directory, asserts observed outcomes, then removes it.
Tested with qsv 26.0.0 on Linux. Differences on another version need investigation.
"""
import argparse
import csv
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

SCHEMA = {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": {
        "order_id": {"type": "string", "pattern": "^[0-9]{4}$"},
        "quantity": {"type": "integer", "minimum": 1},
        "status": {"type": "string", "enum": ["ready", "hold"]},
    },
    "required": ["order_id", "quantity", "status"],
    "additionalProperties": False,
}
HEADER = "order_id,quantity,status\n"
# label, input, structural exit, schema exit, expected accepted/rejected data rows
CASES = [
    ("good", HEADER + "0007,3,ready\n0042,1,hold\n", 0, 0, None),
    ("mixed", HEADER + "0007,3,ready\n0042,-2,ready\n0050,1,sent\n", 0, 1, (1, 2)),
    ("ragged", HEADER + "0007,3\n", 1, 1, None),
    ("empty-id", HEADER + ",3,ready\n", 0, 1, (0, 1)),
    ("extra-column", "order_id,quantity,status,note\n0007,3,ready,hello\n", 0, 1, (0, 1)),
    ("duplicate", HEADER + "0007,3,ready\n0007,3,ready\n", 0, 0, None),
]


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--qsv", default="qsv", help="qsv executable name or path")
    args = parser.parse_args()
    executable = shutil.which(args.qsv)
    if executable is None:
        parser.error("qsv not found; install it separately and check qsv validate --help")
    executable = str(Path(executable).resolve())
    # Do not inherit qsv parser settings or a caller's custom dotenv path.
    env = {k: v for k, v in os.environ.items() if not k.startswith("QSV_")}
    env["QSV_DOTENV_PATH"] = "<NONE>"  # Disable ancestor and binary-adjacent dotenv loading.
    with tempfile.TemporaryDirectory(prefix="qsv-validation-") as tmp:
        root = Path(tmp)
        (root / "orders.schema.json").write_text(json.dumps(SCHEMA, indent=2), encoding="utf-8")

        def run(*arguments):
            return subprocess.run(
                [executable, *arguments], cwd=root, env=env,
                text=True, capture_output=True, timeout=60, check=False,
            )

        version = run("--version")
        require(version.returncode == 0, version.stderr)
        print(version.stdout.strip())
        schema_ok = run("validate", "schema", "orders.schema.json")
        require(schema_ok.returncode == 0, schema_ok.stderr)
        print("label | structural exit | schema exit | mock import gate")
        for label, text, expected_structure, expected_schema, split in CASES:
            name = label + ".csv"
            source = root / name
            source.write_text(text, encoding="utf-8")
            before = source.read_bytes()
            structure = run("validate", name)
            checked = run("validate", name, "orders.schema.json")
            require(structure.returncode == expected_structure,
                    f"{label}: unexpected structural result: {structure.stdout} {structure.stderr}")
            require(checked.returncode == expected_schema,
                    f"{label}: unexpected schema result: {checked.stdout} {checked.stderr}")
            require(source.read_bytes() == before, label + ": source changed")
            marker = root / (label + ".mock-import")
            # This marker substitutes for an import; no destination is contacted.
            if structure.returncode == 0 and checked.returncode == 0:
                marker.write_text("not a real import", encoding="utf-8")
            require(marker.exists() == (expected_structure == expected_schema == 0),
                    label + ": import gate disagrees with exit codes")
            print(f"{label} | {structure.returncode} | {checked.returncode} | "
                  + ("allowed" if marker.exists() else "blocked"))
            if split is not None:
                counts = []
                for suffix in ("valid", "invalid"):
                    with (root / (name + "." + suffix)).open(newline="", encoding="utf-8") as f:
                        counts.append(len(list(csv.DictReader(f))))
                require(tuple(counts) == split, label + ": unexpected quarantine counts")
                print(f"  accepted/rejected rows: {counts[0]}/{counts[1]}")
                print((root / (name + ".validation-errors.tsv")).read_text(encoding="utf-8").strip())
            elif expected_schema == 0:
                require(not list(root.glob(name + ".*")), label + ": unexpected sidecars")
            if label == "good":
                rows = list(csv.DictReader(io.StringIO(source.read_text(encoding="utf-8"))))
                require([r["order_id"] for r in rows] == ["0007", "0042"], "ID text lost")
        print("PASS: 12 data-validation runs, schema check, gates, sidecars and source preservation.")
        print("Duplicate IDs passed this schema. Add the destination's uniqueness checks separately.")
        print("Temporary fixtures and mock markers are now removed on exit.")


if __name__ == "__main__":
    main()
