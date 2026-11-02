# Getting Started

Three ways in: a Simurgh on your profile README, the command line, and the library.
Each stands alone. (راهنمای فارسی: [getting-started.fa.md](getting-started.fa.md))

## On Your Profile README

GitHub shows the README of the repository named after your username, such as
`octocat/octocat`, on your profile page. To give it a Simurgh that redraws every
night:

1. Copy [examples/profile-workflow.yml](../examples/profile-workflow.yml) into
   that repository as `.github/workflows/simurgh.yml`.
2. Embed the Simurgh in its `README.md`:

   ```html
   <img src="simurgh/simurgh.gif" width="256" alt="My Simurgh" />
   ```

   The GIF is drawn at exactly twice the Simurgh's pixel size, so `width="256"`
   shows it at native size without blurring.

3. Run the workflow once from the repository's Actions tab, so the first Simurgh
   appears without waiting for midnight.

The workflow runs just after midnight UTC, so each Simurgh shows the day that just
ended, and again whenever the workflow file changes. Its `simurgh` step is this
repository's Action. It collects your public activity with the workflow's
built-in token and renders `simurgh.gif`, `simurgh.png`, and `preview.html` into `simurgh/`.
The Action never commits. The workflow's last step commits `simurgh/simurgh.gif` only
when it changed, so an unchanged Simurgh adds no commit.

| Input        | Default              | Meaning                                                                |
|--------------|----------------------|------------------------------------------------------------------------|
| `login`      | The repository owner | Account whose Simurgh to draw                                          |
| `body`       | `zarrin`             | `zarrin`, `firuzeh`, `lajvard`, `sepid`, `shab`, or `arghavan`         |
| `output-dir` | `simurgh`            | Folder that receives `simurgh.gif`, `simurgh.png`, `preview.html`      |
| `token`      | The built-in token   | Token for reading the account's public data                            |

To change the body, edit `body` in your copy of the workflow; pushing the change
runs the workflow and redraws the Simurgh. If you change `output-dir`, change the
commit step's path and the README image to match. The example uses
`OWNER/REPO@v0.1.0`: replace `OWNER/REPO` with the repository that holds your
copy of this project. The Action finds its own repository and downloads the
release binary with the same version, so reference it at a release tag of that
repository.

GitHub pauses scheduled workflows in a public repository after 60 days without
activity such as a push. A Simurgh that stops changing during a long quiet stretch
adds no commits, so its schedule can pause and does not resume on its own.
Re-enable it from the Actions tab or with `gh workflow enable simurgh.yml`.

## From the Command Line

Install one of two ways. With Rust at or above the `rust-version` in
[Cargo.toml](../Cargo.toml), from a checkout of the repository:

```sh
cargo install --path . --locked
```

(If you publish the crate to crates.io, `cargo install <crate-name> --locked`
works too.) Or download the archive for your platform from the Releases page of
your copy of the repository, a `.tar.gz` for Linux and macOS or a `.zip` for
Windows. Each archive has a `.sha256` file
beside it; check the download with `shasum -a 256 -c` on macOS, `sha256sum -c`
on Linux, or `certutil -hashfile` on Windows, then put `simurgh` on your `PATH`.
macOS may refuse to open a binary it downloaded;
`xattr -d com.apple.quarantine simurgh` clears that flag.

