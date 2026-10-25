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

#include <QCoreApplication>
#include <QDialog>
#include <QList>

class MinecraftInstance;
class QDialogButtonBox;
class QListWidget;

/** Lets the user tick the profiles (instances) that mods should be added to. */
class ChooseProfilesDialog : public QDialog {
    Q_OBJECT

   public:
    explicit ChooseProfilesDialog(QWidget* parent, const QString& preselectedInstanceId = {});

    /** The ticked profiles, in list order. */
    QList<MinecraftInstance*> selectedInstances() const;

   private:
    void updateOkButton();

    QListWidget* m_list;
    QDialogButtonBox* m_buttons;
};

/**
 * The launcher-wide Mods page: pick profiles, browse Modrinth / CurseForge, and install the chosen mods
 * into every picked profile, each getting the newest version that matches its own game version and mod loader.
 */
class ProfileModsInstaller {
    Q_DECLARE_TR_FUNCTIONS(ProfileModsInstaller)

   public:
    static void run(QWidget* parent, const QString& preselectedInstanceId = {});
};
