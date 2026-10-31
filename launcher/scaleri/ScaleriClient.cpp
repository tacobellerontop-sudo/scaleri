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

#include "ScaleriClient.h"

#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QUrl>

#include "Application.h"
#include "BuildConfig.h"
#include "FileSystem.h"
#include "InstanceList.h"
#include "minecraft/MinecraftInstance.h"
#include "minecraft/PackProfile.h"
#include "net/ApiDownload.h"
#include "net/ChecksumValidator.h"
#include "settings/INISettingsObject.h"
#include "ui/shell/ProfileModsInstaller.h"

namespace ScaleriClient {

namespace {

constexpr auto FabricLoaderUid = "net.fabricmc.fabric-loader";
constexpr auto FabricApiProject = "P7dR8mSH";
constexpr auto ProfileCreatedSetting = "ScaleriClientProfileCreated";

QByteArray fileHash(const QString& path)
{
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly)) {
        return {};
    }
    QCryptographicHash hash(QCryptographicHash::Sha1);
    hash.addData(&file);
    return hash.result();
}

/** Jars in a mods folder that are Scaleri Client, other than the one the launcher manages. */
QStringList strayCopies(const QString& modsDir)
{
    QStringList stray;
    const auto files = QDir(modsDir).entryList({ "scaleri-client*.jar" }, QDir::Files);
    for (const auto& file : files) {
        if (file != JarName && !file.endsWith("-sources.jar")) {
            stray << FS::PathCombine(modsDir, file);
        }
    }
    return stray;
}

}  // namespace

QString bundledJar()
{
    return APPLICATION->getJarPath(JarName);
}

QString version()
{
    return BuildConfig.SCALERI_CLIENT_VERSION;
}

QString minecraftVersion()
{
    return BuildConfig.SCALERI_CLIENT_MINECRAFT_VERSION;
}

bool isEnabled(MinecraftInstance* instance)
{
    instance->loadSpecificSettings();
    return instance->settings()->get(EnabledSetting).toBool();
}

bool isCompatible(MinecraftInstance* instance, QString* reason)
{
    // The version is empty until the profile's metadata has been downloaded once; don't hold that against it.
    const auto gameVersion = ProfileInfo::minecraftVersion(instance);
    if (!gameVersion.isEmpty() && gameVersion != minecraftVersion()) {
        if (reason) {
            *reason =
                QObject::tr("Scaleri Client needs Minecraft %1, but this profile uses Minecraft %2.").arg(minecraftVersion(), gameVersion);
        }
        return false;
    }
    auto loader = instance->getPackProfile()->getComponent(FabricLoaderUid);
    if (!loader || !loader->isEnabled()) {
        if (reason) {
            *reason = QObject::tr("Scaleri Client needs Fabric Loader, which this profile doesn't use.");
        }
        return false;
    }
    return true;
}

bool setEnabled(MinecraftInstance* instance, bool enabled, QString* error)
{
    instance->loadSpecificSettings();
    if (!enabled) {
        instance->settings()->set(EnabledSetting, false);
        return removeJar(instance->modsRoot());
    }
    if (!isCompatible(instance, error)) {
        return false;
    }
    instance->settings()->set(EnabledSetting, true);
    // Fabric API is fetched on the next launch, which already needs the network for game files.
    return installJar(instance->modsRoot(), error) != InstallResult::Failed;
}

InstallResult installJar(const QString& modsDir, QString* error)
{
    return installJar(bundledJar(), modsDir, error);
}

InstallResult installJar(const QString& source, const QString& modsDir, QString* error)
{
    if (source.isEmpty() || !QFileInfo(source).isFile()) {
        if (error) {
            *error = QObject::tr("This build of %1 doesn't include Scaleri Client.").arg(BuildConfig.LAUNCHER_DISPLAYNAME);
        }
        return InstallResult::NotBundled;
    }

    for (const auto& stray : strayCopies(modsDir)) {
        if (!QFile::remove(stray)) {
            qWarning() << "Couldn't remove an old copy of Scaleri Client:" << stray;
        }
    }

    const auto target = FS::PathCombine(modsDir, JarName);
    if (QFileInfo::exists(target) && fileHash(target) == fileHash(source)) {
        return InstallResult::UpToDate;
    }

    // Copy next to the target first so a failed copy never leaves the profile without a working jar.
    const auto staging = target + ".part";
    if (!FS::ensureFolderPathExists(modsDir) || (QFileInfo::exists(staging) && !QFile::remove(staging)) || !QFile::copy(source, staging) ||
        (QFileInfo::exists(target) && !QFile::remove(target)) || !QFile::rename(staging, target)) {
        QFile::remove(staging);
        if (error) {
            *error = QObject::tr("Couldn't copy Scaleri Client into %1.").arg(modsDir);
        }
        return InstallResult::Failed;
    }
    // QFile::copy keeps the source's permissions, which are read-only in some installs (e.g. Flatpak and AppImage).
    QFile::setPermissions(target, QFile::permissions(target) | QFileDevice::WriteOwner);
    return InstallResult::Installed;
}

bool removeJar(const QString& modsDir)
{
    const auto target = FS::PathCombine(modsDir, JarName);
    return !QFileInfo::exists(target) || QFile::remove(target);
}

bool hasFabricApi(const QString& modsDir)
{
    const auto files = QDir(modsDir).entryList({ "*.jar" }, QDir::Files);
    return std::any_of(files.cbegin(), files.cend(), [](const QString& file) {
        auto name = file.toLower();
        return name.startsWith("fabric-api") || name.startsWith("fabric_api");
    });
}

bool hasProfile()
{
    auto* list = APPLICATION->instances();
    for (int i = 0; i < list->count(); i++) {
        if (auto* instance = dynamic_cast<MinecraftInstance*>(list->at(i)); instance && isEnabled(instance)) {
            return true;
        }
    }
    return false;
}

