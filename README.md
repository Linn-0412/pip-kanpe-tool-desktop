# PiP Kanpe Tool Desktop

PiP Kanpe Tool のデスクトップアプリ版リポジトリです。

このリポジトリは Web 版をベースに、Tauri などのデスクトップアプリ化、アプリ内アップデーター、デスクトップ向け配布を検討するために分離しています。

## 現在の状態

- Web 版の主要ファイルを初期ベースとしてコピー
- Tauri v2 の最小構成を追加
- IndexedDB / localStorage の代わりに、Tauri ではアプリデータ配下の JSON ストアへ保存
- メイン画面から常に手前表示の Tauri 小窓を開き、現在のカンペ画像を同期
- Tauri の global-shortcut plugin で `Ctrl+F5` / `Ctrl+F6` による前後切り替えに対応
- 画面上でデスクトップ版グローバルショートカットを変更可能
- 起動時とボタン操作でアプリ内更新チェックを実行し、手動操作で更新適用へ進める
- GitHub Actions で CI と Windows 向け Release asset 作成を実行
- 初回 stable Release `v0.1.0` を作成済み

## 開発コマンド

```bash
npm run check
npm test
npm run tauri:dev
npm run tauri:build
npm run tauri:build:beta
```

Rust / Cargo が必要です。未導入の場合は Tauri の前提条件を入れてから `npm run tauri:dev` を実行してください。

## 仕様書

今後はデスクトップ版を主系統として、詳細仕様を [docs/README.md](./docs/README.md) に集約します。

Web版から継承した機能も、デスクトップ版で正式採用したものはこの仕様書を正として扱います。

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
https://github.com/Linn-0412/pip-kanpe-tool-desktop/releases/download/beta/latest.json
```

Tauri のアップデーターは署名検証が必須です。`src-tauri/tauri.conf.json` には公開鍵だけを設定し、秘密鍵はリポジトリに含めません。

鍵生成:

```bash
npm run tauri -- signer generate --ci -p "<password>" -w ~/.tauri/pip-kanpe-tool-desktop.key
```

Windows PowerShell でビルド時に秘密鍵を指定:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = Get-Content "C:\Users\<user>\.tauri\pip-kanpe-tool-desktop.key" -Raw
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<password>"
npm run tauri:build
```

beta チャンネルをビルドする場合:

```powershell
$env:PIP_KANPE_UPDATE_CHANNEL="beta"
npm run tauri:build:beta
```

GitHub Actions では次の Secrets を使って署名します。

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

## リリース運用

通常リリース:

1. `package.json` と `src-tauri/tauri.conf.json` のバージョンを更新
2. `main` に push して CI を通す
3. GitHub Actions の `Release Desktop` を `stable` で実行
4. Release に `.msi` / `.exe` / `latest.json` / 署名ファイルが追加されたことを確認

βリリース:

1. GitHub Actions の `Release Desktop` を `beta` で実行
2. 固定タグ `beta` の prerelease に成果物と `latest.json` をアップロード
3. β版アプリは `src-tauri/tauri.beta.conf.json` の endpoint から更新確認

CI は `main` への push、pull request、手動実行で `npm run check`、`npm test`、`cargo check` を実行します。

## Windows コード署名の判断

現時点では、Windows のコード署名証明書はまだ導入しません。

理由:

- Tauri updater の署名は設定済みで、更新ファイルの改ざん検証はできます。
- Windows コード署名は SmartScreen 警告の軽減に役立ちますが、Microsoft の現在の説明では EV 証明書でも警告を必ず回避できるわけではありません。
- Microsoft は Store 配布、または Artifact Signing / Trusted Signing を非 Store 配布向けの選択肢として案内しています。
- まずは GitHub Releases で配布し、利用者数や問い合わせが増えた段階で導入判断するほうが費用対効果が高いです。

導入を検討する目安:

- GitHub Releases からのダウンロード数が増え、SmartScreen で離脱している報告が出る
- 支援や寄付で月額の運用費をまかなえる
- Microsoft Store 公開または企業・固定向け配布を考える

候補:

- Microsoft Store: SmartScreen 警告を避けやすいが、ストア公開手続きが必要
- Azure Artifact Signing / Trusted Signing: CI/CD に組み込みやすいが、本人確認と月額費用が必要
- OV/EV 証明書: 署名者名を表示できるが、SmartScreen reputation は別途積み上げが必要

参考:

- Tauri Windows Code Signing: https://v2.tauri.app/distribute/sign/windows/
- Microsoft SmartScreen reputation: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/smartscreen-reputation

## 次にやること

- `更新を確認` ボタンからの更新確認は確認済み。次バージョンの Release 後に、旧版インストール済み環境で起動時の自動更新確認を実機検証
- JSON ストアの容量が重くなった場合に、画像ファイル分離または SQLite 移行を検討
