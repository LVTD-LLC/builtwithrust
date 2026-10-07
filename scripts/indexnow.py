"""Submit sitemap changes to IndexNow; advance the snapshot only after acceptance."""
import argparse
import json
from pathlib import Path
from urllib.parse import urlsplit
from urllib.request import Request, urlopen
import xml.etree.ElementTree as ET

SITE = "https://builtwithrust.com"
ENDPOINT = "https://api.indexnow.org/indexnow"
NS = {"s": "http://www.sitemaps.org/schemas/sitemap/0.9"}
KEY = (Path(__file__).resolve().parents[1] / "indexnow-key.txt").read_text().strip()


def fetch(url, data=None):
    headers = {"User-Agent": "BuiltWithRust-IndexNow/1.0", "Cache-Control": "no-cache"}
    if data is not None:
        headers["Content-Type"] = "application/json; charset=utf-8"
    with urlopen(Request(url, data=data, headers=headers), timeout=30) as response:
        return response.status, response.read()


def validate_url(url):
    parsed = urlsplit(url)
    if (parsed.scheme != "https" or parsed.netloc != "builtwithrust.com"
            or parsed.query or parsed.fragment):
        raise ValueError(f"Non-canonical sitemap URL: {url}")


def parse_sitemap(body):
    root = ET.fromstring(body)
    if root.tag != f"{{{NS['s']}}}urlset":
        raise ValueError("Expected a sitemap URL set")
    result = {}
    for entry in root.findall("s:url", NS):
        url = entry.findtext("s:loc", default="", namespaces=NS)
        validate_url(url)
        result[url] = entry.findtext("s:lastmod", default="", namespaces=NS)
    if SITE + "/" not in result:
        raise ValueError("Incomplete sitemap: homepage missing")
    return result


def changed_urls(previous, current, force=False):
    for url in previous:
        validate_url(url)
    changed = {url for url in current if force or url not in previous or current[url] != previous[url]}
    changed.update(previous.keys() - current.keys())
    if changed:
        # Listing edits also change directory/category pages and related-project cards.
        # The catalog is small; refresh all public sitemap pages on actual changes.
        changed.update(current)
    return sorted(changed)


def run(snapshot, force=False):
    _, body = fetch(SITE + "/sitemap.xml")
    current = parse_sitemap(body)
    previous = json.loads(snapshot.read_text()) if snapshot.exists() else {}
    urls = changed_urls(previous, current, force)
    if urls:
        key_url = f"{SITE}/{KEY}.txt"
        status, body = fetch(key_url)
        if status != 200 or body.decode().strip() != KEY:
            raise ValueError("Live IndexNow ownership file does not match")
        for start in range(0, len(urls), 10000):
            batch = urls[start:start + 10000]
            payload = {"host": "builtwithrust.com", "key": KEY, "keyLocation": key_url, "urlList": batch}
            status, _ = fetch(ENDPOINT, json.dumps(payload).encode())
            if status not in (200, 202):
                raise RuntimeError(f"IndexNow rejected submission: HTTP {status}")
            print(f"IndexNow HTTP {status}: {len(batch)} URLs received (not proof of indexing)")
    else:
        print("No sitemap changes; no submission")
    snapshot.parent.mkdir(parents=True, exist_ok=True)
    temporary = snapshot.with_suffix(".tmp")
    temporary.write_text(json.dumps(current, sort_keys=True) + "\n")
    temporary.replace(snapshot)
    return urls


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--snapshot", type=Path, default=Path(".indexnow/snapshot.json"))
    parser.add_argument("--force", action="store_true", help="Refresh after a successful deployment")
    args = parser.parse_args()
    run(args.snapshot, args.force)
