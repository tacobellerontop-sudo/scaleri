// SPDX-License-Identifier: GPL-3.0-only
/*
 *  Prism Launcher - Minecraft Launcher
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

#include <QPointer>
#include <QStringList>
#include <QWidget>

class MinecraftInstance;
class QHBoxLayout;
class QLabel;
class QVBoxLayout;

namespace ResourceDownload {
class ModDownloadDialog;
}

/**
 * The Mods page of the main window: tick one or more profiles at the top, browse Modrinth and CurseForge below,
 * and the chosen mods are installed into every ticked profile.
 */
class ModsPage : public QWidget {
    Q_OBJECT

   public:
    explicit ModsPage(QWidget* parent = nullptr);

    /** Makes the given profile the one search results are filtered for, ticking it if needed. */
    void focusProfile(const QString& instanceId);

   protected:
    void showEvent(QShowEvent* event) override;

   private:
    void rebuildChips();
    void chipToggled(const QString& instanceId, bool checked);
    void updateHint();

    /** Recreates the browser if the profile it is filtered for changed (or always, if force is set). */
    void syncBrowser(bool force = false);
    void browserFinished(int result);

    MinecraftInstance* referenceProfile() const;
    QList<MinecraftInstance*> checkedProfiles() const;

    QWidget* m_chips;
    QHBoxLayout* m_chipLayout;
    QLabel* m_hint;
    QVBoxLayout* m_browserArea;
    QLabel* m_placeholder;

    QPointer<ResourceDownload::ModDownloadDialog> m_browser;
    QString m_browserProfileId;

    /** Ticked profiles, in the order they were ticked. The first one filters the search results. */
    QStringList m_checkedIds;
};
