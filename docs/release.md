# Release Checklist

## Trust Model
Release tags are published through GitHub Releases, not by a personal
maintainer signing key. This project treats the canonical GitHub repository as
the release authority: GitHub creates the release tag, release immutability
locks the tag and assets after publication, and GitHub generates the release
attestation for consumers who want platform-backed provenance.

This is intentionally different from an OpenPGP-signed Git tag. Do not promise
Arch-style `?signed#tag=` verification unless the project later adopts a
separate tag-signing key.

## Prep
- Update `CHANGELOG.md` with release notes and date.
- Bump versions in `Cargo.toml` (workspace and crates) as needed.
- Ensure `Cargo.lock` is updated and committed.
- Confirm vendor/picoquic is at the intended commit and submodules are initialized.
- Ensure release immutability is enabled for the repository before publishing
  the release:
  https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/establish-provenance-and-integrity/preventing-changes-to-your-releases
- Confirm the release tag does not already exist on `origin`:
  `git ls-remote --exit-code --tags origin refs/tags/vX.Y.Z`

## Validation
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test -p slipstream-dns`
- `cargo test`

## Hygiene
- Verify build outputs stay untracked: `.interop/`, `.picoquic-build/`, `target/`.
- Regenerate vectors and docs if DNS behavior changed:
  `./scripts/gen_vectors.sh`, `docs/protocol.md`, `docs/dns-codec.md`.

## Release
- For a source-only release, create the GitHub Release and let GitHub create
  the tag:

  ```sh
  gh release create vX.Y.Z \
    --repo Mygod/slipstream-rust \
    --target COMMIT_SHA \
    --title vX.Y.Z \
    --notes-file RELEASE_NOTES.md
  ```

  Use the exact commit SHA that passed validation. Do not create or push a
  local Git tag first for this workflow.
- For a release with attached assets, create a draft first, upload all assets,
  then publish it:

  ```sh
  gh release create vX.Y.Z \
    --repo Mygod/slipstream-rust \
    --target COMMIT_SHA \
    --title vX.Y.Z \
    --notes-file RELEASE_NOTES.md \
    --draft
  gh release upload vX.Y.Z dist/* --repo Mygod/slipstream-rust
  gh release edit vX.Y.Z --repo Mygod/slipstream-rust --draft=false
  ```

  Release immutability only locks the tag and assets after the release is
  published.
- Verify the published release:

  ```sh
  gh release verify vX.Y.Z --repo Mygod/slipstream-rust
  git ls-remote --tags origin refs/tags/vX.Y.Z
  ```

- Fetch the tag locally after publication if needed:

  ```sh
  git fetch --tags origin
  ```

## Packager Notes
- Source packages can track `vX.Y.Z` tags from the canonical GitHub repository.
- GitHub release verification uses GitHub's release attestation and immutable
  release state:
  https://docs.github.com/en/code-security/how-tos/secure-your-supply-chain/secure-your-dependencies/verifying-the-integrity-of-a-release
- GitHub-generated source archives are created on download, so verify the
  release/tag identity rather than treating those archives as pre-attached
  release assets.
