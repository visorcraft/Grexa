// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

use cxx_qt_build::{CxxQtBuilder, QmlModule};
use qt_build_utils::{QResource, QResourceFile, QResources};

fn emit_vergen() {
    use vergen_git2::{Emitter, Git2};
    let git2 = Git2::builder().sha(true).build();
    if let Err(e) = Emitter::default()
        .add_instructions(&git2)
        .and_then(|e| e.emit())
    {
        println!("cargo:warning=vergen failed: {e}, using fallback");
        println!("cargo:rustc-env=VERGEN_GIT_SHA=unknown");
    }
}

fn main() {
    emit_vergen();
    println!("cargo:rerun-if-changed=src/icon_theme.cpp");

    let builder = CxxQtBuilder::new_qml_module(
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
                "qml/LicensesPage.qml",
                "qml/CreditsPage.qml",
                "qml/GplLicenseDialog.qml",
                "qml/HistoryPage.qml",
                "qml/ProfilesPage.qml",
                "qml/DesignTokens.qml",
                "qml/NavItem.qml",
                "qml/ResultRow.qml",
                "qml/PrimaryButton.qml",
                "qml/EmptyState.qml",
                "qml/Card.qml",
                "qml/SearchBar.qml",
                "qml/FlagChip.qml",
                "qml/AppTextField.qml",
                "qml/AppComboBox.qml",
                "qml/AppCheckBox.qml",
                "qml/AppSpinBox.qml",
                "qml/AppFlatButton.qml",
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
    .file("src/qobjects/ai.rs");

    // icon_theme.cpp prepends the bundled AppDir/usr/share/icons path to Qt's
    // icon theme search paths so the AppImage can resolve Breeze symbolic
    // icons without relying on the host theme.
    let builder = unsafe {
        builder.cc_builder(|cc| {
            cc.file("src/icon_theme.cpp");
        })
    };

    builder.build();
}
