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

ai-empty-state = AI ボタンをクリックして AI 支援検索を開始します。

ai-error-not-configured = AI エンドポイントが設定されていません。

ai-error-empty-response = AI エンドポイントが空の応答を返しました。

action-reveal-in-file-manager = ファイルマネージャーで表示

action-copy-path = パスをコピー

# 日本語には複数形の文法的区別がないため、selector を使わず単一形にする。

count-matches = {$count} 件のマッチ

count-files = {$count} 件のファイル

count-files-modified = {$count} 件のファイルを変更

count-matches-replaced = {$count} 件のマッチを置換

count-failures = {$count} 件の失敗

flag-whole-word = 単語全体


# --- Placeholder entries copied from English source ---

count-messages = {$count} 件のメッセージ

ui-fast-content-search = Fast content search

ui-streams-matches-as-files-are-scanned-a42cad = Streams matches as files are scanned — no waiting for the whole tree.

ui-regex-builder = Regex builder

ui-test-patterns-against-a-sample-with-ac6cae = Test patterns against a sample with the same engine the search uses.

ui-smart-filters = Smart filters

ui-gitignoreaware-with-perextension-include-perdirectory-exclude-0d4b24 = \.gitignore-aware, with per-extension include + per-directory exclude globs.

ui-optional-ai-assist = Optional AI assist

ui-plug-in-any-openaicompatible-endpoint-keys-931293 = Plug in any OpenAI-compatible endpoint. Keys live in Secret Service.

ui-about = About

ui-built-on-rust-qt-6-kirigami = Built on Rust + Qt 6 / Kirigami via cxx-qt.

ui-fast-linux-file-content-search-built = Fast Linux file content search — built on the ripgrep core.

ui-gpl-v3 = GPL v3

ui-linux-qt-6 = Linux · Qt 6

ui-native-linux-search-app-built-with-0ff6f8 = Native Linux search app built with Rust, Qt 6, Kirigami, and cxx-qt.

ui-a-hrefhttpsgithubcomvisorcraftgrexagithubcomvisorcraftgrexaa-90eda6 = <a href='https://github.com/visorcraft/grexa'>github.com/visorcraft/grexa</a>

ui-visit-grexa = Visit Grexa

ui-official-linux-port-of-our-grex = Official Linux port of our Grex tool for Windows.

ui-a-hrefhttpsgithubcomvisorcraftgrexgithubcomvisorcraftgrexa-0e491d = <a href='https://github.com/visorcraft/grex'>github.com/visorcraft/grex</a>

ui-visit-grex = Visit Grex

ui-licenses-credits = Licenses & Credits

ui-every-direct-transitive-crate-acknowledgments-and-2f6d3e = Every direct + transitive crate, acknowledgments, and full license text is bundled in the built-in licenses view.

ui-licenses = Licenses

ui-credits = Credits

ui-built-by-bvisorcraftb = Built by <b>VisorCraft</b>

ui-powered-by-rust-qt-6-kirigami = Powered by Rust, Qt 6, Kirigami, and cxx-qt

ui-ai-search-is-off-enable-it = AI search is off. Enable it in Settings → AI Search.

ui-clear = Clear

ui-clear-the-chat-panel-doesnt-touch-2a169a = Clear the chat panel. Doesn't touch your API key or stored history.

ui-ask-ai-for-help-shaping-a = Ask AI for help shaping a search

ui-describe-what-youre-looking-for-in-1d0236 = Describe what you're looking for in plain English. The model will suggest a path, term, and flags.

ui-ask-the-ai = Ask the AI…

ui-chat-message = Chat message

ui-line-1-630b65 = line %1

ui-1-cargo-crates-2-runtime-components-4ce163 = %1 Cargo crates - %2 runtime components

ui-runtime-components = Runtime components

ui-system-libraries-grexa-links-against-at-0646c4 = System libraries Grexa links against at execution. None are bundled - downstream packagers handle redistribution.

ui-view-license-text = View license text

ui-open-project-website = Open project website

ui-cargo-crates = CARGO CRATES

ui-filter-by-crate-name-or-license = Filter by crate name or license...

ui-filter-thirdparty-credits = Filter third-party credits

ui-1-2-d4b2ac = %1 / %2

ui-crate = Crate

ui-version = Version

ui-license-expression = License expression

ui-open-crate-project = Open crate project

ui-license-text = License Text

ui-gnu-general-public-license-v3 = GNU General Public License v3

