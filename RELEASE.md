# Release process

Instructions for maintainers releasing `tpcgen-rs`. Replace `X.Y.Z` with the
version being released.

## How publishing works

Publishing a GitHub Release whose tag starts with `v` publishes every package.
This is the only way.

| Registry  | Packages                                                                                | Workflow                                                      |
|-----------|-----------------------------------------------------------------------------------------|---------------------------------------------------------------|
| crates.io | `tpchgen`, `tpchgen-arrow`, `tpchgen-cli`, `tpcdsgen`, `tpcdsgen-arrow`, `tpcgen-cli`     | `publish-crates.yml`                                          |
| PyPI      | `tpchgen-cli`, `tpcgen-cli`                                                             | `publish-pypi.yml`                                            |

- Only repository admins can create, move or delete `v*` tags.
- The `release` (crates.io) and `pypi` environments only deploy from `v*` tags.
- Both registries use trusted publishing, so no tokens are needed.
- PyPI versions come from the Cargo version: `X.Y.Z-rc.1` becomes `X.Y.Zrc1`.
- A version can never be reused on crates.io or PyPI.

## 1. Release a release candidate (required)

Every release starts with a release candidate (RC) to test the whole pipeline.
If anything fails, fix it on `main` and release the next RC (`rc.2`, ...).

1. Open a pull request that sets the version to `X.Y.Z-rc.1`:

   ```shell
   cargo install cargo-edit  # once
   cargo set-version --workspace X.Y.Z-rc.1
   ```
2. Merge it through the merge queue. CI dry runs `cargo publish` and builds
   the wheels.
3. Publish the RC as a GitHub pre-release, targeting the merged version-bump
   commit:

   ```shell
   gh release create vX.Y.Z-rc.1 --target <commit-sha> --title vX.Y.Z-rc.1 --generate-notes --prerelease
   ```

4. Wait for the two publish workflows to succeed (`gh run watch` lets you
   pick an active run).
5. Check the RC:

   ```shell
   cargo install tpcgen-cli --version X.Y.Z-rc.1
   uvx tpcgen-cli@X.Y.Zrc1 --version
   uvx tpchgen-cli@X.Y.Zrc1 --version
   ```

## 2. Release the final version

Only continue once an RC has published and been checked successfully.

1. Open a pull request that sets the version to `X.Y.Z`
   (`cargo set-version --workspace X.Y.Z`). It should change nothing else.
2. Merge it through the merge queue.
3. Create a draft release targeting the merged version-bump commit, with
   release notes starting from the previous final release (not the RC):

   ```shell
   gh release create vX.Y.Z --target <commit-sha> --title vX.Y.Z --generate-notes --notes-start-tag <previous-release-tag> --draft
   ```

4. Review and edit the release notes on GitHub, then click **Publish release**.
5. Wait for the two publish workflows to succeed (`gh run watch`).
6. Check the release:

   ```shell
   cargo install tpcgen-cli --version X.Y.Z
   uvx tpcgen-cli@X.Y.Z --version
   uvx tpchgen-cli@X.Y.Z --version
   ```

## If something fails

| Failure                                           | What to do                                                                                     |
|---------------------------------------------------|------------------------------------------------------------------------------------------------|
| Transient error, or a registry setting was wrong  | Fix the setting if needed, then click **Re-run failed jobs** (available for 30 days).          |
| PyPI upload failed partway                        | **Re-run failed jobs**. Files already uploaded are skipped.                                    |
| crates.io upload failed partway                   | Publish the remaining crates manually (below).                                                 |
| Bug in the code or a workflow                     | Fix it on `main`, then release the next RC. If the final release was affected, release `X.Y.(Z+1)`, starting with an RC. |
| A published release is broken                     | Yank it on both registries (`cargo yank --version X.Y.Z <crate>` for each crate; "Yank" on each PyPI project's release page), then release a patch version. |

To publish the remaining crates manually:

1. If **trusted publishing only** is enabled in a crate's settings on
   crates.io, turn it off for each crate still to publish.
2. Publish each remaining crate from the tag with a personal token, in
   dependency order: `tpchgen`, `tpcdsgen`, `tpchgen-arrow`, `tpcdsgen-arrow`,
   `tpcgen-cli`, `tpchgen-cli`.

   ```shell
   git checkout vX.Y.Z
   cargo publish -p <crate>
   ```

3. Turn **trusted publishing only** back on.