Task* createProfileTask()
{
    auto* task = new ProfileCreationTask();
    task->setName(QString("Scaleri Client %1").arg(minecraftVersion()));
    task->setIcon(IconKey);
    return APPLICATION->instances()->wrapInstanceTask(task);
}

void ensureDefaultProfile()
{
    auto settings = APPLICATION->settings();
    if (settings->get(ProfileCreatedSetting).toBool()) {
        return;
    }
    if (hasProfile()) {
        settings->set(ProfileCreatedSetting, true);
        return;
    }

    qDebug() << "Creating the Scaleri Client profile";
    auto* task = createProfileTask();
    // Only remember success, so a first start without Internet tries again next time.
    QObject::connect(task, &Task::succeeded, [] { APPLICATION->settings()->set(ProfileCreatedSetting, true); });
    QObject::connect(task, &Task::failed,
                     [](const QString& reason) { qWarning() << "Couldn't create the Scaleri Client profile:" << reason; });
    QObject::connect(task, &Task::finished, task, &QObject::deleteLater);
    task->start();
}

std::unique_ptr<MinecraftInstance> ProfileCreationTask::createInstance()
{
    setStatus(tr("Creating the Scaleri Client profile"));

    auto instance = std::make_unique<MinecraftInstance>(
        m_globalSettings, std::make_unique<INISettingsObject>(FS::PathCombine(m_stagingPath, "instance.cfg")), m_stagingPath);
    SettingsObject::Lock lock(instance->settings());

    auto components = instance->getPackProfile();
    components->buildingFromScratch();
    components->setComponentVersion("net.minecraft", minecraftVersion(), true);
    components->setComponentVersion(FabricLoaderUid, BuildConfig.SCALERI_CLIENT_LOADER_VERSION);

    instance->setName(name());
    instance->setIconKey(m_instIcon);
    instance->loadSpecificSettings();
    instance->settings()->set(EnabledSetting, true);

    QString error;
    if (installJar(FS::PathCombine(m_stagingPath, "minecraft", "mods"), &error) == InstallResult::Failed) {
        // Not fatal: the jar is copied again before every launch.
        qWarning() << error;
    }

    return instance;
}

FabricApiDownloadTask::FabricApiDownloadTask(QString modsDir, QString minecraftVersion)
    : m_modsDir(std::move(modsDir)), m_minecraftVersion(std::move(minecraftVersion))
{}

bool FabricApiDownloadTask::abort()
{
    if (m_job) {
        return m_job->abort();
    }
    return true;
}

void FabricApiDownloadTask::executeTask()
{
    setStatus(tr("Looking up Fabric API for Minecraft %1").arg(m_minecraftVersion));

    auto url = QString(R"(%1/project/%2/version?game_versions=["%3"]&loaders=["fabric"])")
                   .arg(BuildConfig.MODRINTH_PROD_URL, FabricApiProject, m_minecraftVersion);
    m_job.reset(new NetJob("Fabric API versions", APPLICATION->network()));
    auto [action, response] = Net::ApiDownload::makeByteArray(QUrl(url));
    m_job->addNetAction(action);
    connect(m_job.get(), &NetJob::succeeded, this, [this, response] {
        // Take the response out before the next request replaces the job that owns it.
        const QByteArray data = std::move(*response);
        downloadFile(data);
    });
    connect(m_job.get(), &NetJob::failed, this, &FabricApiDownloadTask::emitFailed);
    connect(m_job.get(), &NetJob::aborted, this, &FabricApiDownloadTask::emitAborted);
    m_job->start();
}

void FabricApiDownloadTask::downloadFile(const QByteArray& response)
{
    // Modrinth lists versions newest first; prefer the newest release over betas.
    const auto versions = QJsonDocument::fromJson(response).array();
    QJsonObject chosen;
    for (const auto& value : versions) {
        auto version = value.toObject();
        if (chosen.isEmpty() || (version["version_type"].toString() == "release" && chosen["version_type"].toString() != "release")) {
            chosen = version;
        }
        if (chosen["version_type"].toString() == "release") {
            break;
        }
    }

    QJsonObject file;
    for (const auto& value : chosen["files"].toArray()) {
        if (file.isEmpty() || value.toObject()["primary"].toBool()) {
            file = value.toObject();
        }
    }
    const auto fileName = QFileInfo(file["filename"].toString()).fileName();
    const auto fileUrl = QUrl(file["url"].toString());
    if (fileName.isEmpty() || !fileUrl.isValid()) {
        emitFailed(tr("Modrinth has no Fabric API release for Minecraft %1.").arg(m_minecraftVersion));
        return;
    }

    setStatus(tr("Downloading %1").arg(fileName));
    auto download = Net::Download::makeFile(fileUrl, FS::PathCombine(m_modsDir, fileName));
    if (auto sha512 = file["hashes"].toObject()["sha512"].toString(); !sha512.isEmpty()) {
        download->addValidator(new Net::ChecksumValidator(QCryptographicHash::Sha512, sha512));
    }
    m_job.reset(new NetJob("Fabric API", APPLICATION->network()));
    m_job->addNetAction(download);
    connect(m_job.get(), &NetJob::succeeded, this, &FabricApiDownloadTask::emitSucceeded);
    connect(m_job.get(), &NetJob::failed, this, &FabricApiDownloadTask::emitFailed);
    connect(m_job.get(), &NetJob::aborted, this, &FabricApiDownloadTask::emitAborted);
    connect(m_job.get(), &NetJob::progress, this, &FabricApiDownloadTask::setProgress);
    m_job->start();
}

}  // namespace ScaleriClient
