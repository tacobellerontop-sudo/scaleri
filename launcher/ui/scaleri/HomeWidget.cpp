// SPDX-License-Identifier: GPL-3.0-only
/*
 *  Scaleri - Minecraft Launcher
 *  Copyright (C) 2026 Scaleri Contributors
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
#include "HomeWidget.h"

#include <QAction>
#include <cmath>
#include <QHBoxLayout>
#include <QLabel>
#include <QLineEdit>
#include <QLinearGradient>
#include <QMenu>
#include <QPainter>
#include <QToolButton>
#include <QVBoxLayout>

#include "Application.h"
#include "BaseInstance.h"
#include "BuildConfig.h"
#include "MMCTime.h"

namespace {

// Small deterministic hash, so the landscape looks the same on every repaint
quint32 hash(int x, int y)
{
    quint32 h = static_cast<quint32>(x) * 374761393u + static_cast<quint32>(y) * 668265263u;
    h = (h ^ (h >> 13)) * 1274126177u;
    return h ^ (h >> 16);
}

// Smooth 1D value noise in [0, 1]
double noise1d(double x, int seed)
{
    int i = static_cast<int>(std::floor(x));
    double f = x - i;
    double a = (hash(i, seed) & 0xffff) / 65535.0;
    double b = (hash(i + 1, seed) & 0xffff) / 65535.0;
    double t = f * f * (3 - 2 * f);
    return a + (b - a) * t;
}

QColor mixColor(const QColor& a, const QColor& b, double t)
{
    return QColor::fromRgbF(a.redF() + (b.redF() - a.redF()) * t, a.greenF() + (b.greenF() - a.greenF()) * t,
                            a.blueF() + (b.blueF() - a.blueF()) * t);
}

QColor jitter(const QColor& c, quint32 h, int amount)
{
    int d = static_cast<int>(h % (2 * amount + 1)) - amount;
    return QColor(qBound(0, c.red() + d, 255), qBound(0, c.green() + d, 255), qBound(0, c.blue() + d, 255));
}

}  // namespace

HeroBanner::HeroBanner(QWidget* parent) : QWidget(parent)
{
    setObjectName("heroBanner");
    setAttribute(Qt::WA_OpaquePaintEvent);
    setMinimumHeight(250);
}

void HeroBanner::paintEvent(QPaintEvent*)
{
    QPainter p(this);
    const int b = 12;  // block size in pixels
    const int cols = width() / b + 1;
    const int rows = height() / b + 1;

    const QColor skyTop(0x16, 0x18, 0x33);
    const QColor skyMid(0x2e, 0x2a, 0x5c);
    const QColor skyLow(0x7a, 0x4b, 0x6e);
    const QColor farHills(0x25, 0x24, 0x3e);
    const QColor midHills(0x1d, 0x30, 0x2a);
    const QColor grass(0x5f, 0xae, 0x34);
    const QColor grassTop(0x8b, 0xd4, 0x4a);
    const QColor dirt(0x6b, 0x4a, 0x2f);
    const QColor stone(0x55, 0x56, 0x58);

    // sky, in banded blocks like a Minecraft sunset
    for (int y = 0; y < rows; y++) {
        double t = static_cast<double>(y) / qMax(1, rows - 1);
        QColor row = t < 0.55 ? mixColor(skyTop, skyMid, t / 0.55) : mixColor(skyMid, skyLow, (t - 0.55) / 0.45);
        for (int x = 0; x < cols; x++) {
            p.fillRect(x * b, y * b, b, b, jitter(row, hash(x, y), 3));
        }
    }

    // stars
    for (int y = 0; y < rows / 2; y++) {
        for (int x = 0; x < cols; x++) {
            quint32 h = hash(x, y + 1000);
            if (h % 23 == 0) {
                int s = (h >> 8) % 3 == 0 ? 4 : 2;
                p.fillRect(x * b + int((h >> 4) % (b - s)), y * b + int((h >> 12) % (b - s)), s, s, QColor(0xe8, 0xe6, 0xff));
            }
        }
    }

    // a square moon
    const int moonX = cols - 9;
    p.fillRect(moonX * b, 2 * b, 3 * b, 3 * b, QColor(0xf2, 0xee, 0xd8));
    p.fillRect(moonX * b + b, 2 * b + b, b, b, QColor(0xd8, 0xd3, 0xb8));
    p.fillRect(moonX * b + 2 * b, 2 * b, b, b, QColor(0xe2, 0xdd, 0xc4));

    auto ridge = [&](double scale, int seed, double base, double amp, int x) {
        double n = noise1d(x / scale, seed) * 0.7 + noise1d(x / (scale / 3), seed + 7) * 0.3;
        return static_cast<int>(rows * base - n * amp);
    };

    // far mountains
    for (int x = 0; x < cols; x++) {
        int top = ridge(14.0, 1, 0.78, rows * 0.45, x);
        for (int y = top; y < rows; y++)
            p.fillRect(x * b, y * b, b, b, jitter(farHills, hash(x, y + 2000), 2));
    }

    // forest hills
    for (int x = 0; x < cols; x++) {
        int top = ridge(9.0, 2, 0.92, rows * 0.32, x);
        for (int y = top; y < rows; y++)
            p.fillRect(x * b, y * b, b, b, jitter(midHills, hash(x, y + 3000), 3));
        // the odd pixel tree on the ridge
        if (hash(x, 4000) % 9 == 0) {
            p.fillRect(x * b, (top - 2) * b, b, 2 * b, jitter(midHills, hash(x, 4001), 2).darker(115));
            p.fillRect((x - 1) * b, (top - 4) * b, 3 * b, 2 * b, jitter(midHills, hash(x, 4002), 4));
        }
    }

    // grass and dirt in the foreground
    for (int x = 0; x < cols; x++) {
        int top = rows - 3 - static_cast<int>(noise1d(x / 6.0, 3) * 2.5);
        for (int y = top; y < rows; y++) {
            QColor c;
            if (y == top)
                c = jitter(grass, hash(x, y + 5000), 6);
            else if (y - top > 3 && hash(x, y + 5001) % 4 == 0)
                c = jitter(stone, hash(x, y + 5002), 6);
            else
                c = jitter(dirt, hash(x, y + 5003), 7);
            p.fillRect(x * b, y * b, b, b, c);
        }
        p.fillRect(x * b, top * b, b, 3, grassTop);
    }

    // darken the left side and bottom so the text and buttons stay readable
    QLinearGradient side(0, 0, width(), 0);
    side.setColorAt(0.0, QColor(18, 18, 19, 235));
    side.setColorAt(0.45, QColor(18, 18, 19, 170));
    side.setColorAt(1.0, QColor(18, 18, 19, 20));
    p.fillRect(rect(), side);

    QLinearGradient bottom(0, height() * 0.5, 0, height());
    bottom.setColorAt(0.0, QColor(18, 18, 19, 0));
    bottom.setColorAt(1.0, QColor(18, 18, 19, 160));
    p.fillRect(rect(), bottom);

    p.fillRect(0, height() - 2, width(), 2, QColor(0x12, 0x12, 0x13));
}

HomeWidget::HomeWidget(QWidget* parent) : QWidget(parent)
{
    setObjectName("scaleriHome");

    auto root = new QHBoxLayout(this);
    root->setContentsMargins(0, 0, 0, 0);
    root->setSpacing(0);

    // ---------- sidebar ----------
    auto sidebar = new QWidget(this);
    sidebar->setObjectName("scaleriSidebar");
    sidebar->setAttribute(Qt::WA_StyledBackground);
    sidebar->setFixedWidth(208);
    auto sidebarLayout = new QVBoxLayout(sidebar);
    sidebarLayout->setContentsMargins(12, 16, 12, 12);
    sidebarLayout->setSpacing(4);

    auto brand = new QWidget(sidebar);
    auto brandLayout = new QHBoxLayout(brand);
    brandLayout->setContentsMargins(4, 0, 0, 12);
    brandLayout->setSpacing(10);
    auto logo = new QLabel(brand);
    logo->setPixmap(APPLICATION->logo().pixmap(36, 36));
    auto wordmark = new QLabel(BuildConfig.LAUNCHER_DISPLAYNAME.toUpper(), brand);
    wordmark->setObjectName("scaleriWordmark");
    QFont wordmarkFont = wordmark->font();
    wordmarkFont.setBold(true);
    wordmarkFont.setPointSizeF(wordmarkFont.pointSizeF() * 1.45);
    wordmarkFont.setLetterSpacing(QFont::AbsoluteSpacing, 2);
    wordmark->setFont(wordmarkFont);
    brandLayout->addWidget(logo);
    brandLayout->addWidget(wordmark, 1);
    sidebarLayout->addWidget(brand);

    m_sidebarTop = new QVBoxLayout();
    m_sidebarTop->setSpacing(4);
    sidebarLayout->addLayout(m_sidebarTop);
    sidebarLayout->addStretch(1);
    m_sidebarFooter = new QVBoxLayout();
    m_sidebarFooter->setSpacing(4);
    sidebarLayout->addLayout(m_sidebarFooter);

    root->addWidget(sidebar);

    // ---------- main column ----------
    auto main = new QWidget(this);
    main->setObjectName("scaleriMain");
    main->setAttribute(Qt::WA_StyledBackground);
    auto mainLayout = new QVBoxLayout(main);
    mainLayout->setContentsMargins(0, 0, 0, 0);
    mainLayout->setSpacing(0);

    auto hero = new HeroBanner(main);
    auto heroLayout = new QVBoxLayout(hero);
    heroLayout->setContentsMargins(28, 24, 28, 22);
    heroLayout->setSpacing(14);

    auto infoRow = new QHBoxLayout();
    infoRow->setSpacing(18);
    m_instanceIcon = new QToolButton(hero);
    m_instanceIcon->setObjectName("heroInstanceIcon");
    m_instanceIcon->setIconSize(QSize(88, 88));
    m_instanceIcon->setFixedSize(104, 104);
    m_instanceIcon->setCursor(Qt::PointingHandCursor);
    infoRow->addWidget(m_instanceIcon, 0, Qt::AlignTop);

    auto textColumn = new QVBoxLayout();
    textColumn->setSpacing(4);
    m_eyebrow = new QLabel(tr("SELECTED INSTANCE"), hero);
    m_eyebrow->setObjectName("heroEyebrow");
    m_instanceName = new QLabel(hero);
    m_instanceName->setObjectName("heroTitle");
    QFont titleFont = m_instanceName->font();
    titleFont.setBold(true);
    titleFont.setPointSizeF(titleFont.pointSizeF() * 2.1);
    m_instanceName->setFont(titleFont);
    m_instanceDetails = new QLabel(hero);
    m_instanceDetails->setObjectName("heroDetails");
    m_instanceDetails->setWordWrap(true);
    m_instancePlaytime = new QLabel(hero);
    m_instancePlaytime->setObjectName("heroPlaytime");
    textColumn->addWidget(m_eyebrow);
    textColumn->addWidget(m_instanceName);
    textColumn->addWidget(m_instanceDetails);
    textColumn->addWidget(m_instancePlaytime);
    textColumn->addStretch(1);
    infoRow->addLayout(textColumn, 1);
    heroLayout->addLayout(infoRow, 1);

    auto buttonRow = new QHBoxLayout();
    buttonRow->setSpacing(8);
    m_playButton = new QToolButton(hero);
    m_playButton->setObjectName("heroPlayButton");
    m_playButton->setToolButtonStyle(Qt::ToolButtonTextOnly);
    m_playButton->setMinimumSize(200, 52);
    QFont playFont = m_playButton->font();
    playFont.setBold(true);
    playFont.setPointSizeF(playFont.pointSizeF() * 1.35);
    playFont.setLetterSpacing(QFont::AbsoluteSpacing, 1.5);
    m_playButton->setFont(playFont);
    buttonRow->addWidget(m_playButton);

    auto makeHeroButton = [hero](const char* name) {
        auto button = new QToolButton(hero);
        button->setObjectName(name);
        button->setProperty("heroSecondary", true);
        button->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
        button->setMinimumHeight(52);
        return button;
    };
    m_editButton = makeHeroButton("heroEditButton");
    m_folderButton = makeHeroButton("heroFolderButton");
    m_moreButton = makeHeroButton("heroMoreButton");
    m_moreButton->setText(tr("More"));
    m_moreButton->setPopupMode(QToolButton::InstantPopup);
    m_moreMenu = new QMenu(m_moreButton);
    m_moreButton->setMenu(m_moreMenu);
    buttonRow->addWidget(m_editButton);
    buttonRow->addWidget(m_folderButton);
    buttonRow->addWidget(m_moreButton);
    buttonRow->addStretch(1);
    heroLayout->addLayout(buttonRow);

    mainLayout->addWidget(hero);

    // ---------- library ----------
    auto library = new QWidget(main);
    library->setObjectName("scaleriLibrary");
    m_libraryLayout = new QVBoxLayout(library);
    m_libraryLayout->setContentsMargins(20, 16, 20, 12);
    m_libraryLayout->setSpacing(10);

    auto libraryHeader = new QHBoxLayout();
    auto libraryTitle = new QLabel(tr("Your Instances"), library);
    libraryTitle->setObjectName("libraryTitle");
    QFont libraryFont = libraryTitle->font();
    libraryFont.setBold(true);
    libraryFont.setPointSizeF(libraryFont.pointSizeF() * 1.3);
    libraryTitle->setFont(libraryFont);
    m_search = new QLineEdit(library);
    m_search->setObjectName("librarySearch");
    m_search->setPlaceholderText(tr("Search instances"));
    m_search->setClearButtonEnabled(true);
    m_search->setMaximumWidth(280);
    libraryHeader->addWidget(libraryTitle);
    libraryHeader->addStretch(1);
    libraryHeader->addWidget(m_search);
    m_libraryLayout->addLayout(libraryHeader);

    mainLayout->addWidget(library, 1);
    root->addWidget(main, 1);

    setInstance(nullptr, QIcon());
}

QToolButton* HomeWidget::makeSidebarButton(QAction* action)
{
    auto button = new QToolButton(this);
    button->setProperty("sidebarButton", true);
    button->setDefaultAction(action);
    button->setToolButtonStyle(Qt::ToolButtonTextBesideIcon);
    button->setIconSize(QSize(22, 22));
    button->setSizePolicy(QSizePolicy::Expanding, QSizePolicy::Fixed);
    button->setMinimumHeight(40);
    button->setFocusPolicy(Qt::NoFocus);
    if (action->menu())
        button->setPopupMode(QToolButton::InstantPopup);
    return button;
}

void HomeWidget::addSidebarAction(QAction* action)
{
    m_sidebarTop->addWidget(makeSidebarButton(action));
}

void HomeWidget::addSidebarFooterAction(QAction* action)
{
    auto button = makeSidebarButton(action);
    button->setProperty("sidebarFooter", true);
    button->setIconSize(QSize(28, 28));
    button->setMinimumHeight(48);
    m_sidebarFooter->addWidget(button);
}

void HomeWidget::setInstanceActions(QAction* play, QAction* edit, QAction* folder, QAction* changeIcon, const QList<QAction*>& more)
{
    m_playButton->setDefaultAction(play);
    m_playButton->setPopupMode(QToolButton::MenuButtonPopup);
    m_playButton->setToolButtonStyle(Qt::ToolButtonTextOnly);
    m_editButton->setDefaultAction(edit);
    m_folderButton->setDefaultAction(folder);
    connect(m_instanceIcon, &QToolButton::clicked, changeIcon, &QAction::trigger);
    m_instanceIcon->setToolTip(changeIcon->toolTip());
    for (auto action : more) {
        if (action)
            m_moreMenu->addAction(action);
        else
            m_moreMenu->addSeparator();
    }
}

void HomeWidget::setInstanceView(QWidget* view)
{
    m_libraryLayout->addWidget(view, 1);
}

void HomeWidget::setInstance(BaseInstance* instance, const QIcon& icon)
{
    m_eyebrow->setVisible(instance != nullptr);
    if (!instance) {
        m_instanceIcon->setIcon(APPLICATION->logo());
        m_instanceName->setText(tr("Welcome to %1").arg(BuildConfig.LAUNCHER_DISPLAYNAME));
        m_instanceDetails->setText(tr("Pick an instance below, or create one with Add Instance."));
        m_instancePlaytime->clear();
        return;
    }

    m_instanceIcon->setIcon(icon);
    m_instanceName->setText(instance->name());
    // the status bar description starts with the game version; playtime gets its own line below
    m_instanceDetails->setText(instance->getStatusbarDescription().section(", ", 0, 0));

    const bool noDays = APPLICATION->settings()->get("ShowGameTimeWithoutDays").toBool();
    if (instance->isRunning()) {
        m_instancePlaytime->setText(tr("Running now"));
    } else if (instance->totalTimePlayed() > 0) {
        m_instancePlaytime->setText(tr("Played for %1").arg(Time::prettifyDuration(instance->totalTimePlayed(), noDays)));
    } else {
        m_instancePlaytime->setText(tr("Not played yet"));
    }
}
