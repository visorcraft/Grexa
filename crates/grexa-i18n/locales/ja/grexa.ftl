# 日本語翻訳。キー集合は /locales/en/grexa.ftl と完全一致が必須。
# `scripts/check_locale_sync.py` が CI で同期を強制する。

app-name = Grexa
app-tagline = 高速 Linux ファイル内容検索

search-status-ready = 準備完了
search-status-running = 検索中…
search-status-cancelled = キャンセルされました
search-status-error = エラー: {$message}

# 日本語には複数形の文法的区別がないため、selector を使わず単一形にする。
search-status-found = {$files} 件のファイルから {$matches} 件のマッチを {$elapsed} で発見しました
search-status-filtered = {$total} 件中 {$shown} 件のマッチを {$files} 件のファイルで表示中

elapsed-subsecond = 1秒未満
elapsed-seconds = {$seconds} 秒
elapsed-minutes-only = {$minutes} 分
elapsed-minutes-and-seconds = {$minutes} 分 {$seconds} 秒

replace-status-running = 置換中…
replace-status-completed = {$files} 件のファイル内で {$matches} 件のマッチを置換しました（{$elapsed}）
replace-confirm-title = 置換の確認
replace-confirm-message = {$files} 件のファイル内で {$matches} 件のマッチを置換しますか？元に戻すことはできません。

container-target-local = ローカルファイル
container-target-docker = Docker コンテナ
container-target-podman = Podman コンテナ
container-mirror-fallback-badge = （ミラー）

ai-disabled-banner = AI 検索はオフです。設定 → AI 検索でオンにしてください。
ai-empty-state = AI ボタンをクリックして AI 支援検索を開始します。
ai-error-not-configured = AI エンドポイントが設定されていません。
ai-error-empty-response = AI エンドポイントが空の応答を返しました。

action-open-in-editor = エディターで開く
action-reveal-in-file-manager = ファイルマネージャーで表示
action-copy-path = パスをコピー

# 日本語には複数形の文法的区別がないため、selector を使わず単一形にする。
count-matches = {$count} 件のマッチ
count-files = {$count} 件のファイル
count-files-modified = {$count} 件のファイルを変更
count-matches-replaced = {$count} 件のマッチを置換
count-failures = {$count} 件の失敗
