// SPDX-License-Identifier: GPL-3.0-only
/*
 *  Scaleri - Minecraft Launcher
 *  Copyright (C) 2026 Scaleri Contributors
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, version 3.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
#pragma once

#include <QIcon>
#include <QWidget>

class BaseInstance;
class QLabel;
class QLineEdit;
class QToolButton;
class QVBoxLayout;
class QHBoxLayout;
class QMenu;

/// The pixel-art banner at the top of the home screen. Paints a blocky Minecraft-style landscape at dusk.
class HeroBanner : public QWidget {
    Q_OBJECT
   public:
    explicit HeroBanner(QWidget* parent = nullptr);

   protected:
    void paintEvent(QPaintEvent* event) override;
};

/// Scaleri's main screen: a sidebar for launcher-wide actions, a hero banner for the selected instance with a big
/// Play button, and the instance library below it. It only arranges and displays things; every button is driven
/// by one of MainWindow's existing actions, so behaviour stays the same as the classic layout.
class HomeWidget : public QWidget {
    Q_OBJECT
   public:
    explicit HomeWidget(QWidget* parent = nullptr);

    /// Adds a button bound to `action` to the top of the sidebar.
    void addSidebarAction(QAction* action);
    /// Adds a button bound to `action` to the bottom of the sidebar (used for the account switcher).
    void addSidebarFooterAction(QAction* action);

    /// Binds the hero buttons to the instance actions.
    void setInstanceActions(QAction* play, QAction* edit, QAction* folder, QAction* changeIcon, const QList<QAction*>& more);

    /// Puts the instance view under the library header.
    void setInstanceView(QWidget* view);

    /// Shows `instance` in the hero, or an empty state when it is null.
    void setInstance(BaseInstance* instance, const QIcon& icon);

    QLineEdit* searchBox() const { return m_search; }

   private:
    QToolButton* makeSidebarButton(QAction* action);

    QVBoxLayout* m_sidebarTop = nullptr;
    QVBoxLayout* m_sidebarFooter = nullptr;
    QVBoxLayout* m_libraryLayout = nullptr;

    QToolButton* m_instanceIcon = nullptr;
    QLabel* m_eyebrow = nullptr;
    QLabel* m_instanceName = nullptr;
    QLabel* m_instanceDetails = nullptr;
    QLabel* m_instancePlaytime = nullptr;
    QToolButton* m_playButton = nullptr;
    QToolButton* m_editButton = nullptr;
    QToolButton* m_folderButton = nullptr;
    QToolButton* m_moreButton = nullptr;
    QMenu* m_moreMenu = nullptr;
    QLineEdit* m_search = nullptr;
};
