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

#include "ProfileModsInstaller.h"

#include <QHash>

#include <algorithm>
#include <optional>

#include "Application.h"
#include "ResourceDownloadTask.h"
#include "minecraft/Component.h"
#include "minecraft/MinecraftInstance.h"
#include "minecraft/PackProfile.h"
#include "minecraft/mod/Mod.h"
#include "minecraft/mod/ModFolderModel.h"
#include "minecraft/mod/tasks/GetModDependenciesTask.h"
#include "modplatform/flame/FlameAPI.h"
#include "modplatform/modrinth/ModrinthAPI.h"
#include "tasks/ConcurrentTask.h"
#include "ui/dialogs/CustomMessageBox.h"
#include "ui/dialogs/ProgressDialog.h"

namespace ProfileInfo {

void ensureLoaded(MinecraftInstance* instance)
{
    auto* profile = instance->getPackProfile();
    // A running update task is already loading the components. Reloading would abort it and restart its metadata
    // downloads while they are still running, which trips an assertion in Task::start().
    if (profile->getComponentVersion("net.minecraft").isEmpty() && !profile->getCurrentTask()) {
        profile->reload(Net::Mode::Offline);
    }
}

QString minecraftVersion(MinecraftInstance* instance)
{
    ensureLoaded(instance);
    return instance->getPackProfile()->getComponentVersion("net.minecraft");
}

QStringList modLoaderNames(MinecraftInstance* instance)
{
    QStringList names;
    auto* profile = instance->getPackProfile();
    for (int i = 0; i < profile->rowCount(); i++) {
        auto component = profile->getComponent(static_cast<size_t>(i));
        if (component && component->isEnabled() && Component::KNOWN_MODLOADERS.contains(component->getID())) {
            names << QString("%1 %2").arg(component->getName(), component->getVersion());
        }
    }
    return names;
}

QString summary(MinecraftInstance* instance)
{
    auto details = ProfileModsInstaller::tr("Minecraft %1").arg(ProfileInfo::minecraftVersion(instance));
    auto loaders = modLoaderNames(instance);
    if (loaders.isEmpty()) {
        return ProfileModsInstaller::tr("%1 - no mod loader").arg(details);
    }
    return QString("%1 - %2").arg(details, loaders.join(", "));
}

}  // namespace ProfileInfo

namespace {

bool isInstalled(MinecraftInstance* instance, const ModPlatform::IndexedPack::Ptr& pack)
{
    const auto mods = instance->loaderModList()->allMods();
    return std::any_of(mods.cbegin(), mods.cend(), [&pack](Mod* mod) {
        auto meta = mod->metadata();
        return meta && meta->provider == pack->provider && meta->project_id == pack->addonId;
    });
}

/** Picks the newest release that fits the profile, falling back to the newest beta / alpha. Versions arrive newest first. */
std::optional<ModPlatform::IndexedVersion> pickVersion(const QVector<ModPlatform::IndexedVersion>& versions,
                                                       const QString& mcVersion,
                                                       std::optional<ModPlatform::ModLoaderTypes> loaders)
{
    std::optional<ModPlatform::IndexedVersion> fallback;
    for (const auto& version : versions) {
        if (!version.mcVersion.isEmpty() && !version.mcVersion.contains(mcVersion)) {
            continue;
        }
        if (loaders && version.loaders && !(version.loaders & *loaders)) {
            continue;
        }
        if (!version.version_type.isValid() || version.version_type == ModPlatform::IndexedVersionType::Release) {
            return version;
        }
        if (!fallback) {
            fallback = version;
        }
    }
    return fallback;
}

/** Runs the task behind a progress dialog. Returns false if the user aborted it. */
bool runWithProgress(QWidget* parent, Task* task, const QString& title)
{
    bool aborted = false;
    ProgressDialog dialog(parent);
    QObject::connect(task, &Task::aborted, &dialog, [&aborted] { aborted = true; });
    dialog.setSkipButton(true, ProfileModsInstaller::tr("Abort"));
    dialog.setWindowTitle(title);
    dialog.execWithTask(task);
    return !aborted;
}

}  // namespace

