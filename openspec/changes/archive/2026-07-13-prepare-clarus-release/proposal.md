## Why

Clarus has a full release pipeline scaffolded (`.github/workflows/release.yml`, updater/download manifest builders, CDK release infra) but **cannot actually publish a signed build**: no GitHub secrets are set on `gsulloa/clarus`, there is no local `.envrc` for local release/signing, the updater keypair referenced in the README is empty (`.context/clarus-updater.key` is a 0-byte file), and the macOS signing certificate is never imported in CI. Argus and TokenWatch already solved this exact problem. Their Apple signing/notarization credentials are **team-scoped** (Apple Developer Team `9M9FA9YAWP`), so they can be reused verbatim for Clarus; the remaining values (updater keypair, AWS release infra) are per-app and must be provisioned for Clarus.

## What Changes

- **Reuse team-scoped Apple credentials from Argus/TokenWatch** — `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `APPLE_SIGNING_IDENTITY`. These are identical across both existing apps because a "Developer ID Application" cert is team-scoped, not app-scoped.
- **Add a local `.envrc`** at the canonical Clarus repo root (`/Users/gabrielulloa/dev/freelance/clarus`, **not** this Conductor git worktree), modeled on the TokenWatch `.envrc`: Apple secrets as literals, AWS release infra resolved from SSM at load, `PUBLIC_URL_BASE=https://releases.clarus.gulloa.click`.
- **Set the shareable secrets on GitHub** for `gsulloa/clarus` via `gh secret set`.
- **Provision the per-app secrets** that cannot be shared: generate a Clarus Tauri updater keypair (writing its pubkey into `tauri.conf.json`, private key + password into the local key file and GitHub), and deploy the CDK release infra to obtain `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, `AWS_RELEASE_ROLE_ARN`, and set `AWS_REGION` / `PUBLIC_URL_BASE`.
- **Fix the CI signing gap**: the Clarus `release.yml` `Build signed bundle` step does not pass `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD`, so notarized macOS bundles cannot be produced. Align it with the TokenWatch workflow (import cert / pass cert env).
- **Make infra publish SSM parameters** (bucket name, distribution id, publish-role ARN) so the local `.envrc` can resolve them the same way Argus/TokenWatch do, instead of hardcoding CloudFormation outputs.
- **Add a `set-updater-keys.sh` helper** (mirroring TokenWatch) so the keypair provisioning is reproducible and documented.

## Capabilities

### New Capabilities
- `release-credentials`: How Clarus's release secrets are sourced, classified (shareable vs per-app), stored locally (`.envrc` at the non-worktree repo root), and mirrored to GitHub Actions secrets, so both local and CI signed releases work.

### Modified Capabilities
<!-- No existing spec's requirements change; this is additive release plumbing. -->

## Impact

- **Local filesystem (outside git)**: new `/Users/gabrielulloa/dev/freelance/clarus/.envrc`; new/regenerated `~/.tauri/clarus-updater.key` (+ `.pub`). Never committed.
- **GitHub `gsulloa/clarus`**: new Actions secrets (Apple set, updater set, AWS release set, `PUBLIC_URL_BASE`, `AWS_REGION`).
- **Repo**: `packages/app/src-tauri/tauri.conf.json` (`plugins.updater.pubkey`), `.github/workflows/release.yml` (Apple cert import), `packages/infra/lib/ReleasesStack` (SSM params), new `packages/app/scripts/set-updater-keys.sh`, README release section.
- **AWS**: CDK deploy of `ClarusDnsStack`, `ClarusReleasesStack` (and `ClarusLandingStack`) under the `Clarus` profile (account `092040680426`, `us-east-1`), using hosted zone `Z02739252KVUU2FWNWPTV` (`gulloa.click`, already exported as `CLARUS_HOSTED_ZONE_ID`).
- **Security**: secrets are copied, not rotated. Apple app-specific password and cert password are reused across three apps; documented as an accepted trade-off. No secret is committed to git.
