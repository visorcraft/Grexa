# Deutsche Übersetzung. Schlüsselsatz muss mit /locales/en/grexa.ftl
# übereinstimmen — `scripts/check_locale_sync.py` setzt das in CI durch.

app-name = Grexa
app-tagline = Schnelle Linux-Dateisuche

search-status-ready = Bereit
search-status-running = Suche läuft…
search-status-cancelled = Abgebrochen
search-status-error = Fehler: {$message}

search-status-found = {$matches ->
    [one] 1 Treffer gefunden
   *[other] {$matches} Treffer gefunden
} in {$files ->
    [one] 1 Datei
   *[other] {$files} Dateien
} in {$elapsed}

search-status-filtered = Zeige {$shown} von {$total} Treffern in {$files} Dateien

elapsed-subsecond = unter einer Sekunde
elapsed-seconds = {$seconds ->
    [one] 1 Sekunde
   *[other] {$seconds} Sekunden
}
elapsed-minutes-only = {$minutes ->
    [one] 1 Minute
   *[other] {$minutes} Minuten
}
elapsed-minutes-and-seconds = {$minutes ->
    [one] 1 Minute
   *[other] {$minutes} Minuten
} {$seconds ->
    [one] und 1 Sekunde
   *[other] und {$seconds} Sekunden
}

replace-status-running = Ersetzen…
replace-status-completed = {$matches ->
    [one] 1 Treffer ersetzt
   *[other] {$matches} Treffer ersetzt
} in {$files ->
    [one] 1 Datei
   *[other] {$files} Dateien
} in {$elapsed}
replace-confirm-title = Ersetzen bestätigen
replace-confirm-message = Möchten Sie {$matches} Treffer in {$files} Dateien ersetzen? Dies kann nicht rückgängig gemacht werden.

container-target-local = Lokale Dateien
container-target-docker = Docker-Container
container-target-podman = Podman-Container
container-mirror-fallback-badge = (gespiegelt)

ai-disabled-banner = KI-Suche ist deaktiviert. Aktivieren Sie sie unter Einstellungen → KI-Suche.
ai-empty-state = Klicken Sie auf KI, um eine KI-unterstützte Suche zu starten.
ai-error-not-configured = KI-Endpunkt ist nicht konfiguriert.
ai-error-empty-response = KI-Endpunkt lieferte eine leere Antwort.

action-open-in-editor = Im Editor öffnen
action-reveal-in-file-manager = Im Dateimanager anzeigen
action-copy-path = Pfad kopieren

count-matches = {$count ->
    [one] 1 Treffer
   *[other] {$count} Treffer
}
count-files = {$count ->
    [one] 1 Datei
   *[other] {$count} Dateien
}
count-files-modified = {$count ->
    [one] 1 Datei geändert
   *[other] {$count} Dateien geändert
}
count-matches-replaced = {$count ->
    [one] 1 Treffer ersetzt
   *[other] {$count} Treffer ersetzt
}
count-failures = {$count ->
    [one] 1 Fehler
   *[other] {$count} Fehler
}
