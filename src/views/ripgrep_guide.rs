//! A controlled file-selection exercise for recursive text search.
use maud::{Markup, PreEscaped, html};

pub fn body(site_url: &str) -> Markup {
    let title = "ripgrep: search hidden and ignored files";
    let url = format!("{site_url}/guides/ripgrep-hidden-ignored-files");
    let steps = [
        ("fixture", "Make a search tree with known contents"),
        ("profiles", "Change one search rule at a time"),
        ("diagnose", "Find out why a path went missing"),
    ];
    let schema = serde_json::json!({
        "@context": "https://schema.org", "@graph": [
            {"@type":"Article", "@id":format!("{url}#article"), "url":url, "headline":title,
             "datePublished":"2026-11-24", "dateModified":"2026-11-24",
             "author":{"@type":"Organization","name":"Built with Rust editorial","url":site_url}},
            {"@type":"HowTo","name":title,"step":steps.iter().map(|(id,name)| serde_json::json!({
                "@type":"HowToStep","name":name,"url":format!("{url}#{id}")
            })).collect::<Vec<_>>()},
            {"@type":"BreadcrumbList","itemListElement":[
                {"@type":"ListItem","position":1,"name":"Projects","item":site_url},
                {"@type":"ListItem","position":2,"name":"Guides","item":format!("{site_url}/guides")},
                {"@type":"ListItem","position":3,"name":title,"item":url}
            ]}
        ]
    });
    let fixture = r#"lab=$(mktemp -d) || exit 1
mkdir -p "$lab/home" "$lab/repo/tree/.settings" "$lab/repo/tree/build"
labrun() {
    env HOME="$lab/home" XDG_CONFIG_HOME="$lab/home/.config" GIT_CONFIG_NOSYSTEM=1 "$@"
}
labrun git -C "$lab/repo" init -q
cd "$lab/repo/tree" || exit 1
printf 'build/\n' > .gitignore
for path in normal.txt .settings/editor.txt build/output.txt build/.cache; do
    printf 'LOOKUP_MARKER\n' > "$path"
done
printf '\000LOOKUP_MARKER\n' > sample.dat
matches() { labrun rg --no-config --no-mmap -lF "$@" LOOKUP_MARKER .; }
matches"#;
    let profiles = [
        ("matches", "normal.txt"),
        ("matches --hidden", ".settings/editor.txt, normal.txt"),
        ("matches --no-ignore", "build/output.txt, normal.txt"),
        ("matches --hidden --no-ignore", ".settings/editor.txt, build/.cache, build/output.txt, normal.txt"),
        ("matches --binary", "normal.txt, sample.dat"),
        (
            "matches --hidden --no-ignore --binary",
            ".settings/editor.txt, build/.cache, build/output.txt, normal.txt, sample.dat",
        ),
    ];
    html! {
        article class="python-guide page" {
            nav class="small muted" aria-label="Breadcrumb" { a href="/" { "Projects" } " / " a href="/guides" { "Guides" } " / " (title) }
            header {
                h1 { (title) }
                p class="small muted" { "By Built with Rust editorial · Published " time datetime="2026-11-24" { "November 24, 2026" } }
            }
            p class="guide-lead" {
                "To search hidden files with ripgrep, use " code { "rg --hidden -F 'needle' ." }
                ". This includes dotfiles without turning off ignore rules. To include ignored paths but keep hidden paths out, use "
                code { "--no-ignore" } " instead. A dotfile inside an ignored directory can need both flags. If you already know the exact file, pass its path rather than broadening the entire search."
            }
            p {
                "Suppose your editor finds a setting when you open a file, but its project search returns nothing. Did the search skip the file, ignore its directory, or open it and fail to find the text? A list of matching lines cannot tell you which. Below is a five-file test you can use to pick a command for a repository, or check whether an editor's search behaves like the command you chose."
            }
            p {
                "This is a test of ripgrep, the Rust text-search tool invoked as " code { "rg" }
                ". If you remember part of the filename rather than something written inside it, start with the "
                a href="/projects/fd" { "fd file finder" } " instead. The "
                a href="/guides/rust-terminal-tools" { "Rust terminal tools guide" } " distinguishes file browsing from terminal windows, shell environments and history search; the "
                a href="/categories/developer-tools" { "developer tools directory" } " lists more projects for working with a codebase."
            }
            nav class="guide-toc" aria-label="On this page" { ol {
                @for (id, name) in steps { li { a href=(format!("#{id}")) { (name) } } }
                li { a href="#handoff" { "Record a search contract for your team" } }
            } }
            h2 id="fixture" { "Make a search tree with known contents" }
            p {
                "Run the following in a fresh Bash session on Linux with Git and ripgrep installed. The labrun function uses an empty home and configuration directory only for the commands it runs. That keeps your normal global Git ignore file out of the comparison without changing your shell configuration. The small test repository stays under the temporary path printed by "
                code { "echo \"$lab\"" } ". We ran the commands on Linux with ripgrep 15.2.0. We did not compare runtimes, search an actual team's repository, or test an editor integration."
            }
            pre { code { (fixture) } }
            p {
                "There are five copies of the same literal marker. One is in a plain text file. One sits below a hidden directory, despite its own filename being visible. A third sits below an ignored directory, and a fourth has both an ignored parent and a hidden name. The fifth file starts with a zero byte immediately before the marker. Its extension is incidental: the byte, not the spelling of sample.dat, matters to this test."
            }
            p {
                "The command searches the child directory called tree, not its parent. That puts the fixture inside a real Git working tree, where ripgrep ordinarily honors .gitignore, without adding Git's own hidden database to the search. There is no need for a commit. The function supplies an explicit search root, calls for filenames rather than matching lines, and uses a fixed string so regular-expression syntax cannot change what matches. It also disables ripgrep's configuration file and memory mapping to keep the comparison consistent."
            }
            h2 id="profiles" { "Change one search rule at a time" }
            p {
                "Run each command in the left column in the shell where you created the fixture. The right column gives the entire set of files we saw, with the leading ./ removed. Compare names, not print order or just the total: three commands found two files each, and each found a different companion to normal.txt."
            }
            div class="task-table-wrap" tabindex="0" role="region" aria-label="Matching paths by search command, scroll horizontally" { table class="task-table" {
                caption { "Matching files in the five-marker exercise · ripgrep 15.2.0, Linux" }
                thead { tr { th scope="col" { "Command" } th scope="col" { "Matching paths (set, not order)" } } }
                tbody { @for (cmd, found) in profiles { tr { th scope="row" { code { (cmd) } } td { (found) } } } }
            } }
            p {
                "Read the two middle text cases as a decision about scope. If the job is to find editor settings in dot-directories without trawling generated output, the hidden-only row gives the desired result. If you need to inspect ordinary build artifacts while keeping dot-directories out, the no-ignore row is a better fit. Neither row admits build/.cache. That file serves as the control for the intersection: only when both restrictions are lifted does it enter the text results."
            }
            p {
                "The last two rows change a different decision. A zero byte appears before the needle in sample.dat, so the normal recursive search does not report it. With binary search enabled, ripgrep can continue past the zero byte to find the marker; with filename-only output, nothing from inside that file goes to the terminal. This does not mean every binary file contains its human-readable text verbatim. Looking for a sentence in a compressed document is a different task. Nor should you infer that every binary file is skipped before a single match can appear: where ripgrep first sees a zero byte matters."
            }
            p {
                "For a one-off search, write out the individual flags. The shorthand " code { "-uu" }
                " lifts ignore and hidden filtering together; it is not a shorter spelling of hidden-only. Likewise, " code { "-uuu" }
                " enables binary searching as well. Neither is a promise that ripgrep will follow symbolic links. The "
                a href="https://github.com/BurntSushi/ripgrep/blob/15.2.0/GUIDE.md#automatic-filtering" { "upstream filtering guide" }
                " describes the switches separately, including the link-following option. The "
                a href="https://github.com/BurntSushi/ripgrep/blob/15.2.0/GUIDE.md#binary-data" { "binary-data notes" }
                " explain why searching for the presence of a match is different from forcing every input to print as text."
            }
            h2 id="diagnose" { "Find out why a path went missing" }
            p {
                "Start with a path you believe should be present. Ask the directory walker for candidate names before asking the matcher for text. In the fixture, the first command below lists normal.txt and sample.dat. It does not list the editor configuration. That gives you two different explanations to investigate, even though both the configuration and the data file were absent from the original match list."
            }
            pre { code { "labrun rg --no-config --files .\nlabrun rg --no-config --debug --files .\nlabrun git check-ignore -v -- build/output.txt\nlabrun rg --no-config --no-mmap -lF LOOKUP_MARKER build/.cache" } }
            p {
                "For a path missing from the candidate list, read the debug output for that path or its parent directory. In the trial, ripgrep identified .settings as hidden and attributed the decision about build to the build/ rule in .gitignore. Search from the same working directory as the original command. Passing an individual file changes the question: the last command above found the marker in build/.cache without either filtering flag. It shows the file is readable and the marker matches, not that the default walk reaches it."
            }
            p {
                "Use Git's explanation only for the part Git knows about. The "
                a href="https://git-scm.com/docs/git-check-ignore" { "git check-ignore manual" }
                " documents verbose output, which includes the source file, line and matching pattern. The example uses an untracked file. No output from Git is not an all-clear from ripgrep: a hidden parent can still prevent traversal, and ripgrep can use rules from .ignore and .rgignore that Git does not consult. For example, a team can keep generated files out of Git but explicitly admit a particular output to ripgrep searches with a project search rule. In that situation, a Git explanation and a ripgrep explanation can both be correct and still disagree about the same path. Check the walk rather than editing .gitignore until a result appears."
            }
            p {
                "For a path that appears in the candidate list but not in the matching list, first confirm the literal you are searching for. In this trial all five copies are deliberately identical; in real code the spelling, capitalization or encoding may differ. Try a filename-only binary search on a small, known sample when the bytes are in doubt. Listing a path with --files proves only that the walker would offer the path, not that the file passed a content test. If the command reports a read error, resolve it before interpreting a missing match."
            }
            h2 id="handoff" { "Record a search contract for your team" }
            p {
                "When a project-search shortcut and a terminal command disagree, give a colleague a positive and a negative example. For a settings search, write down the working directory, the marker, the path that must appear (.settings/editor.txt here) and a plausible neighbor that must not (build/output.txt). Record whether you searched a directory or supplied the filename. Asking only whether the shortcut finds the setting misses the regression where someone fixes it by searching every generated artifact too."
            }
            p {
                "For a repeatable project command, keep the plain file as a positive control. If normal.txt disappears, investigate the search root and the pattern before drawing conclusions about hidden paths. Keep the hidden-inside-ignored example if someone relies on both boundaries. If your workflow never searches binary artifacts, leave binary searching off; the data file is useful precisely because its absence is expected. Change the expected set first when the job changes, then pick the narrowest command that produces that set."
            }
        }
        script type="application/ld+json" { (PreEscaped(schema.to_string().replace('<', "\\u003c"))) }
    }
}
