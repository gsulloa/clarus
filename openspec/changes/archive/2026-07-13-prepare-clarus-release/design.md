## Context

Clarus is a Tauri 2 desktop app with a release pipeline already scaffolded but never wired to real credentials. Two sibling apps by the same author — **Argus** (`/Users/gabrielulloa/dev/freelance/argus`) and **TokenWatch** (`/Users/gabrielulloa/dev/freelance/tokenwatch`) — ship the same way (signed Tauri bundles, `latest.json`/`download.json` on S3+CloudFront, GitHub Actions release on `v*` tags). Their `.envrc` files and GitHub secrets are the reference implementation for this change.

Observed current state:
- `gsulloa/clarus` has **zero** GitHub secrets/variables.
- No `.envrc` exists for Clarus (canonical repo `/Users/gabrielulloa/dev/freelance/clarus`).
- `.context/clarus-updater.key` and its password file are **0 bytes** (empty) — the README claims a keypair was generated, but it wasn't persisted.
- `.github/workflows/release.yml` passes `APPLE_ID/PASSWORD/TEAM_ID/SIGNING_IDENTITY` but **not** `APPLE_CERTIFICATE`/`APPLE_CERTIFICATE_PASSWORD`, and has no keychain-import step — so macOS signing/notarization would fail.
- `packages/infra/lib/ReleasesStack` emits CloudFormation **outputs** for bucket/distribution/role but **no SSM parameters**, unlike Argus/TokenWatch whose `.envrc` reads `/<App>/releases/*` SSM params.
- **Resolved (O1):** a `Clarus` AWS profile now exists — SSO session `gulloa`, account `092040680426`, role `AdministratorAccess`, region `us-east-1`. A base `.envrc` at the canonical repo root already exports `AWS_PROFILE=Clarus` and `CLARUS_HOSTED_ZONE_ID=Z02739252KVUU2FWNWPTV` (the `gulloa.click` hosted zone). Both the account and the hosted zone id are known inputs now.
- A duplicate base `.envrc` also currently exists inside the Conductor worktree, and `.envrc` is **not** yet in `.gitignore` (the ignore file covers `.env`/`.env.*` but not `.envrc`) — a commit risk to fix.

Credential classification (from reading both sibling `.envrc` files):

| Value | Shareable? | Source |
|---|---|---|
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `APPLE_SIGNING_IDENTITY` | **Yes** (team `9M9FA9YAWP`, identical in Argus & TokenWatch) | Copy from TokenWatch `.envrc` |
| `TAURI_UPDATER_PRIVATE_KEY`, `TAURI_UPDATER_KEY_PASSWORD` | No (per-app) | Generate for Clarus |
| `RELEASE_S3_BUCKET`, `RELEASE_CLOUDFRONT_DISTRIBUTION_ID`, `AWS_RELEASE_ROLE_ARN` | No (per-app infra) | CDK deploy → SSM |
| `PUBLIC_URL_BASE`, `AWS_REGION`, `AWS_PROFILE` | No (per-app) | Set to Clarus values |

## Goals / Non-Goals

**Goals:**
- Reuse the shareable Apple credentials from the sibling apps, load them into a local `.envrc` at the canonical (non-worktree) repo root, and mirror them plus the per-app secrets to `gsulloa/clarus` GitHub secrets.
- Bring Clarus's release pipeline to parity with TokenWatch: generate an updater keypair, fix the CI cert import, publish infra SSM params.
- Leave a repeatable helper script and README notes so a future release is a documented, reproducible flow.