ui-gpl30only-license-text-bundled-with-grexa = GPL-3.0-only license text bundled with Grexa.

ui-no-bundled-license-text-is-available = No bundled license text is available.

ui-history = History

ui-every-completed-search-deduped-on-the = Every completed search, deduped on the seven-field Grex key.

ui-refresh = Refresh

ui-filter-history-by-term-or-path = Filter history by term or path

ui-no-history-entries-match-1-ab0ac1 = No history entries match “%1”

ui-no-search-history-yet = No search history yet

ui-try-a-shorter-filter-or-clear = Try a shorter filter, or clear it to see every saved search.

ui-run-a-search-from-the-search = Run a search from the Search page and it'll land here.

ui-1-234-1fba02 = %1 · %2%3%4

ui-open = Open

ui-forget-this-entry = Forget this entry

ui-thirdparty-licenses = Third-party licenses

ui-acknowledgments = Acknowledgments

ui-grexa-license = Grexa License

ui-the-cargoaboutgenerated-bundle-with-every-direct-d02cc5 = The cargo-about-generated bundle with every direct and transitive Rust crate, grouped by license text.

ui-narrative-attribution-for-grexa-grex-runtime-9cb532 = Narrative attribution for Grexa, Grex, runtime components, and direct dependencies.

ui-full-license-texts-for-the-qt-7c5dad = Full license texts for the Qt, KDE Frameworks, Poppler, container, and secret-service runtimes Grexa builds on.

ui-the-complete-gpl30only-license-text-bundled-237019 = The complete GPL-3.0-only license text bundled into the application.

ui-bundled-license-and-attribution-documents-available-9098e4 = Bundled license and attribution documents, available without opening a browser.

ui-thirdparty = Third-party

ui-copy = Copy

ui-copy-the-current-document = Copy the current document

ui-dialog = Dialog

ui-open-the-gpl-text-in-a = Open the GPL text in a dialog

ui-1-matches-ac30b9 = %1 matches

ui-1-lines-9b1ae5 = %1 lines

ui-find-by-crate-package-license-or = Find by crate, package, license, or phrase...

ui-find-in-license-document = Find in license document

ui-wrap = Wrap

ui-open-sidebar = Open Sidebar

ui-close-sidebar = Close Sidebar

ui-fast-file-search = Fast file search

ui-search = Search

ui-profiles = Profiles

ui-settings = Settings

ui-interrupted-replace-from-a-previous-run = Interrupted replace from a previous run

ui-grexa-found-a-residual-replace-journal-1e9ebe = Grexa found a residual replace journal at $XDG_STATE_HOME/grexa/replace-journal.json. The previous run rewrote some files before being interrupted.

ui-click-discard-to-remove-the-journal-b1485d = Click Discard to remove the journal, or Close to keep it for forensic review. The file is a JSON document you can inspect by hand.

ui-named-search-presets-the-search-pages-5a905c = Named search presets. The Search page's “Save current as profile…” captures the active form here.

ui-filter-profiles-by-name-term-or = Filter profiles by name, term, or path

ui-no-profiles-match-1-275b92 = No profiles match “%1”

ui-no-saved-profiles = No saved profiles

ui-try-a-shorter-filter-or-clear-76584b = Try a shorter filter, or clear it to see every saved profile.

ui-open-the-search-page-fill-in-2bc930 = Open the Search page, fill in path + term + flags, then save the form as a named profile.

ui-1-2345-67b02f = %1 · “%2”%3%4%5

ui-delete-profile = Delete profile

ui-email = Email

ui-phone = Phone

ui-date = Date

ui-digits = Digits

ui-ipv4 = IPv4

ui-hex = Hex

ui-test-patterns-against-sample-text-same-7198ac = Test patterns against sample text — same engine the search uses.

ui-invalid = invalid

ui-1-matches-2749a8 = %1 match(es)

ui-presets = Presets

ui-pattern = Pattern

ui-regex-pattern = Regex pattern

ui-caseinsensitive = Case-insensitive

ui-paste-sample-text-and-watch-the = Paste sample text and watch the matches light up…

ui-test-string = Test string

ui-matches = MATCHES

ui-invalid-pattern = Invalid pattern

ui-enter-a-pattern = Enter a pattern

ui-add-sample-text = Add sample text

ui-no-matches = No matches

ui-preview = Preview

ui-open-in-editor = Open in editor

ui-reveal-in-file-manager = Reveal in file manager

