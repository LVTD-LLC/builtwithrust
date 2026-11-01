#!/usr/bin/env python3
"""Built with Rust's synthetic Polars 2.1.0 CSV inference exercise.

Run with Python >=3.11 and polars==2.1.0. No files, network, or real data.
API provenance: official py-2.1.0 CSV and Series source:
https://github.com/pola-rs/polars/blob/py-2.1.0/py-polars/src/polars/io/csv/functions.py
https://github.com/pola-rs/polars/blob/py-2.1.0/py-polars/src/polars/series/series.py
"""

import csv
import io
import json
import platform
import sys

import polars as pl


def check(condition, message):
    # Unlike an assert, these checks are not disabled by python -O.
    if not condition:
        raise RuntimeError(message)


def main():
    check(pl.__version__ == "2.1.0", "This exercise requires polars==2.1.0; do not treat another version as verified.")
    check(sys.version_info >= (3, 11), "Python 3.11 or later is required.")
    short = b"order_id,amount\n0007,12\n0042,34\n"
    quoted = b'order_id,amount\n"0007",12\n"0042",34\n'
    # A text amount deliberately lies outside the first 100 data rows.
    late = ("order_id,amount\n" + "".join(f"{i:04d},{i}\n" for i in range(1, 101)) + "0101,unknown\n").encode()
    original = late
    expected = list(csv.DictReader(io.StringIO(late.decode())))
    expected_ids = [row["order_id"] for row in expected]
    expected_amounts = [row["amount"] for row in expected]
    check(len(expected) == 101 and expected_ids[0] == "0001" and expected_ids[-1] == "0101", "Input oracle failed.")
    results = []

    def observed(name, frame):
        results.append({"case": name, "rows": frame.height, "schema": {k: str(v) for k, v in frame.schema.items()},
                        "first_id": frame["order_id"][0], "last_amount": frame["amount"][-1],
                        "nulls": {k: frame[k].null_count() for k in frame.columns}})

    for name, data in [("numeric_identifiers", short), ("quoted_identifiers", quoted)]:
        frame = pl.read_csv(data)
        check(frame.schema == {"order_id": pl.Int64, "amount": pl.Int64}, f"{name}: unexpected schema.")
        check(frame.height == 2 and frame["amount"].to_list() == [12, 34], f"{name}: amount or row mismatch.")
        check(all(frame[col].null_count() == 0 for col in frame.columns), f"{name}: unexpected null.")
        check(frame["order_id"].to_list() == [7, 42], f"{name}: expected numeric inference, not identifier preservation.")
        observed(name, frame)

    for name, options in [("late_default", {}), ("id_override_only", {"schema_overrides": {"order_id": pl.String}})]:
        try:
            pl.read_csv(late, **options)
        except pl.exceptions.ComputeError:
            results.append({"case": name, "outcome": "ComputeError"})
        else:
            raise RuntimeError(f"{name}: expected late amount parsing to fail.")

    whole = pl.read_csv(late, infer_schema_length=None)
    check(whole.schema == {"order_id": pl.Int64, "amount": pl.String}, "Full inference schema mismatch.")
    check(all(whole[col].null_count() == 0 for col in whole.columns), "Full inference unexpected null.")
    check(whole.height == 101 and whole["amount"].to_list() == expected_amounts, "Full inference amount mismatch.")
    check(whole["order_id"].to_list() == list(range(1, 102)), "Full inference ID mismatch.")
    observed("full_inference", whole)

    for name, options in [("explicit_strings", {"schema_overrides": {"order_id": pl.String, "amount": pl.String}}),
                          ("all_strings", {"infer_schema": False})]:
        frame = pl.read_csv(late, **options)
        check(frame.schema == {"order_id": pl.String, "amount": pl.String}, f"{name}: unexpected schema.")
        check(frame.height == len(expected), f"{name}: row mismatch.")
        check(frame["order_id"].to_list() == expected_ids, f"{name}: identifier bytes changed after decoding.")
        check(frame["amount"].to_list() == expected_amounts, f"{name}: amount mismatch.")
        check(all(frame[col].null_count() == 0 for col in frame.columns), f"{name}: unexpected null.")
        observed(name, frame)

    ignored = pl.read_csv(late, ignore_errors=True)
    check(ignored.schema == {"order_id": pl.Int64, "amount": pl.Int64}, "ignore_errors schema mismatch.")
    check(ignored["amount"].to_list() == list(range(1, 101)) + [None], "ignore_errors amount values mismatch.")
    check(ignored["order_id"].null_count() == 0, "ignore_errors unexpected ID null.")
    check(ignored.height == 101 and ignored["amount"].null_count() == 1 and ignored["amount"][-1] is None,
          "ignore_errors: expected one null, not preserved text.")
    check(ignored["order_id"].to_list() == list(range(1, 102)), "ignore_errors ID mismatch.")
    observed("ignore_errors", ignored)

    # A non-strict cast identifies the invalid value; it does not authorize import.
    text = pl.read_csv(late, infer_schema=False)
    parsed = text["amount"].cast(pl.Int64, strict=False)
    rejected = text.filter(parsed.is_null())
    check(rejected["order_id"].to_list() == ["0101"] and rejected["amount"].to_list() == ["unknown"],
          "Expected the original late record to be identifiable.")
    check(late == original, "Source bytes changed.")
    print(json.dumps({"polars": pl.__version__, "python": platform.python_version(), "platform": platform.system(),
                      "cases": results, "rejected_original": rejected.to_dicts(),
                      "source_unchanged": True, "real_import_performed": False}, indent=2))


if __name__ == "__main__":
    main()
