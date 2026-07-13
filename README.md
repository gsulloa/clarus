# Clarus

Clarus is a Tauri desktop app for understanding local disk usage before taking action. The product direction is precise, calm, and dry-run first: scan, explain, review candidates, then let the user decide.

## Workspace

```sh
pnpm install
pnpm dev
pnpm tauri:dev
pnpm build
pnpm test
pnpm infra:synth
```

## Packages

- `packages/app`: Tauri 2 + React desktop app.
- `packages/infra`: AWS CDK infrastructure for `clarus.gulloa.click`, release artifacts, and updater manifests.

## Release Surface

Clarus ships through signed Tauri bundles. Tag pushes matching `v*` run `.github/workflows/release.yml`, publish installers to GitHub Releases, build `latest.json` for Tauri autoupdate, build `download.json` for the landing page, and upload both to the releases bucket behind CloudFront.

Required GitHub secrets (`gsulloa/clarus`):

- Apple signing / notarization (team-scoped, shared with the author's other
  Tauri apps): `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`,
  `APPLE_PASSWORD`, `APPLE_TEAM_ID`.
- Tauri updater signing (Clarus-specific): `TAURI_UPDATER_PRIVATE_KEY`,
  `TAURI_UPDATER_KEY_PASSWORD`.
- Release infra: `AWS_REGION`, `AWS_RELEASE_ROLE_ARN`, `RELEASE_S3_BUCKET`,
  `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, and `PUBLIC_URL_BASE`
  (`https://releases.clarus.gulloa.click`).

### Local secrets (`.envrc`)

Local signed builds and manual publishing read credentials from a `.envrc` kept
at the repo root and loaded by [direnv](https://direnv.net/). It is **git-ignored**
and never committed. It exports the Apple credentials as literals, points
`TAURI_SIGNING_PRIVATE_KEY` at `~/.tauri/clarus-updater.key`, sets `AWS_PROFILE`,
`AWS_REGION`, and `PUBLIC_URL_BASE`, and resolves `RELEASE_S3_BUCKET` /
`RELEASE_CLOUDFRONT_DISTRIBUTION_ID` / `AWS_RELEASE_ROLE_ARN` from the SSM
parameters published by `ClarusReleasesStack`.

### Updater keypair

The Tauri autoupdater keypair is provisioned (and rotated) with:

```sh
pnpm --filter clarus exec bash scripts/set-updater-keys.sh          # create or resync
pnpm --filter clarus exec bash scripts/set-updater-keys.sh --rotate # new keypair
```

It generates the keypair at `~/.tauri/clarus-updater.key`, writes the public key
into `packages/app/src-tauri/tauri.conf.json` (`plugins.updater.pubkey`), and sets
the `TAURI_UPDATER_PRIVATE_KEY` / `TAURI_UPDATER_KEY_PASSWORD` GitHub secrets. Back
up the private key — losing it forces a rotation that breaks update verification
for already-installed clients.

## Infrastructure

The CDK app expects the `gulloa.click` hosted zone to exist. Pass the hosted zone id through context:

```sh
pnpm infra:synth -- --context hostedZoneId=Z123456789
```

Deploy DNS/certificate first, then releases and landing stacks:

```sh
pnpm infra cdk deploy ClarusDnsStack --context hostedZoneId=Z123456789
pnpm infra cdk deploy ClarusReleasesStack ClarusLandingStack --context hostedZoneId=Z123456789
```
