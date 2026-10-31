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

#include "InstallScaleriClient.h"

#include "scaleri/ScaleriClient.h"

InstallScaleriClient::InstallScaleriClient(LaunchTask* parent, MinecraftInstance* instance, bool online)
    : LaunchStep(parent), m_instance(instance), m_online(online)
{}

void InstallScaleriClient::executeTask()
{
    if (!ScaleriClient::isEnabled(m_instance)) {
        emitSucceeded();
        return;
    }

    const auto modsDir = m_instance->modsRoot();
    QString reason;
    if (!ScaleriClient::isCompatible(m_instance, &reason)) {
        // Leaving the jar in place would stop the game with a dependency error.
        ScaleriClient::removeJar(modsDir);
        emit logLine(tr("%1 Scaleri Client is turned off for this launch.").arg(reason), MessageLevel::Warning);
        emitSucceeded();
        return;
    }

    QString error;
    switch (ScaleriClient::installJar(modsDir, &error)) {
        case ScaleriClient::InstallResult::Installed:
            emit logLine(tr("Installed Scaleri Client %1.").arg(ScaleriClient::version()), MessageLevel::Launcher);
            break;
        case ScaleriClient::InstallResult::UpToDate:
            emit logLine(tr("Scaleri Client %1 is up to date.").arg(ScaleriClient::version()), MessageLevel::Launcher);
            break;
        case ScaleriClient::InstallResult::NotBundled:
            emit logLine(error, MessageLevel::Warning);
            break;
        case ScaleriClient::InstallResult::Failed:
            emit logLine(error, MessageLevel::Fatal);
            emitFailed(error);
            return;
    }

    if (ScaleriClient::hasFabricApi(modsDir)) {
        emitSucceeded();
        return;
    }
    if (!m_online) {
        auto message = tr("Scaleri Client needs Fabric API, which this profile doesn't have yet. Launch online once to download it.");
        emit logLine(message, MessageLevel::Fatal);
        emitFailed(message);
        return;
    }

    emit logLine(tr("Downloading Fabric API for Scaleri Client..."), MessageLevel::Launcher);
    m_download.reset(new ScaleriClient::FabricApiDownloadTask(modsDir, ScaleriClient::minecraftVersion()));
    connect(m_download.get(), &Task::succeeded, this, &InstallScaleriClient::emitSucceeded);
    connect(m_download.get(), &Task::failed, this, [this](const QString& reason) {
        auto message = tr("Couldn't download Fabric API for Scaleri Client: %1").arg(reason);
        emit logLine(message, MessageLevel::Fatal);
        emitFailed(message);
    });
    connect(m_download.get(), &Task::aborted, this, &InstallScaleriClient::emitAborted);
    connect(m_download.get(), &Task::status, this, &InstallScaleriClient::setStatus);
    connect(m_download.get(), &Task::progress, this, &InstallScaleriClient::setProgress);
    m_download->start();
}

bool InstallScaleriClient::abort()
{
    if (m_download && m_download->isRunning()) {
        return m_download->abort();
    }
    return true;
}
