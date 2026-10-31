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
#include <QList>
#include <QStringList>

#include "QObjectPtr.h"
#include "ResourceDownloadTask.h"

class MinecraftInstance;
class QWidget;

namespace ProfileInfo {
/** Loads the profile's components (game version, mod loaders) if that hasn't happened yet. */
void ensureLoaded(MinecraftInstance* instance);
/** The profile's Minecraft version, loading its components if needed. */
QString minecraftVersion(MinecraftInstance* instance);
/** Display names of the profile's enabled mod loaders, e.g. "NeoForge 21.1.77". */
QStringList modLoaderNames(MinecraftInstance* instance);
/** One line describing the profile, e.g. "Minecraft 1.21.1 - NeoForge 21.1.77". */
QString summary(MinecraftInstance* instance);
}  // namespace ProfileInfo

/**
 * Installs mods picked in a browser that was filtered for one profile into that profile and into any number of others.
 * Each other profile gets the newest version of each mod that matches its own game version and mod loader, plus its own
 * dependencies.
 */
class ProfileModsInstaller {
    Q_DECLARE_TR_FUNCTIONS(ProfileModsInstaller)

   public:
    static void install(QWidget* parent,
                        MinecraftInstance* reference,
                        const QList<MinecraftInstance*>& others,
                        QList<shared_qobject_ptr<ResourceDownloadTask>> referenceTasks);
};
