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
#include <QWidget>

class BaseInstance;
class QCheckBox;
class QComboBox;
class QLabel;
class QPushButton;

/** The Home page of the main window: the selected profile and a big Play button. */
class HomePage : public QWidget {
    Q_OBJECT

   public:
    explicit HomePage(QWidget* parent = nullptr);

    /** Shows the given profile (called when the selection changes elsewhere in the launcher). */
    void setCurrentInstance(const QString& instanceId);

   signals:
    void instanceSelected(const QString& instanceId);
    void createInstanceRequested();
    void modsRequested(const QString& instanceId);

   protected:
    void changeEvent(QEvent* event) override;

   private:
    void rebuildProfiles();
    void updateCard();
    void updateStyle();
    void play();
    void setScaleriClientEnabled(bool enabled);
    void addScaleriClientProfile();

    BaseInstance* currentInstance() const;

    QLabel* m_icon;
    QLabel* m_name;
    QLabel* m_details;
    QPushButton* m_play;
    QComboBox* m_profiles;
    QPushButton* m_edit;
    QPushButton* m_mods;
    QPushButton* m_folder;
    QCheckBox* m_scaleriClient;
    QPushButton* m_addScaleriProfile;

    QString m_currentId;
    QPointer<BaseInstance> m_watched;
};