**Non-Goals:**
- Rotating the shared Apple certificate/password or issuing per-app Apple credentials (accepted reuse).
- Building a feedback intake stack / `*_FEEDBACK_APP_KEY` (Argus/TokenWatch have one; Clarus's `tauri.conf.json` has no feedback surface — out of scope unless later added).
- Actually cutting a `v*` release tag. This change makes release *possible*; running it is a follow-up.
- Provisioning the AWS account itself (creating an SSO/profile named `Clarus`) beyond documenting the decision.

## Decisions

### D1. Copy Apple credentials verbatim rather than issue new ones
A "Developer ID Application" certificate authenticates the *team*, so all three apps legitimately share it. Argus and TokenWatch already prove the values work. **Alternative considered:** a separate cert per app — rejected: more keychain/notarization management for zero security gain, since they'd all chain to the same Team ID anyway.

### D2. `.envrc` at `/Users/gabrielulloa/dev/freelance/clarus`, not the worktree
The user works across many Conductor worktrees; a worktree `.envrc` would be ephemeral and per-branch. The canonical clone is the stable home, matching where Argus/TokenWatch keep theirs. The file is gitignored (`.env*` is already ignored, and `.envrc` will be added explicitly). **Alternative:** commit an `.envrc.example` template — keep as a documentation nicety, but the real secret-bearing file stays local only.

### D3. Model the `.envrc` on TokenWatch, not Argus
TokenWatch's `.envrc` is the newer/cleaner template: it sets `AWS_REGION` explicitly and resolves `AWS_RELEASE_ROLE_ARN` from SSM (`/TokenWatch/releases/publish-role-arn`), which Argus's does not. Clarus mirrors that shape with `Clarus`-namespaced SSM paths and `PUBLIC_URL_BASE=https://releases.clarus.gulloa.click`.

### D4. Publish infra values via SSM parameters
To let the `.envrc` resolve infra dynamically (the sibling pattern), `ReleasesStack` must write `/Clarus/releases/bucket-name`, `/Clarus/releases/distribution-id`, and `/Clarus/releases/publish-role-arn` as SSM `StringParameter`s alongside the existing CfnOutputs. **Alternative:** parse `aws cloudformation describe-stacks` outputs in `.envrc` — rejected: slower, brittle to stack renames, diverges from the sibling pattern.

### D5. Reuse TokenWatch's `set-updater-keys.sh` almost verbatim
Port the script to Clarus (`UPDATER_REPO=gsulloa/clarus`, key path `~/.tauri/clarus-updater.key`, `pnpm --filter clarus`). It generates/rotates the keypair, writes `plugins.updater.pubkey` into `tauri.conf.json`, and sets the two GitHub secrets — one command, reproducible. The empty `.context/clarus-updater.key` files are deleted (README updated to stop referencing them).

### D6. Fix `release.yml` cert import by matching TokenWatch
Add the `APPLE_CERTIFICATE`/`APPLE_CERTIFICATE_PASSWORD` env (and, if TokenWatch uses an explicit `security import` keychain step, replicate it) to the `Build signed bundle` job so macOS notarization succeeds. This is the one repo-committed code change that unblocks CI signing.

### D7. GitHub secrets set via `gh secret set` scripted, not by hand
All secrets are set programmatically (readable from the local `.envrc` + SSM + generated keypair) so the operation is auditable and repeatable, and so a reviewer can see exactly which names are populated.

## Risks / Trade-offs

- **Shared Apple app-specific password across 3 apps** → if leaked, revoke one app-specific password in appleid.apple.com and re-issue for all three; documented in README/SECURITY. Accepted for a solo-dev portfolio of apps.
- **Secrets printed into a local file / passed through shell** → `.envrc` is gitignored and lives outside any repo that gets pushed; `gh secret set` reads from stdin/file, never echoed. Never paste secret values into commits, PR descriptions, or this change's artifacts.
- **Worktree `.envrc` + `.envrc` not gitignored** → the secret-bearing file must only live at the canonical root; the duplicate in the Conductor worktree should be removed and `.envrc` added to `.gitignore` before secrets are written, or credentials could be committed.
- **Losing the updater private key breaks autoupdate** for already-installed clients → key stored in `~/.tauri/clarus-updater.key` + GitHub secret; README warns backup is mandatory and rotation invalidates installed clients.
- **`tauri.conf.json` already contains a pubkey** that may not match any real private key → regenerating the keypair overwrites it; the first signed release must be built *after* the pubkey is updated, or autoupdate signature verification will fail.

## Migration Plan

1. Copy shareable Apple values into a new local `.envrc` at the canonical repo root; set Clarus-specific `AWS_PROFILE`, `AWS_REGION`, `PUBLIC_URL_BASE`.
2. Port `set-updater-keys.sh`; run it to generate the keypair, update `tauri.conf.json` pubkey, and set the two updater GitHub secrets.
3. Add SSM parameters to `ReleasesStack`; resolve the AWS account/profile (O1); `cdk deploy` DNS + Releases (+ Landing) stacks; capture bucket/distribution/role.
4. Fix `release.yml` cert import.
5. Set all shareable + AWS secrets on `gsulloa/clarus` via `gh secret set` (values read from `.envrc`/SSM).
6. Verify: `gh secret list` shows the full set; `direnv allow` loads `.envrc` cleanly; a `--dry-run` release and/or a prerelease tag confirms a signed bundle.

**Rollback:** delete the local `.envrc`, `gh secret delete` the added names, revert the `release.yml`/`ReleasesStack`/`tauri.conf.json` commits, `cdk destroy` the Clarus stacks. No user-facing impact until a real tag is pushed.

## Open Questions

- **O1 (RESOLVED)**: Clarus infra runs under the `Clarus` SSO profile (account `092040680426`, session `gulloa`, `us-east-1`). Hosted zone id `Z02739252KVUU2FWNWPTV` is exported as `CLARUS_HOSTED_ZONE_ID` in the base `.envrc` and passed via `--context hostedZoneId=$CLARUS_HOSTED_ZONE_ID`. Prerequisite: verify the account actually owns the `gulloa.click` hosted zone `Z02739252KVUU2FWNWPTV` (it may be a cross-account zone shared with TokenWatch) before `cdk deploy`.
- **O2**: Does Clarus want a feedback intake key like the siblings? Currently no feedback surface in `tauri.conf.json`; excluded unless the user wants it.
- **O3**: Should an `.envrc.example` (non-secret template) be committed for onboarding, or is the local-only file sufficient?