Collection goes through the [GitHub CLI](https://cli.github.com), so install it
and sign in once with `gh auth login`. Then:

```sh
simurgh collect octocat --output snapshot.json
simurgh render snapshot.json --output-dir simurgh --body lajvard
```

Open `simurgh/preview.html` in a browser to see the GIF and the still on GitHub's
light and dark page colors. `--body` takes `zarrin`, the default, `firuzeh`,
`lajvard`, `sepid`, `shab`, or `arghavan`.

`collect` reads only public data: the account's public repositories, the public
repositories it has contributed to, GitHub's color for each language, and its
daily contributions from the account's creation through the last complete UTC
day. It writes the snapshot to standard output, or with `--output` replaces the
file only after collection succeeds. In GitHub Actions, `GH_TOKEN` set to the
workflow's token is enough. `render` draws everything in memory first and
replaces each file through a temporary file, so a failed run keeps the previous
files. Both print errors to standard error and exit 1; invalid usage exits 2.
`simurgh --help` prints the full usage and `simurgh --version` the version.

## As a Library

The same crate is a library. Read its documentation with `cargo doc --open`, or
on docs.rs if you publish it. Add it beside `serde_json`, which reads the
snapshot, from a path or from crates.io:

```sh
cargo add simurgh --path /path/to/your/copy
cargo add serde_json
```

The pipeline runs one way. Read or collect an `AccountSnapshot`, derive the
Simurgh's development, choose the day's behavior, render frames, and encode them:

```rust
use simurgh::behavior::select_behavior;
use simurgh::coat::SimurghBody;
use simurgh::history::AccountSnapshot;
use simurgh::development::derive_simurgh_development;
use simurgh::motion::frame_count;
use simurgh::output::{encode_gif, encode_png};
use simurgh::render::{render_flight_frame, render_simurgh};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let snapshot_json = std::fs::read_to_string("snapshot.json")?;
    let account_snapshot: AccountSnapshot = serde_json::from_str(&snapshot_json)?;
    let simurgh_development = derive_simurgh_development(&account_snapshot)?;
    let simurgh_body = SimurghBody::Lajvard;
    let behavior = select_behavior(&simurgh_development);
    let flight_frames: Vec<_> = (0..frame_count(behavior))
        .map(|frame_index| render_flight_frame(&simurgh_development, simurgh_body, behavior, frame_index))
        .collect();
    std::fs::write("simurgh.gif", encode_gif(&flight_frames)?)?;
    std::fs::write("simurgh.png", encode_png(&render_simurgh(&simurgh_development, simurgh_body))?)?;
    Ok(())
}
```

To collect in-process instead of reading a file, call
`collect::collect_account_snapshot` with `collect::GhCli` and today's UTC date.
Everything after collection is deterministic and performs no I/O, so the same
snapshot always draws the same Simurgh. Breaking changes to the API bump the major
version.

## Publishing Your Own Copy on GitHub

This is the whole path from the files on your computer to a Simurgh on your profile.
Replace `OWNER` with your GitHub username (or organization).

**You need:** a GitHub account, [Git](https://git-scm.com), and optionally the
[GitHub CLI](https://cli.github.com) (`gh`). You do not need Rust to publish.

### 1. Create the repository and push

On github.com choose **New repository**, name it `simurgh`, make it **Public**, and
leave "Add a README", ".gitignore", and "license" all **unchecked** (the project
already has them). Then, in the project folder:

```sh
git init
git add .
git commit -m "Initial commit"
git branch -M main
git remote add origin https://github.com/OWNER/simurgh.git
git push -u origin main
```

With the GitHub CLI, one command does the same:
`gh repo create simurgh --public --source=. --push`.

Commits show the email in your Git settings. To keep your address private, use the
`ID+USERNAME@users.noreply.github.com` address from GitHub's Settings, Emails page:
`git config user.email "ID+USERNAME@users.noreply.github.com"`.

### 2. Check that the workflows run

Open the **Actions** tab. If GitHub asks you to enable workflows, enable them. The
`CI` and `End to end` workflows should finish green. If a run fails, open it and read
the red step; the log says what is wrong.

### 3. Publish the first release

The Action downloads its program from a GitHub Release, so a release must exist before
anyone can use the Action. The tag must equal the version in `Cargo.toml` (`0.1.0`
gives `v0.1.0`), and `CHANGELOG.md` must have a `## [0.1.0]` section; both already do.

```sh
git tag v0.1.0
git push origin v0.1.0
```

Open the Actions tab and wait for the `Release` workflow to turn green. Then the
**Releases** page shows `v0.1.0` with archives for Linux, macOS, and Windows. If the
workflow cannot create the release, open Settings, Actions, General, and under
"Workflow permissions" choose **Read and write permissions**, then re-run it.

### 4. Make the repository easy to find and fork

In the repository's main page, click the gear next to **About** and add a description
and topics, for example `github-profile`, `pixel-art`, `persian`, `simurgh`, `rust`,
`github-actions`. In Settings, General, tick **Template repository** so that anyone
can start a copy with the **Use this template** button.

### 5. Put the Simurgh on your own profile

1. Create a repository named exactly like your username, with a `README.md`. Its
   README is shown on your profile page.
2. In it, create `.github/workflows/simurgh.yml` from
   [`examples/profile-workflow.yml`](../examples/profile-workflow.yml), and change
   `OWNER/REPO@v0.1.0` to `OWNER/simurgh@v0.1.0`. Pick a `body` if you like.
3. Add to its `README.md`:

   ```html
   <img src="simurgh/simurgh.gif" width="256" alt="My Simurgh" />
   ```

4. Open the Actions tab of the profile repository, choose the **Simurgh** workflow,
   and click **Run workflow**. After a minute `simurgh/simurgh.gif` appears and your
   profile shows the bird. From then on it redraws itself every night.

### 6. Release a new version later

After changing anything that users should get:

1. Raise `version` in `Cargo.toml` (for example `0.1.1`).
2. Add a `## [0.1.1]` section at the top of `CHANGELOG.md`, under `## [Unreleased]`.
3. Commit, push, then `git tag v0.1.1` and `git push origin v0.1.1`.
4. Point `uses:` in the profile workflow at `@v0.1.1`.

### 7. Optional: publish to crates.io

Create the repository variable `PUBLISH_TO_CRATES_IO` with the value `true` (Settings,
Secrets and variables, Actions, Variables), and set up trusted publishing for the
crate on crates.io. Without that variable the release workflow never touches crates.io.

### If something goes wrong

| Message or symptom | Cause and fix |
| --- | --- |
| `release v0.1.0 has no asset ...` | The release does not exist yet or is still building. Finish step 3 and wait for it to turn green. |
| `tag ... does not match Cargo.toml version` | The tag and `version` differ. Fix the version, or delete the tag and tag again. |
| `CHANGELOG.md has no section for ...` | Add a `## [x.y.z]` section for the version in the tag. |
| `Resource not accessible by integration` | Give the workflow write access: Settings, Actions, General, Workflow permissions. |
| The profile shows no image | Run the workflow once by hand (step 5), and check that the image path in `README.md` matches `output-dir`. |
| The nightly update stops | GitHub pauses schedules after 60 days without activity in a public repository. Enable the workflow again in the Actions tab. |
