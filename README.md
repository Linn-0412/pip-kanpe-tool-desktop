# PiP Kanpe Tool Desktop

PiP Kanpe Tool のデスクトップアプリ版リポジトリです。

このリポジトリは Web 版をベースに、Tauri などのデスクトップアプリ化、アプリ内アップデーター、デスクトップ向け配布を検討するために分離しています。

## 現在の状態

- Web 版の主要ファイルを初期ベースとしてコピー
- ブラウザ拡張機能と GitHub Pages 用の構成は未追加
- Tauri v2 の最小構成を追加
- IndexedDB / localStorage の代わりに、Tauri ではアプリデータ配下の JSON ストアへ保存
- 初期バージョンは `0.1.0`

## 開発コマンド

```bash
npm run check
npm test
npm run tauri:dev
npm run tauri:build
```

Rust / Cargo が必要です。未導入の場合は Tauri の前提条件を入れてから `npm run tauri:dev` を実行してください。

## 保存方式

Web 版では画像を IndexedDB、設定を localStorage に保存していました。

デスクトップ版では Tauri の Rust コマンドを使い、次のファイルへ保存します。

```text
<app data dir>/kanpe-store.json
```

保存内容:

- 設定
- グループ
- 登録画像メタデータ
- 登録画像本体の DataURL

まずは実装と移行のしやすさを優先して JSON DB としています。画像数や容量が増えて問題が出た場合は、画像本体を個別ファイル化するか SQLite へ移行します。

## アップデーター設計

Tauri v2 の updater plugin を使い、GitHub Releases に置いた JSON を見に行く設計です。

stable:

```text
https://github.com/Linn-0412/pip-kanpe-tool-desktop/releases/latest/download/latest.json
```

beta:

```text
https://github.com/Linn-0412/pip-kanpe-tool-desktop/releases/latest/download/latest-beta.json
```

Tauri のアップデーターは署名検証が必須です。`src-tauri/tauri.conf.json` の `TAURI_UPDATER_PUBLIC_KEY_PLACEHOLDER` は、リリース前に実際の公開鍵へ差し替えてください。

鍵生成:

```bash
npm run tauri signer generate -- -w ~/.tauri/pip-kanpe-tool-desktop.key
```

Windows PowerShell でビルド時に秘密鍵を指定:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY="C:\Users\<user>\.tauri\pip-kanpe-tool-desktop.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
npm run tauri:build
```

beta チャンネルをビルドする場合:

```powershell
$env:PIP_KANPE_UPDATE_CHANNEL="beta"
npm run tauri:build:beta
```

## 次にやること

- Rust / Cargo を導入して `npm run tauri:dev` を確認
- 実際の updater 署名鍵を生成し、公開鍵を `tauri.conf.json` に設定
- GitHub Actions で Windows 向けビルドと Release asset アップロードを自動化
- アプリ内に更新確認ボタンまたは起動時更新通知を追加
