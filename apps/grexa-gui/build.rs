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
    .file("src/qobjects.rs")
    .build();
}
