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

#include "launch/LaunchStep.h"
#include "minecraft/MinecraftInstance.h"
#include "tasks/Task.h"

/**
 * For profiles with Scaleri Client turned on: copies the launcher's copy of the mod into the mods folder when it changed,
 * and downloads Fabric API when the profile has none.
 */
class InstallScaleriClient : public LaunchStep {
    Q_OBJECT

   public:
    InstallScaleriClient(LaunchTask* parent, MinecraftInstance* instance, bool online);
    ~InstallScaleriClient() override = default;

    void executeTask() override;
    bool canAbort() const override { return true; }
    bool abort() override;

   private:
    MinecraftInstance* m_instance;
    bool m_online;
    Task::Ptr m_download;
};