ui-move-to-trash = Move to Trash

ui-copy-full-path = Copy full path

ui-copy-file-name = Copy file name

ui-copy-relative-path = Copy relative path

ui-copy-line-content = Copy line content

ui-copy-12-15141f = Copy %1:%2

ui-search-path = Search path

ui-forget-this-path = Forget this path

ui-browse-for-a-folder = Browse for a folder

ui-search-code-configs-anything = Search code, configs, anything…

ui-search-term = Search term

ui-regex = Regex

ui-casesensitive = Case-sensitive

ui-whole-word = Whole word

ui-searching = Searching

ui-search-tab = Search %1

ui-new-search-tab-ctrlt = New search tab (Ctrl+T)

ui-local-files = Local files

ui-docker = Docker

ui-podman-rootless = Podman rootless

ui-podman-rootful = Podman rootful

ui-container = Container

ui-content = Content

ui-files = Files

ui-filters = Filters

ui-save-profile = Save profile…

ui-export = Export…

ui-export-as-csv = Export as CSV…

ui-export-as-json = Export as JSON…

ui-export-as-markdown = Export as Markdown…

ui-replace = Replace…

ui-stop = Stop

ui-ai-assist = AI assist

ui-enable-ai-in-settings-ai-search = Enable AI in Settings → AI Search to use this panel.

ui-filter-results-substring-or-regex = Filter results — substring or regex

ui-filter-results = Filter results

ui-regex-2 = regex

ui-clear-filter = Clear filter

ui-path = Path

ui-line = Line

ui-match = Match

ui-search-results = Search results

ui-search-anywhere-on-your-system = Search anywhere on your system

ui-pick-a-folder-type-a-term-122c6a = Pick a folder, type a term, and we'll stream matches as they appear.

ui-code-todo = ~/code · TODO

ui-fn-test = ~ · fn .* test

ui-etc-password = /etc · password

ui-no-matches-found = No matches found

ui-the-result-filter-1-hid-every-71d7ee = The result filter '%1' hid every row. Clear it to see the raw matches, or widen the search.

ui-try-a-shorter-term-drop-a-b9718c = Try a shorter term, drop a filter, or pick a broader folder. Hidden files, gitignored paths, and binary content are excluded by default — flip those toggles in the Filters drawer.

ui-open-filters = Open Filters

ui-searching-2 = Searching…

ui-ready = Ready

ui-scanned-1-059984 = scanned %1

ui-recent-1-8f02bd = recent %1

ui-save-search-as-profile = Save search as profile

ui-profile-name = Profile name

ui-profile-name-example = 例: "~/code の TODO"

ui-cancel = Cancel

ui-save = Save

ui-export-results = Export results

ui-json-json = JSON (*.json)

ui-markdown-md = Markdown (*.md)

ui-csv-csv = CSV (*.csv)

ui-choose-folder = Choose folder

ui-changes-apply-to-the-next-search-88d3fc = Changes apply to the next search and also persist as defaults for new sessions.

ui-respect-gitignore = Respect .gitignore

ui-include-hidden-files-dotfiles = Include hidden files (dotfiles)

ui-include-binary-extracted-docs = Include binary / extracted docs

ui-include-system-files = Include system files

ui-include-subfolders-recursive = Include subfolders (recursive)

ui-follow-symbolic-links = Follow symbolic links

ui-match-file-names = Match file names

ui-exclude-directories = Exclude directories

ui-replace-matches = Replace matches

ui-replace-every-match-in-1-files-2617ae = Replace every match in %1 files. The original files are rewritten in place — there is no undo.

ui-replace-every-match-in-1-files-35a611 = Replace every match in %1 files. (Confirmation disabled in Settings.)

ui-replacement = Replacement

ui-replacement-text-regex-captures-1-name-4bb787 = Replacement text (regex captures: $1, ${name})

ui-replacement-text = Replacement text

ui-a-journal-of-rewritten-files-lives-52e0cc = A journal of rewritten files lives at $XDG_STATE_HOME/grexa/replace-journal.json until grexa exits cleanly.

ui-replacing = Replacing…

ui-replace-all = Replace All

ui-replace-complete = Replace complete

ui-replace-finished = Replace finished.

ui-ask-about-the-codebase-your-query-9e7495 = Ask about the codebase. Your query is sent to the configured endpoint only when the panel is enabled in Settings.

ui-autosaved-to-configgrexasettingsjson = Auto-saved to ~/.config/grexa/settings.json

