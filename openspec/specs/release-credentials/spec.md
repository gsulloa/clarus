# release-credentials Specification

## Purpose
Defines how Clarus provisions the signing, notarization, updater, and AWS release credentials required to produce and publish signed, notarized macOS bundles — distinguishing team-scoped Apple values that are reused from sibling apps (Argus, TokenWatch) from per-app values provisioned specifically for Clarus, and how these are wired into the local `.envrc`, the committed Tauri config, and GitHub Actions secrets.

## Requirements
### Requirement: Shareable Apple credentials are reused from sibling apps

Clarus SHALL reuse the team-scoped Apple signing and notarization credentials already provisioned for Argus and TokenWatch, because a "Developer ID Application" certificate is scoped to the Apple Developer Team (`9M9FA9YAWP`), not to an individual app. These values MUST NOT be regenerated for Clarus.

The shareable set is exactly: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `APPLE_SIGNING_IDENTITY`.

#### Scenario: Apple credentials copied verbatim

- **WHEN** Clarus's release credentials are provisioned
- **THEN** the six Apple values are copied from the TokenWatch `.envrc` (identical to Argus) without modification
- **AND** their values match those already stored as `gsulloa/tokenwatch` and `gsulloa/argus` GitHub secrets

#### Scenario: Non-shareable values are not copied

- **WHEN** deciding which values to reuse
- **THEN** the Tauri updater keypair, `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, `AWS_RELEASE_ROLE_ARN`, and `PUBLIC_URL_BASE` are treated as per-app and are provisioned specifically for Clarus, never copied from a sibling app

### Requirement: Local `.envrc` lives at the canonical repo root

Clarus SHALL have a local `.envrc` written to the canonical repository root `/Users/gabrielulloa/dev/freelance/clarus`, NOT to any Conductor git worktree under `/Users/gabrielulloa/conductor/workspaces/`. The file MUST NOT be committed to git.

The `.envrc` SHALL set `AWS_PROFILE` for Clarus, export the six shareable Apple values as literals, set `AWS_REGION` and `PUBLIC_URL_BASE=https://releases.clarus.gulloa.click`, and resolve `RELEASE_S3_BUCKET` / `RELEASE_CLOUDFRONT_DISTRIBUTION_ID` / `AWS_RELEASE_ROLE_ARN` from SSM at load (guarded by `command -v aws`), mirroring the TokenWatch `.envrc` structure.

#### Scenario: File location

- **WHEN** the `.envrc` is created
- **THEN** it exists at `/Users/gabrielulloa/dev/freelance/clarus/.envrc`
- **AND** it does not exist inside any `conductor/workspaces/clarus/*` worktree
- **AND** `.envrc` is covered by `.gitignore` (or is otherwise untracked)

#### Scenario: Infra values resolved dynamically

- **WHEN** direnv loads the `.envrc` and the `aws` CLI is available
- **THEN** `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, and `AWS_RELEASE_ROLE_ARN` are read from Clarus SSM parameters under the Clarus AWS profile
- **AND** if the `aws` CLI is unavailable, loading the file does not error

### Requirement: Shareable secrets are mirrored to GitHub Actions

The shareable Apple credentials, the Clarus-specific updater secrets, and the AWS release values SHALL be set as GitHub Actions secrets on `gsulloa/clarus` so that the `release.yml` workflow can produce signed, notarized bundles and publish them.

#### Scenario: Required secrets present after provisioning

- **WHEN** provisioning completes
- **THEN** `gh secret list --repo gsulloa/clarus` includes at least: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `TAURI_UPDATER_PRIVATE_KEY`, `TAURI_UPDATER_KEY_PASSWORD`, `AWS_REGION`, `AWS_RELEASE_ROLE_ARN`, `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, `PUBLIC_URL_BASE`

### Requirement: Clarus has its own updater keypair

Clarus SHALL have a dedicated Tauri updater signing keypair. The current scaffolded key file (`.context/clarus-updater.key`) is empty and MUST be replaced. The public key MUST be written into `packages/app/src-tauri/tauri.conf.json` at `plugins.updater.pubkey`, and the private key + password MUST be stored in the local key file and as GitHub secrets — consistent between local `.envrc`, the committed pubkey, and CI.

#### Scenario: Keypair consistent across surfaces

- **WHEN** the updater keypair is provisioned
- **THEN** the `pubkey` in `tauri.conf.json` corresponds to the private key stored in `TAURI_UPDATER_PRIVATE_KEY`
- **AND** the private key is never committed to git

### Requirement: CI workflow imports the signing certificate

The `release.yml` `Build signed bundle` step SHALL provide the Apple signing certificate to `tauri-action` (via `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD` env or an explicit keychain-import step), matching the working TokenWatch workflow, so that macOS bundles are actually signed and notarized.

#### Scenario: Certificate available at build time

- **WHEN** the macOS matrix job runs the signing step
- **THEN** `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` are available to the signing/notarization tooling
- **AND** the produced `.app.tar.gz` has a valid `.sig` and the `.dmg` is notarized
