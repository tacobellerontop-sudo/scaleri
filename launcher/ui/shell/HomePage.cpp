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

#include "HomePage.h"

#include <QAbstractItemView>
#include <QCheckBox>
#include <QComboBox>
#include <QEvent>
#include <QFileInfo>
#include <QFrame>
#include <QHBoxLayout>
#include <QLabel>
#include <QPushButton>
#include <QVBoxLayout>

#include <algorithm>

#include "Application.h"
#include "BaseInstance.h"
#include "BuildConfig.h"
#include "DesktopServices.h"
#include "InstanceList.h"
#include "icons/IconList.h"
#include "minecraft/MinecraftInstance.h"
#include "scaleri/ScaleriClient.h"
#include "ui/dialogs/CustomMessageBox.h"
#include "ui/dialogs/ProgressDialog.h"
#include "ui/shell/ProfileModsInstaller.h"

HomePage::HomePage(QWidget* parent)
    : QWidget(parent)
    , m_icon(new QLabel(this))
    , m_name(new QLabel(this))
    , m_details(new QLabel(this))
    , m_play(new QPushButton(this))
    , m_profiles(new QComboBox(this))
    , m_edit(new QPushButton(QIcon::fromTheme("instance-settings"), tr("Edit profile"), this))
    , m_mods(new QPushButton(QIcon::fromTheme("loadermods"), tr("Browse mods"), this))
    , m_folder(new QPushButton(QIcon::fromTheme("viewfolder"), tr("Open folder"), this))
    , m_scaleriClient(new QCheckBox(tr("Scaleri Client"), this))
    , m_addScaleriProfile(new QPushButton(APPLICATION->icons()->getIcon(ScaleriClient::IconKey), tr("Add Scaleri Client profile"), this))
{
    setObjectName("homePage");

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(24, 24, 24, 24);
    layout->setSpacing(16);

    auto* hero = new QFrame(this);
    hero->setObjectName("homeHero");
    hero->setMinimumHeight(320);
    auto* heroLayout = new QVBoxLayout(hero);
    heroLayout->setContentsMargins(36, 32, 36, 32);
    heroLayout->setSpacing(10);

    auto* title = new QLabel(BuildConfig.LAUNCHER_DISPLAYNAME.toUpper(), hero);
    title->setObjectName("homeTitle");
    heroLayout->addWidget(title);

    auto* subtitle = new QLabel(tr("Pick a profile, add some mods, and jump in."), hero);
    heroLayout->addWidget(subtitle);
    heroLayout->addStretch(1);

    auto* card = new QHBoxLayout;
    card->setSpacing(16);
    m_icon->setFixedSize(64, 64);
    m_icon->setParent(hero);
    card->addWidget(m_icon);
    auto* text = new QVBoxLayout;
    text->setSpacing(2);
    m_name->setObjectName("homeProfileName");
    m_name->setParent(hero);
    m_details->setParent(hero);
    text->addWidget(m_name);
    text->addWidget(m_details);
    card->addLayout(text, 1);
    heroLayout->addLayout(card);

    auto* actions = new QHBoxLayout;
    actions->setSpacing(12);
    m_play->setObjectName("homePlay");
    m_play->setParent(hero);
    m_play->setCursor(Qt::PointingHandCursor);
    m_play->setDefault(true);
    actions->addWidget(m_play);
    m_profiles->setParent(hero);
    m_profiles->setMinimumWidth(220);
    m_profiles->setIconSize(QSize(20, 20));
    m_profiles->setToolTip(tr("Profile to play"));
    actions->addWidget(m_profiles);
    actions->addStretch(1);
    heroLayout->addLayout(actions);

    layout->addWidget(hero);

    auto* quick = new QHBoxLayout;
    quick->setSpacing(8);
    quick->addWidget(m_edit);
    quick->addWidget(m_mods);
    quick->addWidget(m_folder);
    quick->addSpacing(12);
    quick->addWidget(m_scaleriClient);
    quick->addStretch(1);
    quick->addWidget(m_addScaleriProfile);
    layout->addLayout(quick);
    layout->addStretch(1);

    connect(m_play, &QPushButton::clicked, this, &HomePage::play);
    connect(m_profiles, &QComboBox::activated, this, [this](int index) {
        m_currentId = m_profiles->itemData(index).toString();
        updateCard();
        emit instanceSelected(m_currentId);
    });
    connect(m_edit, &QPushButton::clicked, this, [this] {
        if (auto* instance = currentInstance(); instance && instance->canEdit()) {
            APPLICATION->showInstanceWindow(instance);
        }
    });
    connect(m_mods, &QPushButton::clicked, this, [this] { emit modsRequested(m_currentId); });
    connect(m_scaleriClient, &QCheckBox::clicked, this, &HomePage::setScaleriClientEnabled);
    connect(m_addScaleriProfile, &QPushButton::clicked, this, &HomePage::addScaleriClientProfile);
    connect(m_folder, &QPushButton::clicked, this, [this] {
        if (auto* instance = currentInstance()) {
            DesktopServices::openPath(QFileInfo(instance->instanceRoot()));
        }
    });

    auto* instances = APPLICATION->instances();
    connect(instances, &QAbstractItemModel::rowsInserted, this, &HomePage::rebuildProfiles);
    connect(instances, &QAbstractItemModel::rowsRemoved, this, &HomePage::rebuildProfiles);
    connect(instances, &QAbstractItemModel::modelReset, this, &HomePage::rebuildProfiles);
    connect(instances, &QAbstractItemModel::dataChanged, this, &HomePage::rebuildProfiles);
    connect(APPLICATION->icons(), &IconList::iconUpdated, this, &HomePage::rebuildProfiles);

    m_currentId = APPLICATION->settings()->get("SelectedInstance").toString();
    updateStyle();
    rebuildProfiles();
}

