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

#include "ModsPage.h"

#include <QDialogButtonBox>
#include <QHBoxLayout>
#include <QLabel>
#include <QMessageBox>
#include <QPushButton>
#include <QScrollArea>
#include <QVBoxLayout>

#include <algorithm>

#include "Application.h"
#include "InstanceList.h"
#include "icons/IconList.h"
#include "minecraft/MinecraftInstance.h"
#include "minecraft/PackProfile.h"
#include "minecraft/mod/ModFolderModel.h"
#include "ui/dialogs/ResourceDownloadDialog.h"
#include "ui/shell/ProfileModsInstaller.h"

namespace {

bool canHaveMods(MinecraftInstance* instance)
{
    ProfileInfo::ensureLoaded(instance);
    return instance->getPackProfile()->getModLoaders().has_value();
}

MinecraftInstance* findProfile(const QString& id)
{
    return dynamic_cast<MinecraftInstance*>(APPLICATION->instances()->getInstanceById(id));
}

}  // namespace

ModsPage::ModsPage(QWidget* parent)
    : QWidget(parent)
    , m_chips(new QWidget)
    , m_chipLayout(new QHBoxLayout(m_chips))
    , m_hint(new QLabel(this))
    , m_placeholder(new QLabel(this))
{
    setObjectName("modsPage");

    auto* layout = new QVBoxLayout(this);
    layout->setContentsMargins(16, 12, 16, 0);
    layout->setSpacing(8);

    auto* title = new QLabel(tr("Mods"), this);
    title->setStyleSheet("font-size: 22px; font-weight: 700;");
    layout->addWidget(title);

    auto* intro = new QLabel(tr("Pick the profiles to install into. Search results are filtered for the first one you pick."), this);
    intro->setWordWrap(true);
    layout->addWidget(intro);

    m_chips->setObjectName("profileChips");
    m_chipLayout->setContentsMargins(0, 0, 0, 0);
    m_chipLayout->setSpacing(6);
    m_chips->setStyleSheet(
        "#profileChips QPushButton { border: 1px solid palette(mid); border-radius: 14px; padding: 4px 12px; }"
        "#profileChips QPushButton:checked { background: palette(highlight); color: palette(highlighted-text); "
        "border-color: palette(highlight); }"
        "#profileChips QPushButton:disabled { color: palette(mid); }");

    auto* chipScroll = new QScrollArea(this);
    chipScroll->setWidget(m_chips);
    chipScroll->setWidgetResizable(true);
    chipScroll->setFrameShape(QFrame::NoFrame);
    chipScroll->setVerticalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
    chipScroll->setFixedHeight(48);
    layout->addWidget(chipScroll);

    m_hint->setWordWrap(true);
    m_hint->setTextFormat(Qt::RichText);
    layout->addWidget(m_hint);

    m_browserArea = new QVBoxLayout;
    m_browserArea->setContentsMargins(0, 0, 0, 0);
    layout->addLayout(m_browserArea, 1);

    m_placeholder->setAlignment(Qt::AlignCenter);
    m_placeholder->setWordWrap(true);
    m_placeholder->setText(
        tr("Pick a profile above to start browsing Modrinth and CurseForge.\n\n"
           "Profiles need a mod loader such as Forge, NeoForge, Fabric or Quilt. "
           "You can add one from a profile's Version page."));
    m_browserArea->addWidget(m_placeholder);

    auto* instances = APPLICATION->instances();
    auto refresh = [this] {
        if (isVisible()) {
            rebuildChips();
        }
    };
    connect(instances, &QAbstractItemModel::rowsInserted, this, refresh);
    connect(instances, &QAbstractItemModel::rowsRemoved, this, refresh);
    connect(instances, &QAbstractItemModel::modelReset, this, refresh);
    connect(instances, &QAbstractItemModel::dataChanged, this, refresh);
    connect(APPLICATION->icons(), &IconList::iconUpdated, this, refresh);
}

void ModsPage::showEvent(QShowEvent* event)
{
    QWidget::showEvent(event);
    // Mod loaders may have been added or removed while we were hidden
    rebuildChips();
}

void ModsPage::focusProfile(const QString& instanceId)
{
    auto* instance = findProfile(instanceId);
    if (!instance || !canHaveMods(instance)) {
        return;
    }
    m_checkedIds.removeAll(instanceId);
    m_checkedIds.prepend(instanceId);
    if (isVisible()) {
        rebuildChips();
    }
}

