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

Required GitHub secrets:

- `TAURI_UPDATER_PRIVATE_KEY`
- `TAURI_UPDATER_KEY_PASSWORD`
- `PUBLIC_URL_BASE`, expected to be `https://releases.clarus.gulloa.click`
- `AWS_RELEASE_ROLE_ARN`
- `AWS_REGION`
- `RELEASE_S3_BUCKET`
- `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`
- Apple signing and notarization secrets when macOS releases are enabled.

This scaffold generated a local updater keypair for Clarus. The private key and
password are intentionally outside git:

- `.context/clarus-updater.key`
- `.context/clarus-updater-password.txt`

Use those values for `TAURI_UPDATER_PRIVATE_KEY` and
`TAURI_UPDATER_KEY_PASSWORD`, or generate a new pair and replace the public key
in `packages/app/src-tauri/tauri.conf.json`.

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
