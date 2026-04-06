<p align="center">
  <a href="README.md">English</a> | <a href="README.zh.md">中文</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.hi.md">हिन्दी</a> | <a href="README.it.md">Italiano</a> | <a href="README.pt-BR.md">Português (BR)</a>
</p>

<p align="center">
  <img src="https://raw.githubusercontent.com/mcp-tool-shop-org/brand/main/logos/saints-mile/readme.png" width="400" alt="Saint's Mile" />
</p>

<p align="center">
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/actions"><img src="https://github.com/mcp-tool-shop-org/saints-mile/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/mcp-tool-shop-org/saints-mile/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
  <a href="https://mcp-tool-shop-org.github.io/saints-mile/"><img src="https://img.shields.io/badge/Landing_Page-saints--mile-d97706" alt="Landing Page"></a>
</p>

大人向けの、かつてそのゲームを愛した人々向けの、フロンティアを舞台にしたJRPG。

「セイントスマイル」は、鉄道、水路、そして法によって変貌を遂げているフロンティアの地、シンダー盆地を舞台にした、ターン制のパーティーRPGです。プレイヤーは、彼が到着する前にその名前が町中に広まるギャレン・ルークとなり、誰かが書いた「指名手配犯」リストの下で生きる40年間の人生を体験します。

ターミナル用にRustで開発されました。グラフィックは一切使用していません。決定論的なゲームシステム、パーティー戦闘、そしてプレイヤーを信頼する物語に焦点を当てています。

## このゲームについて

- 4つのキャラクターをパーティーに編成できる、**90年代風のJRPG**。独自の役割、連携技、ターン制バトル。
- 名声が複雑な関係を築き、距離が意思決定を左右し、道がダンジョンとなる**フロンティア西部劇**。
- 後悔、義務、妥協、老い、忠誠心、そして新たな始まりといったテーマを扱う**大人向けのゲーム**。
- どんなターミナルでも動作する**ターミナルネイティブな体験**。[ratatui](https://ratatui.rs/)を使用。

## 物語

このゲームは、ほぼ40年間を舞台にしています。19歳の駆け出し保安官から、他人の罪を背負う若きガンマン、そして、傷ついたスペシャリストたちと共に荒れ果てた土地を旅するアウトロー、そして、生きているうちに償えるかどうかを決めなければならない年老いた男へと、主人公の人生が描かれます。

表面的な対立は、鉄道、水、そして土地を巡るものです。より深い対立は、セイントスマイルで何が起こったのか、その物語を誰が書くのか、ということです。

## 戦闘

重要な戦闘の前に、緊張感のある対峙が発生します。息をのむような緊張感、精神力の試練、そして、主導権を握るための駆け引き。その後、4人のアクティブメンバーで構成されるJRPGのバトルシステムが展開されます。各キャラクターは、独自のコマンドセット、物語や絆を通じて深まるスキル、そして、パーティーへの貢献を報いる連携技を持っています。

西部劇の要素は、単なる雰囲気だけでなく、ゲームシステムにも影響を与えます。MPの代わりに弾薬、士気の代わりに精神力、防御力アップの代わりに「タフネス」、そして、戦闘の間にも残る傷といった要素があります。

## パーティー

| キャラクター | 役割 | 戦闘における役割 |
|-----------|------|----------------|
| **Galen Rook** | ガンハンド | 正確な射撃、精密射撃、戦場での指揮。年齢とともに成長します。 |
| **Eli Winter** | グリフター | 精神攻撃、妨害、卑怯な手。忠誠心は後から解放されます。 |
| **Dr. Ada Mercer** | ソーボンズ | 回復、傷の手当、弱点の発見。 |
| **Rosa Varela** | 牧場の手伝い | ロープを使った敵の拘束、前線での防御、敵へのプレッシャー。 |
| **Rev. Miriam Slate** | 宣教師 | バフ、精神力のサポート、敵の集団への対応。 |
| **Lucien "Fuse" Marr** | ダイナマイター | 遅延爆発、環境破壊、地形の変更。 |

## 開発状況

**v1.0.1 — リリース候補。** フルキャンペーン実装済み（プロローグ + 4つの人生フェーズにわたる15章）。戦闘エンジン、対峙システム、プレッシャーエンカウンター、ステート管理、セーブ/ロード、TUIプレゼンテーション層すべて動作中。151テスト合格。

## セキュリティ

「セイントスマイル」は、シングルプレイのオフラインゲームです。以下の機能は使用しません。
- インターネット接続
- テレメトリや分析データの収集
- 自身のセーブデータフォルダ以外のファイルへのアクセス
- ターミナル入出力以外の権限の要求

セーブデータは、ユーザーがアクセス可能なディレクトリにRON形式で保存されます。

## システム要件

- Rust 1.80+ (2021 edition)
- 256色表示に対応したターミナル

## ライセンス

MIT

---

開発者: <a href="https://mcp-tool-shop.github.io/">MCP Tool Shop</a>
