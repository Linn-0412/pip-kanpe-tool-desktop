# PiP Kanpe Tool Desktop

PiP Kanpe Tool のデスクトップアプリ版リポジトリです。

このリポジトリは Web 版をベースに、Tauri などのデスクトップアプリ化、アプリ内アップデーター、デスクトップ向け配布を検討するために分離しています。

## 現在の状態

- Web 版の主要ファイルを初期ベースとしてコピー
- ブラウザ拡張機能と GitHub Pages 用の構成は未追加
- Tauri の scaffold は未実施
- 初期バージョンは `0.1.0`

## 開発コマンド

```bash
npm run check
npm test
```

## 次にやること

- Tauri の導入
- デスクトップ版として必要な画面・保存方式の見直し
- GitHub Releases を使ったアプリ内アップデーター設計
- stable / beta の更新チャンネル設計
