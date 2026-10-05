## Research: Small Test Video Files for Cross-Platform Video Player
*Date: 2026-06-23*

### Summary

Found excellent sources for small test videos across all 5 requested formats (MP4, WebM, AVI, MOV, MKV). Multiple repositories offer CC0/public-domain files, and most of them are small: of the 13 files committed to `TESTING_videos/`, 10 are under 1.5 MB and three are not: `test1-matroskaconformance.mkv` at 22,792 KB, `sample_1280x720.mov` at 17,027 KB, and `elephants_dream_1024x576_30s.mp4` at 15,358 KB. Below is the curated list with direct download URLs.

This document was re-checked against the committed files on 2026-10-05 with `ffprobe` and HTTP HEAD. Where a URL is dead, a size is wrong or a codec is mislabelled, it now says so. Sizes, codecs and container facts are re-checkable by anyone from the committed bytes. Everything about the network, the URL verdicts, the redirect behaviour and the checksums of re-fetched copies is a record of that one dated pass, not something this repository can reproduce: nothing here stores a log, a header dump or a fetched copy.

### Known problem: `sample-1mb_alt.mp4` is not a video

`sample-1mb_alt.mp4` is committed to git (123,943 bytes) and is **not playable media**. Its first bytes are `<!doctype html><html lang="en-US" prefix="og: https://ogp.me/ns`, an HTML landing page, and `ffprobe` reports `moov atom not found` / `Invalid data found when processing input`. A tester who opens it gets "Error loading video".

The intended source is unknown. `inventory.csv` used to record `filesfortesting.com` as the origin, and it now records `unknown`, with a note that the intended source is unknown. That cell was emptied because the bytes on disk are an HTML share page rather than the media, so they support no attribution at all. The small MP4 the table below recommends for this slot (`sample-1mb.mp4`) is a dead URL. The file has been left in place; removing it is the owner's decision, not this document's.

### Playability of the committed files

"Confirmed to play" in `inventory.csv` means exactly this: the container and codec combination appears in the project's own recorded test run, which was **played on Windows 11 with WebView2 Runtime 154**. `README.md` names those four combinations twice, under Known Limitations and in the legend under the formats table: "H.264 in MP4, H.264 in MOV, VP8 and VP9 in WebM and H.264 in MKV were each played on Windows 11 with WebView2 Runtime 154". Nothing here was tested on macOS or Linux. The same legend states the other columns from documentation rather than a test run: "every macOS and Linux cell, come from each platform's documented engine support and were not tested".

Confirmed to play on Windows:

| File | Container | Video | Audio |
|------|-----------|-------|-------|
| `flower.mp4` | MP4 | H.264 High | AAC LC |
| `rabbit320.mp4` | MP4 | H.264 Constrained Baseline | AAC LC |
| `elephants_dream_1024x576_30s.mp4` | MP4 | H.264 Constrained Baseline | AAC LC |
| `sample_1280x720.mov` | MOV | H.264 High | none |
| `flower.webm` | WebM | VP8 | Vorbis |
| `rabbit320.webm` | WebM | VP8 | Vorbis |
| `test-vp9.webm` | WebM | VP9 Profile 0 | none |
| `sample_640x360.mkv` | MKV | H.264 High | none |

Refused, with "Error loading video":

| File | Why |
|------|-----|
| `sample-5s.avi`, `sample-15s.avi`, `sample_640x360.avi` | AVI is turned down by both the Windows and macOS engines. `README.md` says so twice: the formats table's summary sentence lists `.avi` among the types that "play on neither Windows nor macOS", and Known Limitations says `.avi`, `.wmv`, `.flv`, `.mpeg` and `.ogv` "are absent because neither engine plays them at all". All three hold MPEG-4 Part 2. |
| `test1-matroskaconformance.mkv` | The Matroska container plays; the MPEG-4 Part 2 (msmpeg4v2) video inside it is refused. `README.md`, under the formats table: "an `.mkv` holding MPEG-4 Part 2 (DivX or Xvid) is refused even though MKV itself is fine". This is the only MKV here that cannot play. |
| `sample-1mb_alt.mp4` | Not media at all; see above. |

The three AVI files and the conformance MKV are still useful: they are the fixtures for the refusal path, which is a path a tester needs to exercise. They are not fixtures for playback.

---

### RECOMMENDED FILES TO DOWNLOAD

Every "Live", "DEAD (404 on 2026-10-05)", "byte-identical", "SHA256-matched" and "429" below is one HTTP HEAD or GET made on 2026-10-05, recorded as a dated check that cannot be repeated from this repository. Sizes, durations and codecs are measurements of the committed bytes and are repeatable.

