# AGENTS.md — working in this repo

Read this before cutting a release. The app, its releases, and its download
page live in two repos, and the in-app update check trusts whichever release
GitHub calls newest.

## The two repos

| Repo | Path on this machine | What it is |
| --- | --- | --- |
| **Dr.Player** (you are here) | `B:\_Git\DEV\Babar-Meet\Dr.Player` | The Rust video player. Application source, branch `main`. |
| **dist-drplayer** | `B:\_Git\DEV\Babar-Meet\dist-drplayer` | Download page only, live at <https://drplayer.pages.dev>. |

## Release checklist

1. Bump `version` in `Cargo.toml` to `x.y.z`.
2. `cargo build --release`, so the binary carries the new number, then build
   the installer with the same number:
   `makensis "-DAPP_VERSION=x.y.z" installer\DrPlayer.nsi`.
   The order matters (see Release order in `README.md`): skipping the rebuild
   ships a stale binary that prompts for the version it already runs.
3. Publish in **`Babar-Meet/Dr.Player` only**, tag **`vx.y.z`**, with exactly
   these two assets:
   - `DrPlayer-Setup.exe` (from `dist-installer\`)
   - `dr-player-x.y.z.exe` (the release binary, renamed with the version)

   ```powershell
   $env:GH_PROMPT_DISABLED = "1"
   Copy-Item "target\release\dr-player.exe" "$env:TEMP\dr-player-3.2.0.exe" -Force
   gh release create v3.2.0 "dist-installer\DrPlayer-Setup.exe" "$env:TEMP\dr-player-3.2.0.exe" `
     --repo Babar-Meet/Dr.Player --title "Dr.Player v3.2.0" --notes "## Dr.Player v3.2.0"
   ```

   Confirm the version with the owner before running any `gh release create`.
   It is a public deploy.
4. Never backfill: do not publish an older version after a newer one. The
   update check reads `releases/latest` (`RELEASES_API` in `src/main.rs`), and
   GitHub resolves that to the most recently published release, so a backfilled
   old tag becomes "latest" and silences update prompts.
5. Refresh the dist-drplayer static bits: the hero Standalone href, the hero
   overline version, and the version line in `README.md`. Redeploy from the
   dist-drplayer dir:

   ```
   npx wrangler pages deploy . --project-name drplayer --branch main
   ```

   Then click both hero links (Setup, Standalone) on the live page and confirm
   they download.
