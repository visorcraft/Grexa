// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use cxx_qt_build::{CxxQtBuilder, QmlModule};
use qt_build_utils::{QResource, QResourceFile, QResources};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("com.visorcraft.Grexa")
            .version(1, 0)
            .qml_files([
                "qml/Main.qml",
                "qml/SearchPage.qml",
                "qml/RegexBuilderPage.qml",
                "qml/SettingsPage.qml",
                "qml/ContextPreviewDialog.qml",
                "qml/AiChatPanel.qml",
                "qml/AboutPage.qml",
                "qml/DesignTokens.qml",
                "qml/NavItem.qml",
                "qml/ResultRow.qml",
                "qml/PrimaryButton.qml",
                "qml/EmptyState.qml",
                "qml/Card.qml",
                "qml/SearchBar.qml",
                "qml/FlagChip.qml",
            ]),
    )
    // Bundle brand + illustration assets at
    // qrc:/qt/qml/com/visorcraft/Grexa/resources/* so QML can reference
    // them regardless of whether the host has the freedesktop theme
    // installed. grexa.png is the pink-gecko brand mark (female
    // counterpart to Grex's green male gecko); empty-search.svg is the
    // empty-state illustration.
    .qrc_resources(
        QResources::new().resource(
            QResource::new()
                .file(QResourceFile::new("resources/grexa.png").alias("resources/grexa.png"))
                .file(
                    QResourceFile::new("resources/grex-mark.png").alias("resources/grex-mark.png"),
                )
                .file(
                    QResourceFile::new("resources/empty-search.svg")
                        .alias("resources/empty-search.svg"),
                ),
        ),
    )
    .file("src/qobjects/search.rs")
    .file("src/qobjects/settings.rs")
    .file("src/qobjects/regex_builder.rs")
    .file("src/qobjects/ai.rs")
    .build();
}