#### MP4 (H.264/AAC - universal format)
| File | URL | Size | Duration | License | Status |
|------|-----|------|----------|---------|--------|
| sample-1mb.mp4 | https://truefilesize.com/files/mp4/sample-1mb.mp4 | 1 MB | ~4s | CC0 / Public Domain | DEAD (404 on 2026-10-05). No replacement chosen; see "Decisions left to the owner" below. |
| sample-5mb.mp4 | https://truefilesize.com/files/mp4/sample-5mb.mp4 | 5 MB | ~20s | CC0 / Public Domain | DEAD (404 on 2026-10-05). |
| flower.mp4 (MDN) | https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4 | 1,102 KB | ~5s | CC0 | Live; byte-identical to the committed copy. |
| rabbit320.mp4 (MDN) | https://github.com/mdn/learning-area/raw/main/html/multimedia-and-embedding/video-and-audio-content/rabbit320.mp4 | 815 KB | 7.8s | CC0 | Live; byte-identical to the committed copy. |

#### WebM (open web format)

Measured codecs, because the codecs are not what the section heading used to claim: two of the three files are VP8 with Vorbis, not VP9 with Opus, and the one VP9 file has no audio stream at all. There is no Opus coverage anywhere in this directory.

| File | URL | Size | Duration | Measured codecs | License | Status |
|------|-----|------|----------|------------------|---------|--------|
| flower.webm (MDN) | https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.webm | 541 KB | ~5s | VP8 + Vorbis (2ch) | CC0 | Live; SHA256-matched against the committed copy. |
| rabbit320.webm (MDN) | https://github.com/mdn/learning-area/raw/main/html/multimedia-and-embedding/video-and-audio-content/rabbit320.webm | 323 KB | 7.8s | VP8 + Vorbis (2ch) | CC0 | Live; byte-identical to the committed copy. |
| test-vp9.webm (WPT) | https://raw.githubusercontent.com/web-platform-tests/wpt/master/media-source/webm/test-vp9.webm | 43 KB | ~2s | VP9 Profile 0, no audio stream | BSD-3 / Public Domain | Live; byte-identical to the committed copy. |

#### AVI (Legacy container - MSRIFF)

**AVI is refused by both the Windows and macOS engines**, so these are fixtures for the refusal path, not the playback path. The three committed ones all hold MPEG-4 Part 2 (Simple Profile), confirmed with `ffprobe`. The first two are saved under different names by the download script: `avi_5s_sample_file_318KB.avi` becomes `sample-5s.avi` and `avi_15s_sample_file_927KB.avi` becomes `sample-15s.avi`.

| File | URL | Size | Duration | License | Status |
|------|-----|------|----------|---------|--------|
| avi_5s_sample_file_318KB.avi | https://samplefile.com/samples/download/video/avi/avi_5s_sample_file_318KB.avi/ | 318 KB | 5s | Public domain | Live; byte-identical to the committed `sample-5s.avi`. |
| avi_15s_sample_file_927KB.avi | https://samplefile.com/samples/download/video/avi/avi_15s_sample_file_927KB.avi/ | 927 KB | 15s | Public domain | Live; byte-identical to the committed `sample-15s.avi`. |
| sample_640x360.avi | https://filesamples.com/samples/video/avi/sample_640x360.avi | 570 KB | 13s | Royalty-free | Live; SHA256-matched against the committed copy. |
| sample_1280x720.avi | https://filesamples.com/samples/video/avi/sample_1280x720.avi | 4.2 MB | 28s | Royalty-free | Live. Not committed to this directory. |

#### MOV (QuickTime - Apple ecosystem)
| File | URL | Size | Duration | License | Status |
|------|-----|------|----------|---------|--------|
| sample-5mb.mov | https://truefilesize.com/files/mov/sample-5mb.mov | 5 MB | ~15s | CC0 / Public Domain | DEAD (404 on 2026-10-05). |
| sample_1280x720.mov | https://testfileorg.netwet.net/Sample%20Video%202/sample_1280x720.mov | 17,027 KB | ~28s | Free to use | Live. `testfileorg.netwet.net` is the host that actually served this file; `testfile.org` does not. |
| roundhay.mov | https://web.archive.org/web/20070316205741if_/http://www.nmpft.org.uk:80/insight/info/roundhay.mov | ~1,275 KB | 2.1s | Public Domain (1888) | Live. Not committed to this directory. |

#### MKV (Matroska - flexible container)

**`sample_640x360.mkv` is H.264 and plays. `test1.mkv` is not H.264 and is refused.** The Matroska container itself is fine on a current Evergreen Runtime; the MPEG-4 Part 2 (msmpeg4v2) video inside `test1.mkv` is what the engine turns down, so this file exercises the refusal path, not the playback path. The download script saves it as `test1-matroskaconformance.mkv`. It is also the largest file here, at 22,792 KB and 87.3 s, not the ~5 MB / ~30 s this table originally claimed.

