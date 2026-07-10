# データ仕様

## 保存方針

デスクトップ版では、Web版のIndexedDB / localStorageの代わりに、アプリデータ配下のJSONファイルへ保存します。

保存ファイル:

```text
<app data dir>/kanpe-store.json
```

画像本体はDataURLとしてJSON内に保存します。実装と移行のしやすさを優先した形式です。画像数や容量が増えて問題が出た場合は、画像本体の個別ファイル化またはSQLite移行を検討します。

## `kanpe-store.json`

### ルート

```json
{
  "version": 1,
  "settings": {},
  "cards": []
}
```

| フィールド | 型 | 内容 |
| --- | --- | --- |
| `version` | number | デスクトップ保存形式のバージョン。現在は `1` |
| `settings` | object | 画面設定、PiP設定、グループ設定など |
| `cards` | array | 登録画像の配列 |

### card

```json
{
  "id": "uuid",
  "name": "phase1.png",
  "type": "image/png",
  "size": 123456,
  "originalSize": 234567,
  "order": 0,
  "hidden": false,
  "groupIds": ["group-id"],
  "createdAt": 1760000000000,
  "dataUrl": "data:image/png;base64,..."
}
```

| フィールド | 型 | 内容 |
| --- | --- | --- |
| `id` | string | アプリ内識別子。UUID |
| `name` | string | 表示名 |
| `type` | string | MIME type |
| `size` | number | 保存後サイズ |
| `originalSize` | number | 元ファイルサイズ |
| `order` | number | 並び順 |
| `hidden` | boolean | プレビュー/PiPで非表示にするか |
| `groupIds` | string[] | 所属グループID |
| `createdAt` | number | 作成日時。Unix time milliseconds |
| `dataUrl` | string | 画像本体 |

## settings

主な設定値:

| キー | 内容 | 例 |
| --- | --- | --- |
| `fitMode` | 画像表示方法 | `contain`, `cover` |
| `pipSize` | PiPサイズプリセット | `640x360` |
| `pipSizeMode` | プリセット/カスタム | `preset`, `custom` |
| `pipCustomWidth` | カスタム幅 | `640` |
| `pipCustomHeight` | カスタム高さ | `360` |
| `pipControlsSize` | 操作UIサイズ | `small`, `medium`, `large` |
| `pipControlsPlacement` | 矢印配置 | `horizontal`, `vertical-left`, `vertical-right` |
| `pipControlsFullHeightButtons` | 縦ライン全体を当たり判定にするか | `true`, `false` |
| `pipControlsPosition` | 操作UI位置 | `top`, `middle`, `bottom` |
| `pipControlsBackground` | 操作UI背景 | `solid`, `translucent`, `clear` |
| `pipControlsSeparateFromImage` | 操作UIを画像に重ねないか | `true`, `false` |
| `pipControlsAutoHide` | マウス操作時だけ表示するか | `true`, `false` |
| `pipHideTitleBar` | PiPタイトルバーを隠すか | `true`, `false` |
| `showPipLabel` | 画像名と枚数を表示するか | `true`, `false` |
| `showFileExtension` | 拡張子を表示するか | `true`, `false` |
| `optimizeImages` | 保存時に画像を軽量化するか | `true`, `false` |
| `shortcutPrevious` | 前のカンペショートカット | `Ctrl+F5` |
| `shortcutNext` | 次のカンペショートカット | `Ctrl+F6` |
| `hideGuideOnLaunch` | 起動時ガイドを非表示にするか | `true`, `false` |
| `activeGroupId` | 選択中グループID | `all` |
| `groups` | グループ一覧 | `[{ "id": "...", "name": "..." }]` |

## `.pipkanpe`

カンペセットのエクスポート/インポート用ファイルです。JSON形式ですが、利用者向けには `.pipkanpe` 拡張子で扱います。

### ルート

```json
{
  "format": "pip-kanpe-tool.deck",
  "version": 1,
  "exportedAt": "2026-07-10T00:00:00.000Z",
  "settings": {},
  "groups": [],
  "cards": []
}
```

| フィールド | 型 | 内容 |
| --- | --- | --- |
| `format` | string | 固定値 `pip-kanpe-tool.deck` |
| `version` | number | デッキ形式バージョン。現在は `1` |
| `exportedAt` | string | ISO 8601形式の出力日時 |
| `settings` | object | 共有対象のPiP表示設定 |
| `groups` | array | グループ一覧 |
| `cards` | array | 画像一覧 |

### エクスポート対象の設定

`.pipkanpe` には次の設定を含めます。

- `fitMode`
- `pipSize`
- `pipSizeMode`
- `pipCustomWidth`
- `pipCustomHeight`
- `pipControlsSize`
- `pipControlsPlacement`
- `pipControlsFullHeightButtons`
- `pipControlsPosition`
- `pipControlsBackground`
- `pipControlsSeparateFromImage`
- `pipControlsAutoHide`
- `pipHideTitleBar`
- `showPipLabel`
- `showFileExtension`

### インポート時の扱い

- `format` が一致しない場合は読み込みません。
- `version` が現在対応バージョンより新しい場合は読み込みません。
- `cards` が不正、または画像DataURLでない場合は読み込みません。
- 登録済み枚数と合わせて80枚を超える場合は読み込みません。
- グループIDはインポート先で再割り当てします。
- 同名グループがある場合は既存グループに統合します。

## 後方互換

- `groupIds` は旧データの単一文字列も受け取り、配列へ正規化します。
- 不明なPiP表示設定値は、既定値へフォールバックします。
- `order` がないカードは読み込み順を並び順として扱います。
