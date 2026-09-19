# Phase 0: what contemporary upstream actually does

Measured against `water-rs/waterui` at `a1210501` (2026-09-19), from the fork
`maxiboch/waterui`. Upstream moved four times on the day this was written, so
re-measure before acting on any figure here; the durable content is the
findings, not the SHAs.

The question phase 0 exists to answer is narrow: before porting anything, does
upstream build and render for the shapes our consumers need, and what does it
cost to find out? Everything below came from running it rather than reading it.

## The fork's CI is not upstream's CI, and that matters twice

**Upstream's pull-request gate does not test.** On our probe PR, `Changes`,
`Typos`, `Hygiene`, `Release scripts`, `Android FFI` and `FFI Header` all
passed and *every* `Test /` job skipped. That is deliberate: every substantive
job in `test.yml` is gated on a `full` input, and the real matrix lives in
`nightly.yml`, which GitHub leaves `disabled_fork` on a fork.

So inheriting upstream's CI buys almost no signal unless you enable their
nightly — and that runs their entire platform matrix daily to answer questions
about code we do not touch. We did not enable it. We added one workflow of our
own that builds one crate on two targets.

This is the operational form of the de-vendoring argument: **we run our CI on
our code, and upstream's CI stays upstream's problem.** Once satellites are
dependencies rather than vendored trees, their matrices are never ours to run.

Worth recording because it is easy to assume otherwise: GitHub-hosted runners
are free for public repositories, and upstream's workflows are entirely
GitHub-hosted with no self-hosted jobs. Running their CI on a public fork costs
no money and no maxi-tools runner time. The reason not to inherit it is signal
and maintenance, not cost.

## `--locked` is enforced everywhere

Upstream runs `cargo clippy --locked` and `cargo check --locked` throughout.
A workspace member added without its `Cargo.lock` entry therefore fails the
gate **before compiling anything** — `Test / lint` went red at 1m07s, and the
failure says nothing about the code.

Every manifest change in phases 1–3 has to land with its lock entry in the same
commit. A lock entry can be written by hand when the dependencies already
resolve elsewhere in the tree, which matters on a shared CI runner where
running cargo locally is not acceptable.

## The headless lane has a native dependency floor

Upstream carries `.github/actions/setup-linux-deps` because a Linux build of
the workspace needs ALSA, PipeWire, VA-API, nasm-assembled codecs, fonts, GTK4,
EGL and udev. Behind a `gpu` input it also installs the Vulkan ICD loader.

That input is the load-bearing part: `mesa-vulkan-drivers` brings
llvmpipe/lavapipe, and a **software** Vulkan implementation is what lets a
GPU-backed render run headless on a machine with no GPU. That is the entire
hydrolysis lane — the one maxi-tray, maxi-visage and the HUD gallery depend on,
and the one the fleet depends on because every Mac in it is screen-locked.

Consequence for phase 4 and for the fleet: "hydrolysis headless" is not free
anywhere we run it. Any box expected to produce snapshots needs this floor
installed, not just a Rust toolchain.

## The consumer dependency map is smaller than assumed

Of the six waterui crates maxi-ui pins, five remain inside upstream's one
workspace and one has left:

| crate | upstream | phase-1 action |
|---|---|---|
| `waterui` | in-tree, `path = "."`, 0.4.1 | git dep on the fork |
| `waterui-ffi` | in-tree, `ffi`, 0.3.2 | same repo, subpath |
| `waterui-controls` | in-tree, `components/foundation/controls`, 0.3.2 | same repo, subpath |
| `waterui-graphics` | in-tree, `components/visual/graphics`, 0.4.1 | same repo, subpath |
| `waterui-testing` | in-tree, `testing` | same repo, subpath |
| `waterui-canvas` | **registry `"0.2.0"`, no path** | plain crates.io dependency |

Canvas is already published, so it needs no git pin at all. The phase-1 edit is
correspondingly smaller than the plan assumed.

## Two Swift patches, with different futures

voicemaci's `build.rs` applies exactly two things to the Apple backend:

- **`swiftLanguageModes: [.v5]`** — forces Swift 5 mode because our vendored
  backend's C factory returns a non-Sendable type that Swift 6 strict
  concurrency rejects. Upstream hit the identical wall (their issue #867,
  "apple-backend-version 0.2.0 pin fails to compile under Swift 6 strict
  concurrency") and fixed it: `apple-backend` at `dev` is
  `swift-tools-version: 6.3.0` with no language-mode downgrade. **This patch
  plausibly retires on the port.**
- **A hermetic C factory overlay** — an additive Swift file copied into the
  package's `Sources/WaterUI/`, SHA-256 pinned in `build.rs`, with an explicit
  note that it never mutates upstream's `WaterUI.swift`. That is the
  disciplined shape for a local change, and it ports cleanly if still needed.
  Whether it is still needed depends on how `apple-backend` 0.3.0 exposes the
  factory symbol.

`build.rs` also already records that `maxi-tools/apple-backend` "has since been
deleted — `git ls-remote` and the GitHub API both 404", which is why it reaches
into `maxi-tools/waterui` at `backends/apple`. The submodule fork this replaced
is gone; upstream consumes `apple-backend` as a versioned dependency instead.
