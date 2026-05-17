// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use cxx_qt_build::{CxxQtBuilder, QmlModule};

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
            ]),
    )
    .file("src/qobjects/search.rs")
    .file("src/qobjects/settings.rs")
    .file("src/qobjects/regex_builder.rs")
    .file("src/qobjects/ai.rs")
    .build();
}
