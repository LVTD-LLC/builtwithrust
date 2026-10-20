# Public project journey (schema 1)

The existing PostHog `$autocapture` event receives these properties from HTML
[`data-ph-capture-attribute-*` attributes](https://posthog.com/docs/product-analytics/autocapture#capturing-additional-properties-in-autocapture-events).
No separate event, listener, redirect or request-time network call is added.

- `bwr-action`: `project_open` on shared project cards, `website_visit` on the
  detail-page website button, or `repository_visit` on its repository button.
- `bwr-project`: the already-public catalog slug, never an internal database ID.
- `bwr-surface`: `project_card` or `project_detail`.
- `bwr-schema`: `1`.

Filter `$autocapture` to `$event_type = click`, `$host = builtwithrust.com` and
`bwr-schema = 1`. Use the existing `$pathname` to distinguish homepage, category,
search and related-project card placements. These labels cover shared cards and
the two detail-page buttons, not every inline guide/source link.

Exclude QA and known bots, and retain the exclusions with each analysis. Missing
labels on historical or blocked-analytics visits are unobserved, not zero intent.
Neither the remaining traffic nor a click is proof of a human, an external page
load, installation, payment or downstream product adoption. Avoid comparing
pre-label text-derived click estimates as if they used the same definition.

After deployment, validate the actual SDK payload in a disposable browser with
ingestion intercepted. Do not send synthetic QA events into the production
baseline. Verify ordinary and keyboard navigation still work with analytics
blocked. The labels themselves contain only public slugs and fixed enums; they
do not add search text, form values, destination queries or user identifiers.
Existing PostHog properties and privacy settings are otherwise unchanged.
