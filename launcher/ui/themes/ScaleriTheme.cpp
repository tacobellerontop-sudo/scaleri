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
#include "ScaleriTheme.h"

#include <QFile>
#include <QObject>

QString ScaleriTheme::id()
{
    return "scaleri";
}

QString ScaleriTheme::name()
{
    return QObject::tr("Scaleri");
}

QString ScaleriTheme::tooltip()
{
    return QObject::tr("Dark and blocky, inspired by Minecraft");
}

QPalette ScaleriTheme::colorScheme()
{
    QPalette palette;
    palette.setColor(QPalette::Window, QColor(0x31, 0x32, 0x33));
    palette.setColor(QPalette::WindowText, Qt::white);
    palette.setColor(QPalette::Base, QColor(0x1e, 0x1e, 0x1f));
    palette.setColor(QPalette::AlternateBase, QColor(0x25, 0x25, 0x26));
    palette.setColor(QPalette::ToolTipBase, QColor(0x10, 0x00, 0x10));
    palette.setColor(QPalette::ToolTipText, Qt::white);
    palette.setColor(QPalette::Text, Qt::white);
    palette.setColor(QPalette::Button, QColor(0x48, 0x49, 0x4a));
    palette.setColor(QPalette::ButtonText, Qt::white);
    palette.setColor(QPalette::BrightText, QColor(0xff, 0x55, 0x55));
    palette.setColor(QPalette::Light, QColor(0x6d, 0x6e, 0x70));
    palette.setColor(QPalette::Midlight, QColor(0x5a, 0x5b, 0x5c));
    palette.setColor(QPalette::Mid, QColor(0x3a, 0x3b, 0x3c));
    palette.setColor(QPalette::Dark, QColor(0x1e, 0x1e, 0x1f));
    palette.setColor(QPalette::Shadow, QColor(0x12, 0x12, 0x13));
    palette.setColor(QPalette::Link, QColor(0x9b, 0xe1, 0x5d));
    palette.setColor(QPalette::LinkVisited, QColor(0x6c, 0xc3, 0x49));
    palette.setColor(QPalette::Highlight, QColor(0x3c, 0x85, 0x27));
    palette.setColor(QPalette::HighlightedText, Qt::white);
    palette.setColor(QPalette::PlaceholderText, QColor(0x8c, 0x8d, 0x90));
    return fadeInactive(palette, fadeAmount(), fadeColor());
}

double ScaleriTheme::fadeAmount()
{
    return 0.5;
}

QColor ScaleriTheme::fadeColor()
{
    return QColor(0x31, 0x32, 0x33);
}

bool ScaleriTheme::hasStyleSheet()
{
    return true;
}

QString ScaleriTheme::appStyleSheet()
{
    QFile file(":/scaleri/scaleri.qss");
    if (!file.open(QIODevice::ReadOnly)) {
        return {};
    }
    return QString::fromUtf8(file.readAll());
}

LogColors ScaleriTheme::logColorScheme()
{
    // Minecraft chat colours, so game logs read like the in-game console
    LogColors colors = defaultLogColors(colorScheme());
    colors.foreground[MessageLevel::Launcher] = QColor(0x55, 0xff, 0xff);
    colors.foreground[MessageLevel::Debug] = QColor(0xaa, 0xaa, 0xaa);
    colors.foreground[MessageLevel::Warning] = QColor(0xff, 0xff, 0x55);
    colors.foreground[MessageLevel::Error] = QColor(0xff, 0x55, 0x55);
    colors.foreground[MessageLevel::Fatal] = QColor(0xff, 0x55, 0x55);
    return colors;
}
