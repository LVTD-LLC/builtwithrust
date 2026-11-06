# presenterm export handoff exercise — v1

Built with Rust checked this synthetic deck with presenterm 0.16.1 on Linux,
November 6, 2026. It contains two slides, a pause and a code block with two
highlight groups. It contains no executable snippets, images or private data.
Use an empty working directory and your installed presenterm 0.16.1 binary.
Do not enable snippet execution or validation for this exercise.

The command/config syntax comes from the pinned official references:
- https://github.com/mfontanini/presenterm/blob/v0.16.1/docs/src/features/exports.md
- https://github.com/mfontanini/presenterm/blob/v0.16.1/docs/src/configuration/settings.md#presentation-exports
- https://github.com/mfontanini/presenterm/blob/v0.16.1/docs/src/features/code/highlighting.md#dynamic-highlighting

## HTML: compare pause handling

Run from the directory containing these files:

```sh
presenterm --version
presenterm --config-file export-default.yaml --export-html deck.md --output default.html
presenterm --config-file export-pauses.yaml --export-html deck.md --output pauses.html
```

Open both files in a browser and advance through every slide. In our fixture,
`default.html` has two slides: BEFORE_PAUSE and AFTER_PAUSE appear together.
`pauses.html` has three: before-only, before-plus-after, then code.
In 0.16.1, neither export turns `{1|2}` into separate highlight stages;
both code lines remain syntax-colored. Pause expansion and code-highlight
expansion are different requirements.

## PDF: a separate handoff check

With WeasyPrint and its system dependencies installed, use the same configs:

```sh
presenterm --config-file export-default.yaml --export-pdf deck.md --output default.pdf
presenterm --config-file export-pauses.yaml --export-pdf deck.md --output pauses.pdf
```

Our Linux trial used WeasyPrint 66.0 and produced two and three PDF pages,
respectively. Check the actual files, not just successful exit codes. Verify
required content, wrapping, fonts and the sequence you intend to share.
The 80-column/30-row setting is a terminal grid, not a pixel-size guarantee.

This is a bounded functional example, not a benchmark, image/terminal/projector
compatibility test, or guarantee for your deck. Keep a checked fallback and
retest after changing presenterm, WeasyPrint, fonts or the presentation setup.
