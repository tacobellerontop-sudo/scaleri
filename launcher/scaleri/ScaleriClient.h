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

#include <QString>

#include "InstanceCreationTask.h"
#include "net/NetJob.h"
#include "tasks/Task.h"

class MinecraftInstance;

/**
 * Scaleri Client is the PvP mod that ships inside the launcher (built from client/ and installed next to the launcher's
 * own jars). Profiles that use it get the bundled jar copied into their mods folder before every launch, so it is
 * updated together with the launcher, plus Fabric API from Modrinth when the profile doesn't have it yet.
 */
namespace ScaleriClient {

/** Instance setting that turns Scaleri Client on for a profile. */
inline constexpr auto EnabledSetting = "ScaleriClient";
/** File name of the jar, both next to the launcher and in a profile's mods folder. */
inline constexpr auto JarName = "scaleri-client.jar";
/** Built-in instance icon for Scaleri Client profiles. */
inline constexpr auto IconKey = "scaleri_client";

/** Path of the jar that came with the launcher, or an empty string if this build doesn't bundle it. */
QString bundledJar();
/** The bundled mod's version, e.g. "0.1.0". */
QString version();
/** The Minecraft version the mod is built for, e.g. "1.21.1". */
QString minecraftVersion();

bool isEnabled(MinecraftInstance* instance);
/**
 * Whether the profile can run Scaleri Client: the right Minecraft version with Fabric Loader. If not, and `reason` is
 * given, it is set to a sentence saying why.
 */
bool isCompatible(MinecraftInstance* instance, QString* reason = nullptr);
/** Turns Scaleri Client on or off for a profile and adds or removes the jar right away. */
bool setEnabled(MinecraftInstance* instance, bool enabled, QString* error = nullptr);

enum class InstallResult { UpToDate, Installed, NotBundled, Failed };
/**
 * Copies the bundled jar into `modsDir` unless an identical copy is already there, and removes other copies of the mod
 * (e.g. a scaleri-client-0.1.0.jar added by hand) so the game doesn't load it twice.
 */
InstallResult installJar(const QString& modsDir, QString* error = nullptr);
/** Same as installJar(modsDir, error), with `source` instead of the bundled jar. */
InstallResult installJar(const QString& source, const QString& modsDir, QString* error);
/** Removes the jar the launcher installed from `modsDir`. */
bool removeJar(const QString& modsDir);

/** Whether `modsDir` has an enabled Fabric API jar. */
bool hasFabricApi(const QString& modsDir);

/** Whether any profile has Scaleri Client turned on. */
bool hasProfile();
/** Creates a Scaleri Client profile (Minecraft and Fabric Loader at the versions the mod is built for). */
Task* createProfileTask();
/** Creates the Scaleri Client profile on the launcher's first start. Does nothing on later starts. */
void ensureDefaultProfile();

/** Creates a profile with Scaleri Client turned on and the bundled jar already in its mods folder. */
class ProfileCreationTask final : public InstanceCreationTask {
    Q_OBJECT
   public:
    ProfileCreationTask() = default;

    std::unique_ptr<MinecraftInstance> createInstance() override;
};

/** Downloads the newest Fabric API release for a Minecraft version from Modrinth into a mods folder. */
class FabricApiDownloadTask final : public Task {
    Q_OBJECT
   public:
    FabricApiDownloadTask(QString modsDir, QString minecraftVersion);

    bool canAbort() const override { return true; }
    bool abort() override;

   protected:
    void executeTask() override;

   private:
    void downloadFile(const QByteArray& response);

    QString m_modsDir;
    QString m_minecraftVersion;
    NetJob::Ptr m_job;
};

}  // namespace ScaleriClient
