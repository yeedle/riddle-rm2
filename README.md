# riddle — the diary of Tom Riddle, for the reMarkable Paper Pro

Write on the page with your pen. After a pause, the diary **drinks your ink** —
your words fade into the paper — the page thinks for a moment, and an answer
writes itself back in a flowing hand, stroke by stroke, then fades away.

No screen glow, no keyboard, no chat UI. Just ink appearing on paper.

_This is the diary from [the demo](https://x.com/MaximeRivest)._

### 🪄 New to this? Start here

You need a **reMarkable Paper Pro** in developer mode with a launcher installed.
If that sounds like a lot, it isn't — **[remagic](https://github.com/maximerivest/remagic)**
walks you through turning on developer mode and sets up everything with one
command. Come back here, drop riddle in, and start writing to Tom.

Already have xovi + AppLoad? Install from the [remagic](https://github.com/maximerivest/remagic)
catalog, [grab the prebuilt bundle](#install-the-prebuilt-bundle), or
[build from source](#building).

### Install with remagic (easiest)

```sh
remagic install riddle     # checksum-verified download → AppLoad
remagic config riddle      # settings form in your browser (+ QR for phone)
```

Then in **AppLoad**: tap **Reload**, then **The Diary**. Write, and rest your
pen. (Or install it from the **Store** app right on the tablet.)

### Install the prebuilt bundle

1. Grab `riddle-<version>.zip` from the [latest release](https://github.com/MaximeRivest/riddle/releases/latest)
   and unzip it into a folder: `unzip riddle-*.zip -d riddle`
2. Copy the folder to your tablet:
   `scp -O -r riddle root@10.11.99.1:/home/root/xovi/exthome/appload/`
3. Add an API key: `cp oracle.env.example oracle.env` in that folder and put your `RIDDLE_OPENAI_KEY` in it (any OpenAI-compatible key). Or skip it to use [pi](#option-b--pi-the-power-path).
4. In **AppLoad**: tap **Reload**, then **The Diary**. Write, and rest your pen.

> ⚠️ **This modifies your device.** The prebuilt bundle and the catalog build
> run in **takeover mode**: tapping The Diary stops the whole reMarkable UI
> and takes the screen. Leave with a **5-finger tap** — xochitl restarts
> automatically. It runs as root and drives the e-ink engine directly. It has
> only been tested on a **reMarkable Paper Pro** (ferrari, aarch64,
> OS 3.26–3.27). It may not work on other models or OS versions, and you use
> it entirely at your own risk. Not affiliated with reMarkable AS. Keep SSH
> access working before you install anything — if anything ever wedges:
> `ssh root@10.11.99.1 'systemctl start xochitl'`.

## reMarkable 2 — SSH access and compatibility notes

The project was built and tested on the **reMarkable Paper Pro**. Running it on a
**reMarkable 2** is possible with some adjustments; this section covers the differences.

### SSH access on reMarkable 2 (simpler than Paper Pro)

Unlike the Paper Pro, the reMarkable 2 **does not require developer mode** to expose SSH
credentials — they are visible right away:

1. Open the menu (three horizontal lines, top-left corner).
2. Go to **Settings → Help → Copyrights and licenses**.
3. Scroll to the **GPLv3 Compliance** section at the bottom — your username (`root`),
   password, and IP addresses are listed there.

Connect over USB (the device always appears as `10.11.99.1` when plugged in):

```sh
ssh root@10.11.99.1
```

WLAN SSH is disabled by default. Once connected via USB, enable it with:

```sh
rm-ssh-over-wlan on
```

After that you can connect wirelessly using the WLAN IP shown in the same GPLv3 section.
To copy files to the tablet (same path as Paper Pro):

```sh
scp -O -r riddle root@10.11.99.1:/home/root/xovi/exthome/appload/
```

> 💡 **Tip:** To avoid typing the password each time, copy your SSH public key to the
> device with `ssh-copy-id root@10.11.99.1` after your first connection.

### Prerequisites on the reMarkable 2

- **OS 3.9 or newer — 3.27.x recommended.** Older OS versions (≤3.8) ship Qt 5;
  xovi's extensions target Qt 6 and silently do nothing on Qt 5. Update the
  tablet first (Settings → General → Software update; it may step through
  intermediate versions — keep updating until none are offered).
- **xovi + AppLoad** installed:
  1. `xovi-arm32.tar.gz` from [rm-xovi-extensions releases](https://github.com/asivery/rm-xovi-extensions/releases)
     extracted to `/home/root/` (gives `/home/root/xovi/` with `qt-resource-rebuilder.so`).
  2. `appload.so` from the arm32 zip of [rm-appload releases](https://github.com/asivery/rm-appload/releases)
     into `/home/root/xovi/extensions.d/`; the two `qtfb-shim*.so` files into `/home/root/shims/`.
  3. Build the hashtable: `/home/root/xovi/rebuild_hashtable` (takes a couple of
     minutes; the tablet must not sleep — press power if it does).
  4. `/home/root/xovi/start` — an **AppLoad** entry appears in the xochitl menu.

### Display backend on reMarkable 2

- **Windowed mode (AppLoad/qtfb)** — the recommended path, and what this fork
  targets. The rM2 is 32-bit: build for **`armv7-unknown-linux-gnueabihf`**
  with the `rm2` feature. From any machine with Docker:

  ```sh
  docker run --rm -v "$PWD":/work -w /work/riddle rust:1-bullseye bash -c '
    apt-get update -qq && apt-get install -y -qq gcc-arm-linux-gnueabihf
    rustup target add armv7-unknown-linux-gnueabihf
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
    cargo build --release --target armv7-unknown-linux-gnueabihf --features rm2'
  ```

  Then stage an app folder and copy it over (`killall riddle` on the tablet
  first if it is running — otherwise scp fails with "Text file busy"):

  ```
  riddle/                       -> /home/root/xovi/exthome/appload/riddle/
  |- riddle                        the armv7 binary (chmod +x)
  |- riddle-start.sh               launcher: sources oracle.env, execs riddle
  |- external.manifest.json        "application": "riddle-start.sh", "qtfb": true
  |- oracle.env                    your API key + settings (see oracle.env.example)
  |- icon.png
  ```

  `riddle-start.sh` matters: the binary does **not** read `oracle.env` itself —
  the launcher must source it (`set -a; . ./oracle.env; set +a; exec ./riddle`).
  In AppLoad tap **Reload**, then **The Diary**.

- **Takeover mode (quill)** — depends on `libqsgepaper.so` pulled from *your own device*.
  The vendor library exists on the reMarkable 2 but targets a different board variant; the
  shim *may* work, but it has **not** been tested on rM2. Start with the windowed backend
  and switch to takeover only if you need lower latency.

> ⚠️ Keep SSH access live as your escape hatch at all times. If the UI ever
> wedges: `ssh root@10.11.99.1 'systemctl restart xochitl'`.

### Closing the diary (windowed mode)

Three ways out:

- Tap the **✕ button** in the top-right corner — pen or finger.
- Write **close**, **exit**, **quit**, or **goodbye** as the only word on the
  page and rest the pen (the diary confirms via the transcription, so this one
  takes an oracle round-trip).
- From a terminal: `killall riddle`.

### Personalizing the writer (any device)

Set `RIDDLE_PERSONA_EXTRA` in `oracle.env` to append to Tom's system prompt
without rebuilding — the writer's name, age, tone limits, anything:

```sh
RIDDLE_PERSONA_EXTRA="The writer's name is Leah, age 10 … keep it kind."
```

### Resetting the conversation

`rm -rf /home/root/riddle-data/memories` (with the diary closed) makes the
diary forget everything.

### Material changes in this fork (vs upstream riddle)

- **PR [#19](https://github.com/MaximeRivest/riddle/pull/19) applied** — required
  for windowed mode on rM2: the 32-bit AppLoad shim lays out its `ServerMessage`
  with data at offset 4 (not 8), so the old code read a garbage SHM key and died
  on launch with `fatal: No such file or directory`; input-event offsets had the
  same bug (garbage pen coordinates); raw evdev pen is now only opened in
  takeover mode (under qtfb the shim owns the digitizer).
- **Full-canvas geometry in windowed mode** — the qtfb shim always presents a
  1620×2160 (Paper-Pro-size) canvas scaled to the device screen, on every
  device. The 1404×1872 rM2 dimensions now apply only to `rm2`+`takeover`
  builds; previously the windowed build left a dead margin on the right/bottom
  where ink was neither committed nor faded.
- **Visible ✕ close button** (windowed mode), tappable with pen or finger; the
  old corner-tap gesture had a `break` that never actually exited the app.
- **Written farewells** — `close` / `exit` / `quit` / `goodbye` as the only
  word on the page closes the diary.
- **E-ink cleanup passes** — live ink stays on the fast waveform, but the
  drink-fade and Tom's finished replies each get one high-quality (GC16)
  re-render (`Display::deghost`): no more pale handwriting remnants after the
  fade, no more dithering blotches in the reply text.
- **`RIDDLE_PERSONA_EXTRA`** env hook (see above).
- **Font** — replies render in [Patrick Hand](https://fonts.google.com/specimen/Patrick+Hand)
  (print-style, child-readable) instead of Dancing Script; the font is embedded
  at build time (`riddle/fonts/DancingScript.ttf` was replaced in place, the
  original kept as `.orig` — swap the file and rebuild to change it).
- **qtfb flush interval** 20 ms → 10 ms.

## How it works

```
 pen (raw evdev, full 4096-level pressure, hardware event rate)
   │ strokes
   ▼
 riddle ── idle 2.8s → commit page → PNG ──► oracle (resident LLM process,
   │                                          streams reply sentence-by-sentence)
   ▼ strokes (Dancing Script → skeletonized to single-pixel pen paths)
 display backend
   ├── qtfb        — windowed, inside xochitl (build-from-source flavour)
   └── quill       — full takeover: xochitl stopped, vendor e-ink engine
                     driven directly for instant ink (lowest latency there
                     is; what the prebuilt bundle runs)
```

- **`riddle/`** — the app (Rust). Pen input, ink surface, handwriting
  synthesis (rasterize → Zhang-Suen thinning → stroke tracing → animated
  replay), the oracle process manager, and both display backends.
- **`quill/`** — the takeover display host (C/C++). An
  [epfb-re](https://github.com/asivery/epfb-re)-style QImage-constructor
  interposition shim over the vendor `libqsgepaper.so` waveform engine,
  exposed as a small C ABI (`quill_init` / `quill_buffer` / `quill_swap`)
  that riddle links against with `--features takeover`. Also carries a small
  family of demos (`scribble`, a pen-to-glass latency test, plus map, image,
  and GIF renderers).

## Gestures

| Do this | And |
|---------|-----|
| Write, then rest the pen | The diary drinks your ink and Tom replies |
| Write *"show me what I wrote about…"* | The remembered page **rises through the paper**: the date, your own handwriting rewriting itself stroke by stroke, Tom's old reply — all in faded ink. Touch the pen anywhere and today's page returns |
| Write *"what do you remember?"* | Tom answers with a handwritten list of remembered moments |
| Flip the marker | Erase |
| Draw a large **?** | Summon the built-in guide |
| Tap five fingers at once | Leave the diary *(takeover mode)* |
| Power button | The page turns to *"The diary sleeps."*, then the tablet suspends; press again to wake exactly where you were *(takeover mode)* |

In the windowed (qtfb) flavour, xochitl keeps the touchscreen and the power
button: close the diary from AppLoad instead.

## The diary remembers

Every finished page is kept — your actual pen strokes, a transcription, and
Tom's reply — so the diary can do three things:

- **Follow the conversation.** Recent pages ride along with each request, so
  Tom remembers what you wrote yesterday (both backends, same behavior).
- **Conjure the past.** Ask in ink — *"show me the page about the garden"*,
  *"find what I wrote on Tuesday"* — and the diary rewrites that page in
  front of you, in your own hand, dated, in faded ink. No buttons, no lists,
  no chrome: the pen is the only interface.
- **Answer from memory.** *"What do you remember?"* gets a handwritten index.

Memories live only on the tablet, in plain files under
`/home/root/riddle-data/memories` (delete the folder and the diary forgets;
the last ~400 pages are kept). `RIDDLE_MEMORY=off` in `oracle.env` turns all
of it off — no storage, and nothing extra sent with requests. Set
`RIDDLE_TZ_OFFSET` (hours from UTC) so memory dates read right.

## The oracle (the "spirit" in the diary)

The diary's replies come from a vision LLM that reads your handwriting from the
committed page (sent as an inline PNG). There are **two backends**, chosen at
startup — pick whichever you have:

### Option A — any OpenAI-compatible API (easiest, zero setup)

Set an API key and riddle talks straight to an OpenAI-compatible
`/chat/completions` endpoint. Works with OpenAI, OpenRouter, Groq, a local
server — anything that speaks the format. No extra software on the tablet.

```sh
export RIDDLE_OPENAI_KEY="sk-..."                       # required
export RIDDLE_OPENAI_BASE="https://api.openai.com/v1"   # optional (default)
export RIDDLE_OPENAI_MODEL="gpt-4o-mini"                # optional; must see images
export RIDDLE_OPENAI_REASONING="low"                    # thinking models only
export RIDDLE_OPENAI_MAX_TOKENS="2000"                  # runaway guard
```

Any vision-capable model works. On the tablet these live in `oracle.env`
next to the binary (see `oracle.env.example`, or just run
`remagic config riddle` — it has one-tap presets for OpenAI, OpenRouter,
and Gemini). Example with OpenRouter:

```sh
export RIDDLE_OPENAI_KEY="$OPENROUTER_API_KEY"
export RIDDLE_OPENAI_BASE="https://openrouter.ai/api/v1"
export RIDDLE_OPENAI_MODEL="openai/gpt-4o-mini"
```

Two gotchas with thinking models (Gemini 3.x, o-series): set
`RIDDLE_OPENAI_REASONING=low` for faster first ink (some providers reject
the field on non-thinking models — leave it unset there), and keep
`RIDDLE_OPENAI_MAX_TOKENS` roomy — hidden reasoning tokens count against it,
and a tight cap starves the visible reply.

Verify your setup before launching the diary:

```sh
riddle --oracle-test path/to/handwriting.png   # prints the streamed reply
```

Measured ~0.9–1.1 s to first ink on-device. The HTTPS is built into riddle
(pure-Rust, no extra libraries).

### Option B — pi (the power path)

If you already run [`pi`](https://github.com/badlogic/pi-mono), riddle will use
a resident `pi --mode rpc` process kept warm (Node + your subscription auth
loaded once), so each turn pays only model latency. Used automatically when
`RIDDLE_OPENAI_KEY` is **not** set. Defaults (override in `oracle.env`):
pi at `/home/root/node/bin` (`RIDDLE_PI_BIN_DIR`), provider `openai-codex`
(`RIDDLE_PI_PROVIDER`), model `gpt-5.4-mini` (`RIDDLE_PI_MODEL`).

Both stream the reply sentence-by-sentence, so the quill starts writing seconds
before the model finishes. The persona prompt lives in `riddle/src/oracle.rs`.

A note on Tom's memory: with the HTTP backend every page is a fresh
conversation — Tom does not remember your previous page. With pi, the warm
session remembers everything since the diary was opened (and pi persists
that session in its own data dir on the tablet).

If the oracle can't answer — missing key, refused key, no Wi-Fi — Tom writes
the reason on the page instead of a reply, and the full error goes to the
journal (`journalctl -u riddle-takeover`).

## Building

Cross-compiled from x86_64. Two flavours:

### Windowed (AppLoad/qtfb) — build from source

The bundles above are the takeover flavour; the windowed flavour must be
built. Requires [xovi + AppLoad](https://github.com/asivery/rm-appload) on
the device.

```sh
cd riddle
cargo build --release --target aarch64-unknown-linux-gnu
```

Install the binary to `/home/root/xovi/exthome/appload/riddle/` with an
`external.manifest.json` that sets `"qtfb": true` and points `"application"`
at the binary itself (the manifest in this repo is the takeover one — AppLoad
only hands riddle a window, via `QTFB_KEY`, when `qtfb` is true).

### Takeover (instant ink) — the one from the demo

Requires the reMarkable SDK toolchain (`~/rm-sdk-3.26`) because the linked
vendor Qt libs need its glibc, **and** `libqsgepaper.so` pulled from *your own
device* (it is proprietary and not distributed here):

```sh
cd quill && ./build.sh              # pulls libqsgepaper.so from the device over
                                    # ssh, builds libquill.so + the demos
cd ../riddle && ./build-takeover.sh
./scripts/make-bundle.sh            # stages the AppLoad bundle in dist/riddle/
```

The staged `dist/riddle/` is self-contained (binary, `libquill.so`, launch
scripts, manifest) — copy it to
`/home/root/xovi/exthome/appload/riddle/`, or publish it to the catalog with
`remagic publish dist/riddle`. Launching via AppLoad (`appload-launch.sh`)
detaches into a transient systemd unit, stops xochitl, runs the diary, and
**always restores xochitl on exit** — leave with a 5-finger tap or SIGTERM
(`systemctl stop riddle-takeover`); the power button sleeps and wakes the
diary without leaving it. The unit's stop hook restarts xochitl even if
riddle dies uncleanly. If anything wedges:
`ssh root@10.11.99.1 'systemctl start xochitl'`.

## What leaves the device

- Each committed page is rasterized to a small grayscale PNG and sent to the
  oracle **you** configured — nothing else ever leaves the tablet, and there
  is no telemetry.
- The PNG (`/tmp/riddle-page.png`) is deleted as soon as the oracle has read
  it; set `RIDDLE_KEEP_PAGE=1` to keep the last page around for debugging.
- riddle never writes replies to disk. The pi backend, however, keeps its own
  session history in its data dir — the HTTP backend keeps nothing.
- Tom stays in character by design: the persona prompt (see
  `riddle/src/oracle.rs`) tells the model it is the diary and nothing else.

## Fonts

The reply hand is [Dancing Script](https://github.com/googlefonts/DancingScript)
(SIL OFL 1.1 — see `riddle/fonts/OFL.txt`).

## License

MIT for everything in this repository (see `LICENSE`). The vendor libraries it
interposes (`libqsgepaper.so`, Qt) are **not** included and must come from
your own device/SDK.