// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Reserved for a future standalone `ResultsModel` QObject.
//!
//! Today the result list model is implemented directly on
//! `SearchController` (it inherits `QAbstractListModel`). If we ever
//! split search orchestration from the list model — e.g. to share a
//! single results view across multiple search tabs — that QObject
//! goes here.
//!
//! Keeping the module file in the tree as scaffolding documents the
//! design intent and reserves the QML registration path.