void ModsPage::rebuildChips()
{
    while (auto* item = m_chipLayout->takeAt(0)) {
        // A chip may be rebuilt from inside its own toggled handler
        if (auto* widget = item->widget()) {
            widget->hide();
            widget->deleteLater();
        }
        delete item;
    }

    QList<MinecraftInstance*> profiles;
    auto* instances = APPLICATION->instances();
    for (int i = 0; i < instances->count(); i++) {
        if (auto* instance = dynamic_cast<MinecraftInstance*>(instances->at(i))) {
            profiles << instance;
        }
    }
    std::ranges::sort(profiles,
                      [](MinecraftInstance* a, MinecraftInstance* b) { return QString::localeAwareCompare(a->name(), b->name()) < 0; });

    m_checkedIds.removeIf([](const QString& id) {
        auto* instance = findProfile(id);
        return !instance || !canHaveMods(instance);
    });
    if (m_checkedIds.isEmpty()) {
        // Start with the profile selected elsewhere in the launcher
        auto selected = APPLICATION->settings()->get("SelectedInstance").toString();
        if (auto* instance = findProfile(selected); instance && canHaveMods(instance)) {
            m_checkedIds << selected;
        }
    }

    for (auto* instance : profiles) {
        auto id = instance->id();
        auto* chip = new QPushButton(APPLICATION->icons()->getIcon(instance->iconKey()), instance->name(), m_chips);
        chip->setCheckable(true);
        chip->setIconSize(QSize(20, 20));
        chip->setCursor(Qt::PointingHandCursor);
        if (canHaveMods(instance)) {
            chip->setToolTip(ProfileInfo::summary(instance));
            chip->setChecked(m_checkedIds.contains(id));
        } else {
            chip->setEnabled(false);
            chip->setToolTip(tr("%1\nAdd a mod loader such as Forge, NeoForge, Fabric or Quilt from this profile's Version page first.")
                                 .arg(ProfileInfo::summary(instance)));
        }
        connect(chip, &QPushButton::toggled, this, [this, id, chip = QPointer<QPushButton>(chip)](bool checked) {
            chipToggled(id, checked);
            // The user may have backed out of switching profiles
            if (chip) {
                QSignalBlocker blocker(chip);
                chip->setChecked(m_checkedIds.contains(id));
            }
        });
        m_chipLayout->addWidget(chip);
    }
    if (profiles.isEmpty()) {
        m_chipLayout->addWidget(new QLabel(tr("You don't have any profiles yet. Add one from the toolbar."), m_chips));
    }
    m_chipLayout->addStretch(1);

    updateHint();
    syncBrowser();
}

void ModsPage::chipToggled(const QString& instanceId, bool checked)
{
    auto updated = m_checkedIds;
    updated.removeAll(instanceId);
    if (checked) {
        updated << instanceId;
    }

    auto newReference = updated.value(0);
    if (newReference != m_browserProfileId && m_browser && !m_browser->getTasks().isEmpty()) {
        auto answer = QMessageBox::question(this, tr("Switch profile?"),
                                            tr("Search results follow the first profile you picked, so switching it clears the mods you "
                                               "have selected. Continue?"),
                                            QMessageBox::Yes | QMessageBox::No, QMessageBox::No);
        if (answer != QMessageBox::Yes) {
            return;
        }
    }

    m_checkedIds = updated;
    updateHint();
    syncBrowser();
}

void ModsPage::updateHint()
{
    auto profiles = checkedProfiles();
    if (profiles.isEmpty()) {
        m_hint->clear();
        m_hint->hide();
        return;
    }
    auto* reference = profiles.takeFirst();
    auto text =
        tr("Showing mods for <b>%1</b> (%2).").arg(reference->name().toHtmlEscaped(), ProfileInfo::summary(reference).toHtmlEscaped());
    if (!profiles.isEmpty()) {
        QStringList names;
        for (auto* instance : profiles) {
            names << instance->name().toHtmlEscaped();
        }
        text += " " + tr("Also installs into %1, using the best matching version for each.").arg(names.join(", "));
    }
    m_hint->setText(text);
    m_hint->show();
}

void ModsPage::syncBrowser(bool force)
{
    auto* reference = referenceProfile();
    auto referenceId = reference ? reference->id() : QString();
    if (!force && referenceId == m_browserProfileId && (m_browser || !reference)) {
        return;
    }

    if (m_browser) {
        m_browser->disconnect(this);
        m_browser->hide();
        m_browser->deleteLater();
        m_browser = nullptr;
    }
    m_browserProfileId = referenceId;
    m_placeholder->setVisible(!reference);
    if (!reference) {
        return;
    }

    reference->loaderModList()->update();

    // The browser remembers its geometry for the per-profile dialog; don't let the embedded copy overwrite that
    auto savedGeometry = APPLICATION->settings()->get("ModDownloadGeometry");

    auto* browser = new ResourceDownload::ModDownloadDialog(this, reference->loaderModList(), reference);
    browser->setWindowFlags(Qt::Widget);
    browser->setSizeGripEnabled(false);
    if (auto* buttons = browser->findChild<QDialogButtonBox*>()) {
        if (auto* cancel = buttons->button(QDialogButtonBox::Cancel)) {
            cancel->setText(tr("Clear selection"));
        }
    }
    connect(browser, &QDialog::finished, this, [this, savedGeometry](int result) {
        APPLICATION->settings()->set("ModDownloadGeometry", savedGeometry);
        browserFinished(result);
    });
    m_browserArea->addWidget(browser);
    browser->show();
    m_browser = browser;
}

void ModsPage::browserFinished(int result)
{
    if (result == QDialog::Accepted && m_browser) {
        auto profiles = checkedProfiles();
        if (!profiles.isEmpty() && profiles.first()->id() == m_browserProfileId) {
            auto* reference = profiles.takeFirst();
            ProfileModsInstaller::install(window(), reference, profiles, m_browser->getTasks());
        }
    }
    // Start over with an empty selection
    syncBrowser(true);
}

MinecraftInstance* ModsPage::referenceProfile() const
{
    auto profiles = checkedProfiles();
    return profiles.isEmpty() ? nullptr : profiles.first();
}

QList<MinecraftInstance*> ModsPage::checkedProfiles() const
{
    QList<MinecraftInstance*> result;
    for (const auto& id : m_checkedIds) {
        if (auto* instance = findProfile(id); instance && canHaveMods(instance)) {
            result << instance;
        }
    }
    return result;
}