| File | URL | Size | Duration | Measured codecs | License | Status |
|------|-----|------|----------|------------------|---------|--------|
| sample-1mb.mkv | https://truefilesize.com/files/mkv/sample-1mb.mkv | 1 MB | ~4s | unknown, not fetched | CC0 / Public Domain | DEAD (404 on 2026-10-05). |
| sample_640x360.mkv | https://filesamples.com/samples/video/mkv/sample_640x360.mkv | 560 KB | 13s | H.264 High, no audio | Royalty-free | Live; SHA256-matched against the committed copy. Plays on Windows. |
| test1.mkv (Matroska test) | https://github.com/ietf-wg-cellar/matroska-test-files/raw/master/test_files/test1.mkv | 22,792 KB | 87.3s | MPEG-4 Part 2 (msmpeg4v2) + MP3 | CC-BY 3.0 (Big Buck Bunny) | Live; byte-identical to the committed `test1-matroskaconformance.mkv`. **Refused.** |

#### OGG/Theora (Open format - for backup testing)

All three URLs are DEAD (404 on 2026-10-05) and none of the three files is committed to this directory, so nothing here has been verified as playable. Theora in an Ogg is turned down by both the Windows and macOS engines in any case. The download script would save these as `320x240_test_pattern.ogv` and `big_buck_bunny_ecu.ogv`, under different names from the ones below.

| File | URL | Size | Duration | License | Status |
|------|-----|------|----------|---------|--------|
| 320x240.ogv | https://upload.wikimedia.org/wikipedia/commons/3/33/320x240.ogv | 315 KB | 4.4s | Public Domain | DEAD (404 on 2026-10-05). Saved by the script as `320x240_test_pattern.ogv`. |
| Big Buck Bunny 8s clip | https://upload.wikimedia.org/wikipedia/commons/b/b0/Big_Buck_Bunny_8_seconds_bird_clip.ogv | 1.68 MB | 7.3s | CC-BY 3.0 | DEAD (404 on 2026-10-05). Not fetched by the script. |
| Big buck bunny ecu.ogv | https://upload.wikimedia.org/wikipedia/commons/5/58/Big_buck_bunny_ecu.ogv | 279 KB | 3.3s | CC-BY 3.0 | DEAD (404 on 2026-10-05). Saved by the script as `big_buck_bunny_ecu.ogv`. |

`upload.wikimedia.org` returned 429 once before a later plain 404, so rate-limiting cannot be fully excluded for that host. A replacement, if one is wanted, is the owner's decision.

---

### BEST SOURCE SITES

| Site | Formats | License | Notes |
|------|---------|---------|-------|
| truefilesize.com/video/ | MP4, MKV, MOV, AVI, WebM | CC0 / Public Domain | All four of its URLs above are 404 as of 2026-10-05. A clean run of `download_test_videos.ps1` fails on them. |
| file-examples.com | MP4, AVI, MOV, OGG, WEBM, WMV | Free | Size-per-resolution table. Direct links. |
| filesfortesting.com | MP4, WEBM, MOV, MKV, AVI, HEVC, HLS | CC0 / Public Domain | Most comprehensive. No CAPTCHA. |
| samplefile.com | AVI | Public Domain | Verified SHA256 checksums. |
| examplefiles.org | MP4, AVI, MOV, WebM | Royalty-free | Same content across formats for comparison. |
| filesamples.com | AVI, MKV | Royalty-free | Resolution metadata in filenames (`640x360`, `1280x720`). |
| Wikimedia Commons | OGV, WebM, MP4 | CC-BY / Public Domain | Huge real-content clip library. |
| MDN Web Docs | WebM, MP4 | CC0 | Official Mozilla samples. Very reliable URLs. |
| Matroska GitHub | MKV | CC-BY 3.0 | Official Matroska conformance test suite. |
| Pixabay Videos | MP4 | Pixabay License | The site's own size claim, not verified here. No download from it is recommended above. Short clips available. |
| Mixkit | MP4 | Mixkit Free License | High-quality free stock footage. |

---

### GitHub Repositories with Test Video Files

1. **bower-media-samples/big-buck-bunny-480p-30s** - 30s 480p Big Buck Bunny snippet (CC-BY 3.0)
   - https://github.com/bower-media-samples/big-buck-bunny-480p-30s

2. **ietf-wg-cellar/matroska-test-files** - Official MKV conformance test suite (CC-BY 3.0). This is the current upstream; the older `Matroska-Org/matroska-test-files` path still redirects to it, so the old URLs resolve but name a repository that has moved.
   - https://github.com/ietf-wg-cellar/matroska-test-files

3. **web-platform-tests/wpt** - Browser vendor test suite with media files (BSD-3)
   - https://github.com/web-platform-tests/wpt/tree/master/media-source/webm

