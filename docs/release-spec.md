# リリース・更新仕様

## English Summary

The desktop app uses two update channels:

- `stable`
  - Normal user release.
  - Git tag format: `vX.Y.Z`.
  - Uses the latest GitHub Release updater JSON.
- `beta`
  - Pre-release channel for supporters and testers.
  - Uses the fixed `beta` prerelease tag.

Release metadata must keep the same version across `package.json`, `package-lock.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, and cache-busting query strings in `index.html`/`pip.html`. Use `npm run release:check` before release.

Tauri updater signing is required. Public keys are stored in config files, but private keys must never be committed. GitHub Actions receives signing keys through repository secrets.

## チャンネル

### stable

- 通常利用者向けの正式版です。
- GitHubタグは `vX.Y.Z` 形式です。
- updater endpoint:

```text
https://github.com/Linn-0412/pip-kanpe-tool-desktop/releases/latest/download/latest.json
```

### beta

- 支援者・テスター向けの事前確認版です。
- 固定タグ `beta` のprereleaseへ配布します。
- updater endpoint:

```text
https://github.com/Linn-0412/pip-kanpe-tool-desktop/releases/download/beta/latest.json
```

## バージョン

次のファイルのバージョンを一致させます。

- `package.json`
- `package-lock.json`
- `src-tauri/tauri.conf.json`
- `index.html` のCSS/JSクエリ
- `pip.html` のCSS/JSクエリ

`npm run release:check` で不一致を検出します。

## GitHub Actions

### CI

対象:

- `main` へのpush
- `main` 向けpull request
- 手動実行

主な確認:

- `npm ci`
- `npm run release:check`
- `npm run build:web`
- `npm run check`
- `npm test`
- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo clippy -D warnings`

### Release Desktop

対象:

- `v*` タグpush
- 手動実行

stable:

- `vX.Y.Z` タグから正式Releaseを作成します。
- Windows向けインストーラーとupdater用 `latest.json` をアップロードします。

beta:

- workflow_dispatchで `channel = beta` を選択します。
- `src-tauri/tauri.beta.conf.json` を使ってビルドします。
- 固定タグ `beta` のprereleaseへ成果物をアップロードします。

## 署名鍵

Tauri updaterは署名検証が必須です。

- 公開鍵:
  - `src-tauri/tauri.conf.json`
  - `src-tauri/tauri.beta.conf.json`
- 秘密鍵:
  - リポジトリへ含めません。
  - GitHub Actions Secretsで管理します。

必要なSecrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

ローカルビルド時は環境変数で秘密鍵を渡します。

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "C:\Users\<user>\.tauri\pip-kanpe-tool-desktop.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<password>"
npm run tauri:build
```

## stableリリース手順

1. 実装を `main` へ反映します。
2. バージョンを更新します。
3. `npm run release:check` を通します。
4. `npm run check` を通します。
5. `npm test` を通します。
6. 必要に応じて `npm run tauri:build` をローカル確認します。
7. 変更をcommitします。
8. `vX.Y.Z` タグを作成してpushします。
9. GitHub Actionsの `Release Desktop` 完了を確認します。
10. Release assetsに次があることを確認します。
    - `.exe`
    - `.exe.sig`
    - `.msi`
    - `.msi.sig`
    - `latest.json`
11. `latest.json` の `version` がリリースバージョンと一致することを確認します。
12. 旧バージョンから `更新を確認` で更新できることを実機確認します。

## betaリリース手順

1. beta向け変更を取り込みます。
2. 必要に応じてバージョンまたは表示をbeta向けに調整します。
3. GitHub Actionsの `Release Desktop` を手動実行します。
4. `channel = beta` を選びます。
5. 固定タグ `beta` のprereleaseが更新されたことを確認します。
6. beta版アプリから更新確認できることを確認します。

## Windowsコード署名

現時点ではTauri updaterの署名を必須とし、Windowsコード署名証明書は未導入です。

導入検討の目安:

- SmartScreen警告による離脱報告が増える
- GitHub Releasesからのダウンロード数が増える
- 支援で証明書費用を継続的にまかなえる
- Microsoft Storeや企業向け配布を検討する

候補:

- Microsoft Store
- Azure Trusted Signing
- OV/EV証明書