void HomePage::setCurrentInstance(const QString& instanceId)
{
    if (instanceId.isEmpty() || instanceId == m_currentId) {
        return;
    }
    m_currentId = instanceId;
    rebuildProfiles();
}

void HomePage::changeEvent(QEvent* event)
{
    if (event->type() == QEvent::PaletteChange) {
        updateStyle();
    }
    QWidget::changeEvent(event);
}

void HomePage::rebuildProfiles()
{
    if (m_profiles->view()->isVisible()) {
        return;  // don't yank the list away while the user is picking from it
    }

    QList<BaseInstance*> instances;
    auto* list = APPLICATION->instances();
    for (int i = 0; i < list->count(); i++) {
        instances << list->at(i);
    }
    std::ranges::sort(instances, [](BaseInstance* a, BaseInstance* b) { return QString::localeAwareCompare(a->name(), b->name()) < 0; });

    QSignalBlocker blocker(m_profiles);
    m_profiles->clear();
    for (auto* instance : instances) {
        m_profiles->addItem(APPLICATION->icons()->getIcon(instance->iconKey()), instance->name(), instance->id());
    }

    auto index = m_profiles->findData(m_currentId);
    if (index < 0 && m_profiles->count() > 0) {
        index = 0;
        m_currentId = m_profiles->itemData(0).toString();
    }
    m_profiles->setCurrentIndex(index);
    updateCard();
}