4. **webmproject/libwebm** - WebM project test data
   - https://github.com/webmproject/libwebm/tree/master/testing/testdata

5. **mdn/learning-area** - MDN tutorial media files (CC0)
   - https://github.com/mdn/learning-area/tree/main/html/multimedia-and-embedding/video-and-audio-content

6. **google/shaka-packager** - Shaka packager test media (BSD)
   - https://github.com/google/shaka-packager/tree/master/packager/media/test/data

7. **chromium/media/test/data** - Chrome/Chromium media test suite
   - https://chromium.googlesource.com/chromium/+/refs/heads/trunk/media/test/data/

8. **joshuatz/video-test-file-links** - Curated collection of links to test video sources
   - https://github.com/joshuatz/video-test-file-links

9. **Jellyfin Test Videos** (repo.jellyfin.org) - SDR/HDR/Dolby Vision test files (CC-BY-SA)
   - https://repo.jellyfin.org/files/test-videos/

---

### License Compatibility Notes

- **CC0 / Public Domain**: Safest. Can use for any purpose, no attribution needed.
- **CC-BY 3.0/4.0**: Free to use with attribution. Big Buck Bunny, Blender Foundation films.
- **CC-BY-SA**: Share-alike - derivative works must use same license.
- **Mixkit Free License**: Commercial use allowed, no attribution required.
- **Pixabay License**: Free for commercial use, no attribution required.
- **Sample file sites (truefilesize, file-examples, etc.)**: Typically use CC0 or state "free to use."

**Recommendation**: Download any CC0 or Public Domain files for unrestricted use. For Big Buck Bunny clips (CC-BY 3.0), keep a LICENSE.txt noting the Blender Foundation copyright.

### OUTSTANDING: CC-BY attribution is required and has not been done

This is an unresolved licence obligation, not a note about style, and it is the one thing in this document that is not optional. Two CC-BY works are committed to git and **no attribution file exists**: there is no `TESTING_videos/LICENSE.txt`. `inventory.csv` records the copyright holder and the licence in its `License` column, which is the catalogue's own bookkeeping, not an attribution notice, and CC-BY asks for the attribution to accompany the work. Nothing in this repository states it as a notice beside the media. This document's own recommendation above ("keep a LICENSE.txt noting the Blender Foundation copyright") is the step that was skipped.

| Committed file | Work | Copyright holder | Licence |
|----------------|------|------------------|---------|
| `elephants_dream_1024x576_30s.mp4` | Elephant's Dream | Blender Foundation | CC-BY 3.0 |
| `test1-matroskaconformance.mkv` | Big Buck Bunny (conformance clip) | Blender Foundation | CC-BY 3.0 |

The `matroska-test-files` project states that both works are CC-BY and supplies the exact attribution text. Neither work is CC0 or public domain.

**The outstanding action is to create `TESTING_videos/LICENSE.txt` recording both works, the Blender Foundation copyright, the CC-BY 3.0 licence and a link to it.** That file is not part of this correction, because only this file and `inventory.csv` were in scope for it. `inventory.csv` now carries a `License` column marking both rows.

### The download script does not reproduce this directory

`download_test_videos.ps1` is not a recipe for the committed tree. Three reasons:

1. Five of its sixteen downloads use dead URLs and fail: `sample-1mb.mp4` (line 19), `sample-5mb.mov` (line 97), `sample-1mb.mkv` (line 122), `320x240_test_pattern.ogv` (line 148) and `big_buck_bunny_ecu.ogv` (line 149).
2. It renames five files without saying so in this document, so a re-run produces different names from the ones in the tables above: `avi_5s_sample_file_318KB.avi` to `sample-5s.avi`, `avi_15s_sample_file_927KB.avi` to `sample-15s.avi`, `test1.mkv` to `test1-matroskaconformance.mkv`, and the two Wikimedia clips to `320x240_test_pattern.ogv` and `big_buck_bunny_ecu.ogv`. This document now records the mapping in the relevant tables.
3. It has no entry at all for `sample-1mb_alt.mp4` or `elephants_dream_1024x576_30s.mp4`, both of which are committed. A fresh run cannot produce them. Where `sample-1mb_alt.mp4` came from is unknown.

### Decisions left to the owner

- Creating `TESTING_videos/LICENSE.txt` for the two CC-BY works (required, not yet done).
- Choosing replacements for the seven dead URLs, or accepting that those fixtures do not exist.
- Whether to delete `sample-1mb_alt.mp4`, which is committed HTML rather than media, and what its intended source was.
- Whether `test1-matroskaconformance.mkv` should stay in the catalogue as a refusal fixture or be replaced with an H.264 MKV so the section has a playable file of its own. `sample_640x360.mkv` already covers the playable case.