ui-save-failed = Save failed

ui-saved = Saved

ui-reload = Reload

ui-reread-settingsjson-from-disk-useful-after-a0edd0 = Re-read settings.json from disk (useful after editing the file by hand).

ui-appearance = Appearance

ui-theme-variant-the-gtkplasma-host-palette-6a5274 = Theme variant — the GTK/Plasma host palette still drives the chrome; this picks the in-app accent.

ui-theme = Theme

ui-follow-system = Follow system

ui-light = Light

ui-dark = Dark

ui-search-defaults = Search defaults

ui-applied-to-every-new-tab-you-5b6703 = Applied to every new tab. You can still toggle these per-search in the Search page.

ui-regex-by-default = Regex by default

ui-filesmode-by-default = Files-mode by default

ui-case-sensitive = Case sensitive

ui-include-subfolders = Include subfolders

ui-include-hidden = Include hidden

ui-include-binarydocs = Include binary/docs

ui-filter-defaults = Filter defaults

ui-glob-patterns-and-directory-excludes-that-0e7194 = Glob patterns and directory excludes that pre-populate every new search.

ui-match-files = Match files

ui-exclude-dirs = Exclude dirs

ui-context-preview = Context preview

ui-how-many-lines-surround-a-match-92f6cc = How many lines surround a match when you open the preview dialog.

ui-lines-before = Lines before

ui-lines-after = Lines after

ui-containers = Containers

ui-allow-grexa-to-search-inside-running-eb34b5 = Allow Grexa to search inside running Docker and Podman containers.

ui-enable-container-search = Enable container search

ui-ai-search = AI Search

ui-openaicompatible-chat-endpoint-api-key-is-676397 = OpenAI-compatible chat endpoint. API key is stored in Secret Service (KWallet / GNOME Keyring) and never round-trips through QML.

ui-enable-ai-chat-panel-on-the = Enable AI chat panel on the Search page

ui-endpoint = Endpoint

ui-model = Model

ui-api-key = API key

ui-api-key-stored = •••••• （保存済み）

ui-paste-a-key = paste a key…

ui-key-stored = Key stored.

ui-no-key-stored = No key stored.

ui-test-endpoint = Test endpoint

ui-editor = Editor

ui-which-editor-opens-when-you-choose-b1c23a = Which editor opens when you choose “Open in editor” from a result row.

ui-preset = Preset

ui-editor-preset = Editor preset

ui-jetbrains-ide = JetBrains IDE

ui-neovim-terminal = Neovim (terminal)

ui-system-default-xdgopen = System default (xdg-open)

ui-custom-command-overrides-preset-supports-path-65d401 = Custom command (overrides preset; supports {path} and {line})

ui-custom-command = Custom command

ui-replace-2 = Replace

ui-safety-recovery-options-for-the-irreversible = Safety + recovery options for the irreversible rewrite flow.

ui-confirm-before-replacing = Confirm before replacing

ui-surface-residual-journal-on-startup = Surface residual journal on startup

ui-accessibility = Accessibility

ui-reduced-motion-disables-resultrow-transitions-and-bfd4cf = Reduced motion disables result-row transitions and busy spinners. High contrast nudges the palette toward higher legibility.

ui-reduce-motion = Reduce motion

ui-high-contrast = High contrast

ui-privacy = Privacy

ui-redact-filesystem-paths-from-grexaguilog-and-eb384e = Redact filesystem paths from grexa-gui.log and any crash diagnostics generated locally.

ui-redact-paths-in-diagnostics = Redact paths in diagnostics

ui-diagnostics = Diagnostics

ui-where-grexa-writes-its-logs-and = Where Grexa writes its logs and how to control verbosity.

ui-log-xdgstatehomegrexagrexaguilog = Log: $XDG_STATE_HOME/grexa/grexa-gui.log

ui-filter-grexaloginfogrexacoredebug = Filter: GREXA_LOG=info,grexa_core=debug

# Backfilled keys that were referenced in QML but missing from the catalog.
ui-version-prefix = v%1
ui-no-matches-for-query = 「%1」に一致する結果はありません。
ui-regex-builder-placeholder = パターン
ui-residual-journal-summary = ルート: %1\n検索語: %2\n置換: %3\n変更済み: %4\n失敗: %5
ui-tools-section = ツール
ui-try-one-of-these = 以下をお試しください
ui-whats-inside = 内容
ui-workspace-section = ワークスペース