void ProfileModsInstaller::install(QWidget* parent,
                                   MinecraftInstance* reference,
                                   const QList<MinecraftInstance*>& others,
                                   QList<shared_qobject_ptr<ResourceDownloadTask>> referenceTasks)
{
    auto profiles = others;
    auto downloads = std::move(referenceTasks);
    QList<ModPlatform::IndexedPack::Ptr> chosen;
    for (const auto& task : downloads) {
        // Dependencies were resolved for the reference profile only; each other profile resolves its own below
        if (task->getDownloadReason() != "dependency") {
            chosen << task->getPack();
        }
    }

    QStringList warnings;
    QHash<MinecraftInstance*, QList<std::shared_ptr<GetModDependenciesTask::PackDependency>>> picks;

    if (!profiles.isEmpty() && !chosen.isEmpty()) {
        ModrinthAPI modrinth;
        FlameAPI flame;

        auto lookup =
            makeShared<ConcurrentTask>(tr("Find compatible versions"), APPLICATION->settings()->get("NumberOfConcurrentDownloads").toInt());
        for (auto* instance : profiles) {
            auto mcVersion = ProfileInfo::minecraftVersion(instance);
            auto loaders = instance->getPackProfile()->getSupportedModLoaders();

            for (const auto& pack : chosen) {
                if (isInstalled(instance, pack)) {
                    warnings << tr("%1 is already installed in %2, skipped.").arg(pack->name, instance->name());
                    continue;
                }

                ResourceAPI* api = pack->provider == ModPlatform::ResourceProvider::FLAME ? static_cast<ResourceAPI*>(&flame) : &modrinth;
                ResourceAPI::Callback<QVector<ModPlatform::IndexedVersion>> callbacks;
                callbacks.on_succeed = [&picks, &warnings, instance, pack, mcVersion,
                                        loaders](QVector<ModPlatform::IndexedVersion>& versions) {
                    if (auto version = pickVersion(versions, mcVersion, loaders)) {
                        picks[instance].append(std::make_shared<GetModDependenciesTask::PackDependency>(pack, *version));
                    } else {
                        warnings << tr("%1 has no version for %2 (Minecraft %3), skipped.").arg(pack->name, instance->name(), mcVersion);
                    }
                };
                callbacks.on_fail = [&warnings, instance, pack](const QString& reason, int) {
                    warnings << tr("Couldn't look up %1 for %2: %3").arg(pack->name, instance->name(), reason);
                };

                ResourceAPI::VersionSearchArgs args{ .pack = pack,
                                                     .mcVersions = std::vector<Version>{ Version(mcVersion) },
                                                     .loaders = loaders,
                                                     .resourceType = ModPlatform::ResourceType::Mod };
                if (auto task = api->getProjectVersions(std::move(args), std::move(callbacks))) {
                    lookup->addTask(task);
                }
            }
        }

        if (!runWithProgress(parent, lookup.get(), tr("Finding compatible versions..."))) {
            return;
        }
    }

    bool isIndexed = !APPLICATION->settings()->get("ModMetadataDisabled").toBool();
    bool resolveDependencies = !APPLICATION->settings()->get("ModDependenciesDisabled").toBool();

    for (auto* instance : profiles) {
        if (!picks.contains(instance)) {
            continue;
        }
        auto& mods = picks[instance];
        auto* model = instance->loaderModList();

        if (resolveDependencies) {
            auto dependencies = makeShared<GetModDependenciesTask>(instance, model, mods);
            if (!runWithProgress(parent, dependencies.get(), tr("Checking dependencies for %1...").arg(instance->name()))) {
                return;
            }
            if (!dependencies->wasSuccessful()) {
                warnings << tr("Couldn't check dependencies for %1: %2").arg(instance->name(), dependencies->failReason());
            }
            warnings << dependencies->warnings();
            for (const auto& dependency : dependencies->getDependecies()) {
                downloads << makeShared<ResourceDownloadTask>(dependency->pack, dependency->version, model, isIndexed, "dependency");
            }
        }

        for (const auto& mod : mods) {
            downloads << makeShared<ResourceDownloadTask>(mod->pack, mod->version, model, isIndexed, "standalone");
        }
    }

    auto install = makeShared<ConcurrentTask>(tr("Download Mods"), APPLICATION->settings()->get("NumberOfConcurrentDownloads").toInt());
    for (const auto& task : downloads) {
        install->addTask(task);
    }

    if (!runWithProgress(parent, install.get(), tr("Downloading mods..."))) {
        warnings.prepend(tr("Download stopped by user."));
    } else if (!install->wasSuccessful()) {
        warnings.prepend(install->failReason());
    }
    warnings << install->warnings();

    reference->loaderModList()->update();
    for (auto* instance : profiles) {
        instance->loaderModList()->update();
    }

    warnings.removeAll(QString());
    if (!warnings.isEmpty()) {
        CustomMessageBox::selectable(parent, tr("Mods"), warnings.join('\n'), QMessageBox::Warning)->show();
    }
}
