## 1. Local `.envrc` at canonical repo root (shareable + Clarus values)

> Base already created at `/Users/gabrielulloa/dev/freelance/clarus/.envrc`:
> `AWS_PROFILE=Clarus` and `CLARUS_HOSTED_ZONE_ID=Z02739252KVUU2FWNWPTV`. Extend it, don't recreate it.

- [x] 1.1 Read the six shareable Apple values from `/Users/gabrielulloa/dev/freelance/tokenwatch/.envrc` (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `APPLE_SIGNING_IDENTITY`)
- [x] 1.2 Append to the existing canonical `.envrc` (keeping `AWS_PROFILE=Clarus` and `CLARUS_HOSTED_ZONE_ID`): the six Apple exports as literals, `export AWS_REGION=us-east-1`, `export PUBLIC_URL_BASE="https://releases.clarus.gulloa.click"` — modeled on the TokenWatch `.envrc`
- [x] 1.3 Add the SSM-resolution block (guarded by `command -v aws`) that exports `RELEASE_S3_BUCKET` from `/Clarus/releases/bucket-name`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID` from `/Clarus/releases/distribution-id`, and `AWS_RELEASE_ROLE_ARN` from `/Clarus/releases/publish-role-arn`
- [x] 1.4 Add `.envrc` to `.gitignore` in the canonical repo (currently only `.env`/`.env.*` are ignored) and run `direnv allow` so it loads without error even before infra exists
- [x] 1.5 Remove the duplicate base `.envrc` from the Conductor worktree (`/Users/gabrielulloa/conductor/workspaces/clarus/abu-dhabi-v2/.envrc`) and add `.envrc` to the worktree/repo `.gitignore` so no secret-bearing copy can be committed
- [x] 1.6 Delete the empty `.context/clarus-updater.key` and `.context/clarus-updater-password.txt` files and remove/adjust the README lines that reference them

## 2. Clarus updater keypair

- [x] 2.1 Port TokenWatch's `packages/app/scripts/set-updater-keys.sh` to Clarus (`REPO=gsulloa/clarus`, `KEY_PATH=~/.tauri/clarus-updater.key`, `pnpm --filter clarus exec tauri signer generate`)
- [x] 2.2 Run `set-updater-keys.sh` to generate the keypair, writing `plugins.updater.pubkey` into `packages/app/src-tauri/tauri.conf.json`
- [x] 2.3 Confirm the script set GitHub secrets `TAURI_UPDATER_PRIVATE_KEY` and `TAURI_UPDATER_KEY_PASSWORD` on `gsulloa/clarus`, and back up `~/.tauri/clarus-updater.key`

## 3. AWS release infrastructure

- [x] 3.1 Verify the `Clarus` account (`092040680426`) owns the `gulloa.click` hosted zone `Z02739252KVUU2FWNWPTV`: `aws route53 get-hosted-zone --id Z02739252KVUU2FWNWPTV --profile Clarus` (if it's a cross-account/TokenWatch zone, resolve DNS delegation before deploying)
- [x] 3.2 Add SSM `StringParameter`s to `packages/infra/lib/ReleasesStack/index.ts`: `/Clarus/releases/bucket-name`, `/Clarus/releases/distribution-id`, `/Clarus/releases/publish-role-arn` (alongside existing CfnOutputs)
- [x] 3.3 `pnpm infra:build` and `pnpm infra:synth` to validate the stack
- [x] 3.4 `cdk deploy ClarusDnsStack --context hostedZoneId=$CLARUS_HOSTED_ZONE_ID` (profile `Clarus`)
- [x] 3.5 `cdk deploy ClarusReleasesStack ClarusLandingStack --context hostedZoneId=$CLARUS_HOSTED_ZONE_ID`; capture bucket name, distribution id, and publish-role ARN
- [x] 3.6 Re-run `direnv reload` in the canonical repo and confirm `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, `AWS_RELEASE_ROLE_ARN` now resolve from SSM

## 4. Fix CI signing in `release.yml`

- [x] 4.1 Diff Clarus `release.yml` `Build signed bundle` against TokenWatch's; add the missing `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD` (env on the tauri-action step and/or a preceding keychain-import step, matching TokenWatch)
- [x] 4.2 Verify the workflow's secret references (`PUBLIC_URL_BASE`, `AWS_RELEASE_ROLE_ARN`, `AWS_REGION`, `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`) all have corresponding secrets provisioned in step 5

## 5. Mirror secrets to GitHub (`gsulloa/clarus`)

- [x] 5.1 Set the six shareable Apple secrets via `gh secret set --repo gsulloa/clarus` (values read from the local `.envrc`, never pasted into commits/PRs)
- [x] 5.2 Set `AWS_REGION`, `AWS_RELEASE_ROLE_ARN`, `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, `PUBLIC_URL_BASE` from the deployed infra values
- [x] 5.3 Confirm `TAURI_UPDATER_PRIVATE_KEY` / `TAURI_UPDATER_KEY_PASSWORD` are present (from step 2.3)
- [x] 5.4 Run `gh secret list --repo gsulloa/clarus` and confirm the full required set from the spec is present

## 6. Verify & document

- [x] 6.1 `pnpm --filter clarus release:dry` to confirm the local release flow runs
- [x] 6.2 Update `README.md` Release Surface section to reflect the real secret set, the local `.envrc` location, and the `set-updater-keys.sh` flow (remove the empty-keyfile instructions)
- [ ] 6.3 (Optional, gated on user go-ahead) push a prerelease tag (e.g. `v0.1.0-rc.1`) and confirm `release.yml` produces a signed, notarized bundle and publishes `latest.json`/`download.json`
- [x] 6.4 Commit the repo changes (`tauri.conf.json` pubkey, `release.yml`, `ReleasesStack` SSM params, `set-updater-keys.sh`, README) on a branch and open a PR in English