void HomePage::updateCard()
{
    auto* instance = currentInstance();

    if (m_watched != instance) {
        if (m_watched) {
            disconnect(m_watched, nullptr, this, nullptr);
        }
        m_watched = instance;
        if (instance) {
            connect(instance, &BaseInstance::runningStatusChanged, this, &HomePage::updateCard);
        }
    }

    m_profiles->setVisible(instance != nullptr);
    m_edit->setEnabled(instance && instance->canEdit());
    m_mods->setEnabled(instance && dynamic_cast<MinecraftInstance*>(instance));
    m_folder->setEnabled(instance != nullptr);
    m_addScaleriProfile->setVisible(!ScaleriClient::hasProfile());

    auto* minecraft = dynamic_cast<MinecraftInstance*>(instance);
    m_scaleriClient->setVisible(minecraft != nullptr);
    if (minecraft) {
        QString reason;
        const bool enabled = ScaleriClient::isEnabled(minecraft);
        const bool compatible = ScaleriClient::isCompatible(minecraft, &reason);
        m_scaleriClient->setChecked(enabled);
        m_scaleriClient->setEnabled((compatible || enabled) && !instance->isRunning());
        m_scaleriClient->setToolTip(compatible ? tr("Play this profile with Scaleri Client %1, the PvP mod that comes with %2. It's kept "
                                                    "up to date every time you launch.")
                                                     .arg(ScaleriClient::version(), BuildConfig.LAUNCHER_DISPLAYNAME)
                                               : reason);
    }

    if (!instance) {
        m_icon->setPixmap(APPLICATION->logo().pixmap(64, 64));
        m_name->setText(tr("No profiles yet"));
        m_details->setText(tr("Create a profile to choose a Minecraft version and a mod loader such as Forge or NeoForge."));
        m_play->setText(tr("CREATE PROFILE"));
        m_play->setEnabled(true);
        return;
    }

    m_icon->setPixmap(APPLICATION->icons()->getIcon(instance->iconKey()).pixmap(64, 64));
    m_name->setText(instance->name());
    if (auto* minecraft = dynamic_cast<MinecraftInstance*>(instance)) {
        m_details->setText(ProfileInfo::summary(minecraft));
    } else {
        m_details->setText(instance->instanceType());
    }

    if (instance->isRunning()) {
        m_play->setText(tr("STOP"));
        m_play->setToolTip(tr("Stop the running game."));
        m_play->setEnabled(true);
    } else {
        m_play->setText(tr("PLAY"));
        m_play->setToolTip(tr("Launch %1.").arg(instance->name()));
        m_play->setEnabled(instance->canLaunch());
    }
}

void HomePage::updateStyle()
{
    auto accent = palette().color(QPalette::Highlight);
    setStyleSheet(QString("#homeHero { border-radius: 14px; background: qlineargradient(x1:0, y1:0, x2:1, y2:1, stop:0 %1, stop:1 %2); }"
                          "#homeHero QLabel { color: #ffffff; background: transparent; }"
                          "#homeTitle { font-size: 30px; font-weight: 800; letter-spacing: 2px; }"
                          "#homeProfileName { font-size: 20px; font-weight: 700; }"
                          "#homePlay { background: #ffffff; color: %3; border: none; border-radius: 10px; font-size: 22px; "
                          "font-weight: 800; padding: 14px 56px; }"
                          "#homePlay:hover { background: #ececec; }"
                          "#homePlay:disabled { background: rgba(255, 255, 255, 0.5); }")
                      .arg(accent.darker(220).name(), accent.darker(120).name(), accent.darker(150).name()));
}

void HomePage::play()
{
    auto* instance = currentInstance();
    if (!instance) {
        emit createInstanceRequested();
        return;
    }
    if (instance->isRunning()) {
        APPLICATION->kill(instance);
    } else if (instance->canLaunch()) {
        APPLICATION->launch(instance);
    }
}

void HomePage::setScaleriClientEnabled(bool enabled)
{
    auto* instance = dynamic_cast<MinecraftInstance*>(currentInstance());
    if (!instance) {
        return;
    }
    QString error;
    if (!ScaleriClient::setEnabled(instance, enabled, &error) && !error.isEmpty()) {
        CustomMessageBox::selectable(this, tr("Scaleri Client"), error, QMessageBox::Warning)->show();
    }
    updateCard();
}

void HomePage::addScaleriClientProfile()
{
    unique_qobject_ptr<Task> task(ScaleriClient::createProfileTask());
    connect(task.get(), &Task::failed, this,
            [this](const QString& reason) { CustomMessageBox::selectable(this, tr("Error"), reason, QMessageBox::Critical)->show(); });
    ProgressDialog dialog(this);
    dialog.execWithTask(task.get());
    updateCard();
}

BaseInstance* HomePage::currentInstance() const
{
    return m_currentId.isEmpty() ? nullptr : APPLICATION->instances()->getInstanceById(m_currentId);
}
