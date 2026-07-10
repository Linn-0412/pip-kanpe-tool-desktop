# デスクトップ実装仕様

## English Summary

The desktop app is built with Tauri v2. The main window manages images, groups, settings, the kanpe editor, update checks, and shortcut settings. A separate always-on-top `pip` window renders the current image and receives state snapshots from the main window.

Core implementation points:

- Frontend files are copied into `dist/` for the Tauri build.
- `app.js` owns the main application state.
- `pip-window.js` only renders payloads sent from the main window and requests a snapshot on startup.
- Tauri commands handle persistence, PiP window management, global shortcuts, updater actions, and opening external links.
- Global shortcut events are sent back to the main window as navigation events.
- Localized strings used by the PiP window are included in the payload sent from the main window.

## 構成

- フロントエンド:
  - `index.html`
  - `app.js`
  - `core.js`
  - `i18n.js`
  - `styles.css`
  - `pip.html`
  - `pip-window.js`
- Tauri:
  - `src-tauri/src/lib.rs`
  - `src-tauri/tauri.conf.json`
  - `src-tauri/tauri.beta.conf.json`
- ビルド出力:
  - `dist/`

## ウィンドウ

### メインウィンドウ

- ラベル: `main`
- タイトル: `PiP カンペツール Desktop`
- 初期サイズ: 1280 x 820
- 最小サイズ: 960 x 640
- 主な役割:
  - 画像管理
  - グループ管理
  - PiP表示設定
  - カンペ作成
  - 更新確認
  - ショートカット設定

### PiP小窓

- ラベル: `pip`
- HTML: `pip.html`
- JS: `pip-window.js`
- 常に手前表示: 有効
- リサイズ: 可
- 最小サイズ: 320 x 180
- サイズ上限:
  - 幅: 1280
  - 高さ: 720
- 主な役割:
  - 現在のカンペ画像表示
  - 前/次ボタン操作
  - メイン画面から受け取った状態の描画

## Tauriコマンド

| コマンド | 目的 |
| --- | --- |
| `get_desktop_info` | 更新チャンネルと保存ファイルパスを返す |
| `get_update_channel` | stable / beta のチャンネル名を返す |
| `load_store` | `kanpe-store.json` を読み込む |
| `save_store` | `kanpe-store.json` へ保存する |
| `open_pip_window` | PiP小窓を作成または表示する |
| `resize_pip_window` | 既存PiP小窓のサイズを変更する |
| `set_pip_window_decorations` | PiP小窓のタイトルバー表示を切り替える |
| `close_pip_window` | PiP小窓を閉じる |
| `update_pip_window` | PiP小窓へ現在状態を送って再描画する |
| `request_pip_snapshot` | PiP小窓からメイン画面へ現在状態の送信を要求する |
| `step_pip_card` | メイン画面へ前後切り替えイベントを送る |
| `get_shortcut_info` | 現在のショートカットと登録状態を返す |
| `set_navigation_shortcuts` | 前/次ショートカットを登録し直す |
| `check_update` | updater pluginで更新確認する |
| `install_update` | 更新をダウンロードしてインストールする |
| `open_support_url` | OFUSEページを外部ブラウザで開く |

## PiP同期フロー

1. メイン画面で `PiPで表示` を押します。
2. `app.js` の `openDesktopPip()` が `open_pip_window` を呼びます。
3. Tauri側が `pip` ウィンドウを作成または再表示します。
4. `pip-window.js` は起動時に `request_pip_snapshot` を呼びます。
5. メイン画面は `pip:request-snapshot` を受け、現在カードと設定を `update_pip_window` で送ります。
6. 以降、画像切り替えや設定変更時に `syncDesktopPipWindow()` で小窓を更新します。

## ローカライズ同期

- メイン画面の表示言語は `i18n.js` の辞書で管理します。
- `app.js` は現在言語に合わせてDOM文言を更新します。
- PiP小窓は独立したwindowなので、必要な文言をpayloadの `strings` として受け取ります。
- PiP小窓側はpayload内の `strings` を使い、タイトル、前/次ボタン説明、閉じるボタン説明、空表示を更新します。

## 前後切り替えフロー

- メイン画面:
  - プレビュー矢印
  - `ArrowLeft` / `ArrowRight`
- PiP小窓:
  - 前/次ボタン
  - 縦ライン当たり判定
- グローバルショートカット:
  - `Ctrl+F5` / `Ctrl+F6` またはユーザー設定値

すべて最終的にはメイン画面の現在位置を更新し、`render()` 経由でプレビューとPiP小窓を同期します。

## グローバルショートカット

- 使用プラグイン: `tauri-plugin-global-shortcut`
- 初期値:
  - 前: `Ctrl+F5`
  - 次: `Ctrl+F6`
- 起動時に初期値を登録します。
- ユーザーが設定を変更した場合:
  1. 既存ショートカットを解除
  2. 新しい前ショートカットを登録
  3. 新しい次ショートカットを登録
  4. 失敗時は可能な範囲で元の登録へ戻す
- ショートカット発火時はメインウィンドウへ `pip:navigate` イベントを送ります。

## 保存

- Tauri側は `kanpe-store.json` の読み書きだけを担当します。
- フロントエンド側がカード、設定、画像DataURLを組み立てて `save_store` へ渡します。
- 保存時は一時ファイルへ書き出してから置き換えます。

## 更新

- 使用プラグイン: `tauri-plugin-updater`
- stable endpoint:
  - `https://github.com/Linn-0412/pip-kanpe-tool-desktop/releases/latest/download/latest.json`
- beta endpoint:
  - `https://github.com/Linn-0412/pip-kanpe-tool-desktop/releases/download/beta/latest.json`
- Windowsのインストールモードは `passive` です。
- updater署名はTauriの秘密鍵/公開鍵で検証します。

## 外部リンク

- OFUSEリンクは `tauri-plugin-opener` で外部ブラウザへ開きます。
- 失敗時はブラウザ標準の `window.open` にフォールバックします。

## Capabilities

`src-tauri/capabilities/default.json` で `main` と `pip` の両方に必要な権限を付与します。

主な権限:

- updater確認
- updaterダウンロード/インストール
- 必要なTauriコアAPI

権限を増やす場合は、用途を明確にしてから追加します。
