import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from urllib.error import HTTPError
import indexnow as app

HOME = app.SITE + "/"
PROJECT = app.SITE + "/projects/example"
XML = f'<urlset xmlns="{app.NS["s"]}"><url><loc>{HOME}</loc></url></urlset>'.encode()


class IndexNowTests(unittest.TestCase):
    def test_add_update_remove_and_same_day_edit(self):
        before = {HOME: "", PROJECT: "2026-10-07T10:00:00Z"}
        after = {HOME: "", PROJECT: "2026-10-07T11:00:00Z"}
        self.assertIn(PROJECT, app.changed_urls(before, after))
        self.assertIn(PROJECT, app.changed_urls(before, {HOME: ""}))
        self.assertIn(PROJECT, app.changed_urls({HOME: ""}, after))
        self.assertEqual(app.changed_urls(after, after), [])
        self.assertEqual(app.changed_urls(after, after, True), sorted(after))

    def test_reject_foreign_url_and_incomplete_sitemap(self):
        for body in (XML.replace(HOME.encode(), b'https://evil.example/'), b'<html/>',
                     XML.replace(HOME.encode(), (HOME + '?q=x').encode())):
            with self.assertRaises(ValueError):
                app.parse_sitemap(body)

    def test_success_and_unchanged(self):
        for code in (200, 202):
            with tempfile.TemporaryDirectory() as directory:
                snapshot = Path(directory) / 'state.json'
                with patch.object(app, 'fetch', side_effect=[(200, XML), (200, app.KEY.encode()), (code, b'')]) as fetch:
                    self.assertEqual(app.run(snapshot), [HOME])
                    payload = json.loads(fetch.call_args.args[1])
                    self.assertEqual(payload['urlList'], [HOME])
                with patch.object(app, 'fetch', return_value=(200, XML)) as fetch:
                    self.assertEqual(app.run(snapshot), [])
                    self.assertEqual(fetch.call_count, 1)

    def test_failure_preserves_snapshot(self):
        for failure in (HTTPError(app.ENDPOINT, 429, 'rate limit', {}, None), TimeoutError()):
            with tempfile.TemporaryDirectory() as directory:
                snapshot = Path(directory) / 'state.json'
                original = json.dumps({HOME: '', PROJECT: 'old'})
                snapshot.write_text(original)
                with patch.object(app, 'fetch', side_effect=[(200, XML), (200, app.KEY.encode()), failure]):
                    with self.assertRaises((HTTPError, TimeoutError)):
                        app.run(snapshot)
                self.assertEqual(snapshot.read_text(), original)

    def test_bad_key_never_submits_or_saves(self):
        with tempfile.TemporaryDirectory() as directory:
            snapshot = Path(directory) / 'state.json'
            with patch.object(app, 'fetch', side_effect=[(200, XML), (200, b'wrong')]) as fetch:
                with self.assertRaises(ValueError):
                    app.run(snapshot)
                self.assertEqual(fetch.call_count, 2)
                self.assertFalse(snapshot.exists())


if __name__ == '__main__':
    unittest.main()
