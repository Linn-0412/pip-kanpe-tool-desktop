# PiP Kanpe Tool Desktop 仕様書

この `docs/` はデスクトップ版を主系統として管理するための仕様書置き場です。

Web版から継承したカンペ管理・カンペ作成機能も、今後はデスクトップ版の仕様としてここに集約します。Web版へ戻す、またはWeb版へ移植する場合は、この仕様を基準に差分を確認します。

## English Index

This `docs/` directory is the specification home for PiP Kanpe Tool Desktop.

The desktop app is now treated as the main product line. Features inherited from the web version are documented here once they are officially adopted by the desktop version. If a feature is ported back to the web version, compare it against these desktop specifications.

Japanese is currently the primary source of truth. Each spec file also includes an English summary so non-Japanese contributors can understand the expected behavior, data format, release process, and test coverage.

## 仕様書一覧

- [機能仕様](./functional-spec.md)
  - ユーザー向け機能、画面、操作仕様
- [デスクトップ実装仕様](./desktop-spec.md)
  - Tauri小窓、グローバルショートカット、更新確認などの実装仕様
- [データ仕様](./data-spec.md)
  - `kanpe-store.json` と `.pipkanpe` の保存形式
- [カンペ作成仕様](./maker-spec.md)
  - カンペ作成画面、素材、キャンバス、出力仕様
- [リリース・更新仕様](./release-spec.md)
  - stable / beta、GitHub Releases、updater、署名鍵の扱い
- [テスト仕様](./test-spec.md)
  - CIで確認する内容と、リリース前の手動確認項目

## Specification Files

- [Functional Spec](./functional-spec.md)
  - User-facing features, screens, and interaction rules
- [Desktop Implementation Spec](./desktop-spec.md)
  - Tauri subwindow, global shortcuts, update checks, and native commands
- [Data Spec](./data-spec.md)
  - Storage format for `kanpe-store.json` and `.pipkanpe`
- [Kanpe Editor Spec](./maker-spec.md)
  - In-app diagram editor, assets, canvas behavior, and output
- [Release and Update Spec](./release-spec.md)
  - stable/beta channels, GitHub Releases, updater metadata, and signing keys
- [Test Spec](./test-spec.md)
  - CI checks and manual release checklist

## 管理方針

- デスクトップ版で正式採用した仕様は、このフォルダを正とします。
- READMEは利用者・開発者の入口に留め、詳細仕様は `docs/` に分離します。
- Web版から持ってきた機能は「Web由来」として扱いますが、デスクトップ版で変更した場合はデスクトップ版の挙動を優先します。
- 仕様変更を伴う実装では、可能な範囲で該当仕様書も同時に更新します。

## Maintenance Policy

- Desktop behavior is the source of truth for features officially adopted in this app.
- The root README should stay as an entry point for users and developers; detailed behavior belongs in `docs/`.
- Features inherited from the web version may still be described as web-derived, but desktop behavior takes priority when the two differ.
- When implementation changes affect behavior, update the relevant spec file in the same change whenever possible.
